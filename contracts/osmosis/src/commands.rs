use cosmwasm_std::{
    to_json_binary, BankMsg, DepsMut, Env, MessageInfo, Reply, Response, StdError, SubMsg, WasmMsg,
};
use cw_utils::one_coin;
use ibc_tracking::{
    state::{store_ibc_transfer_reply_state, IbcTransferReplyState},
    util::insert_callback_key,
};
use osmosis_router::{
    router::{build_swap_msg, get_swap_amount_out_response},
    OsmosisSwapMsg,
};
use osmosis_std::types::ibc::applications::transfer::v1::MsgTransfer;

use crate::{
    msg::{AfterSwapAction, ExecuteMsg, MsgReplyId, MultiSwapMsg},
    state::{
        load_multi_swap_state, load_swap_reply_state, remove_multi_swap_state,
        remove_swap_reply_state, store_multi_swap_state, store_swap_reply_state,
        swap_reply_state_exists, MultiSwapState, SwapReplyState,
    },
    ContractError,
};

const TRANSFER_PORT: &str = "transfer";
const IBC_PACKET_LIFITIME: u64 = 3600u64; // 1 Hour

pub fn handle_swap_with_action(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    swap_msg: OsmosisSwapMsg,
    after_swap_action: AfterSwapAction,
    local_fallback_address: String,
) -> Result<Response, ContractError> {
    // re-entrancy check
    if swap_reply_state_exists(deps.storage)? {
        return Err(ContractError::ContractLocked {
            msg: "Another swap in process already".to_owned(),
        });
    }

    store_swap_reply_state(
        deps.storage,
        &SwapReplyState {
            after_swap_action,
            local_fallback_address,
        },
    )?;

    Ok(Response::new().add_submessage(SubMsg::reply_always(
        WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_json_binary(&ExecuteMsg::ProcessSwap { swap_msg })?,
            funds: info.funds.clone(),
        },
        MsgReplyId::SwapWithActionFallback.repr(),
    )))
}

pub fn handle_process_swap(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    swap_msg: OsmosisSwapMsg,
) -> Result<Response, ContractError> {
    if info.sender.ne(&env.contract.address) {
        return Err(ContractError::Unauthorized {});
    }

    let swap_msg = build_swap_msg(deps.storage, env, one_coin(info)?, swap_msg)?;
    Ok(Response::new().add_submessage(SubMsg::reply_on_success(swap_msg, MsgReplyId::Swap.repr())))
}

pub fn handle_after_swap_action(
    deps: DepsMut,
    env: &Env,
    reply: Reply,
) -> Result<Response, ContractError> {
    let output_token_info = get_swap_amount_out_response(deps.storage, reply)?;
    let after_swap_info = load_swap_reply_state(deps.storage)?;

    let response = match after_swap_info.after_swap_action {
        AfterSwapAction::BankSend { receiver } => {
            let bank = BankMsg::Send {
                to_address: receiver,
                amount: vec![output_token_info.output_coin],
            };
            Response::new().add_message(bank)
        }
        AfterSwapAction::CustomCall {
            contract_address,
            msg,
        } => {
            let wasm = WasmMsg::Execute {
                contract_addr: contract_address,
                msg: to_json_binary(&msg)?,
                funds: vec![output_token_info.output_coin],
            };
            Response::new().add_message(wasm)
        }
        AfterSwapAction::IbcTransfer {
            receiver,
            channel,
            next_memo,
        } => {
            let next_memo = next_memo.unwrap_or_else(|| serde_json_wasm::from_str("{}").unwrap());
            let next_memo = insert_callback_key(next_memo.0, env);

            let memo = serde_json_wasm::to_string(&next_memo)
                .map_err(|_e| ContractError::InvalidMemo {})?;

            let ibc_transfer = MsgTransfer {
                source_port: TRANSFER_PORT.to_owned(),
                source_channel: channel.clone(),
                token: Some(output_token_info.output_coin.clone().into()),
                sender: env.contract.address.to_string(),
                receiver,
                timeout_height: None,
                timeout_timestamp: env.block.time.plus_seconds(IBC_PACKET_LIFITIME).nanos(),
                memo,
            };

            store_ibc_transfer_reply_state(
                deps.storage,
                &IbcTransferReplyState {
                    local_fallback_address: after_swap_info.local_fallback_address,
                    channel,
                    denom: output_token_info.output_coin.denom,
                    amount: output_token_info.output_coin.amount,
                },
            )?;

            Response::new().add_submessage(SubMsg::reply_on_success(
                ibc_transfer,
                MsgReplyId::IbcTransfer.repr(),
            ))
        }
    };

    Ok(response)
}

pub fn handle_multiswap(
    deps: DepsMut,
    env: &Env,
    mut swaps: Vec<MultiSwapMsg>,
    local_fallback_address: String,
) -> Result<Response, ContractError> {
    if swaps.is_empty() {
        return Err(ContractError::InvalidAmountOfSwaps {});
    }

    // store multi-swap information
    swaps.reverse();
    store_multi_swap_state(
        deps.storage,
        &MultiSwapState {
            swaps,
            local_fallback_address,
        },
    )?;

    Ok(Response::new().add_submessage(SubMsg::reply_always(
        WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_json_binary(&ExecuteMsg::ProcessMultiSwap {})?,
            funds: vec![],
        },
        MsgReplyId::MultiSwapFallback.repr(),
    )))
}

pub fn handle_multiswap_reply(
    deps: DepsMut,
    env: &Env,
    info: Option<&MessageInfo>,
) -> Result<Response, ContractError> {
    if let Some(info) = info {
        if info.sender.ne(&env.contract.address) {
            return Err(ContractError::Unauthorized {});
        }
    }

    // clean previous swap info
    remove_swap_reply_state(deps.storage);

    let mut multi_swaps = load_multi_swap_state(deps.storage)?;
    if multi_swaps.swaps.is_empty() {
        // all swaps are done, return ok
        return Ok(Response::new());
    }

    // store next swap info
    let next_swap = multi_swaps.swaps.pop().unwrap();
    store_swap_reply_state(
        deps.storage,
        &SwapReplyState {
            after_swap_action: next_swap.after_swap_action,
            local_fallback_address: multi_swaps.local_fallback_address.clone(),
        },
    )?;

    let swap_msg = WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_json_binary(&ExecuteMsg::ProcessSwap {
            swap_msg: next_swap.swap_msg,
        })?,
        funds: vec![next_swap.amount_in],
    };

    store_multi_swap_state(deps.storage, &multi_swaps)?;
    Ok(Response::new().add_submessage(SubMsg::reply_on_success(
        swap_msg,
        MsgReplyId::MultiSwap.repr(),
    )))
}

pub fn handle_swap_with_action_fallback_reply(
    deps: DepsMut,
    env: &Env,
    reply: Reply,
) -> Result<Response, ContractError> {
    let swap_state = load_swap_reply_state(deps.storage)?;
    remove_swap_reply_state(deps.storage);

    // in case of no error finish execution
    if reply.result.is_ok() {
        return Ok(Response::new().add_attribute("execution", "success"));
    }

    // otherwise transfer all owned funds to the fallback address
    recover_funds(deps, env, &swap_state.local_fallback_address)
}

pub fn handle_multiswap_fallback_reply(
    deps: DepsMut,
    env: &Env,
    reply: Reply,
) -> Result<Response, ContractError> {
    let multi_swaps = load_multi_swap_state(deps.storage)?;
    remove_multi_swap_state(deps.storage);

    // in case of no error finish execution
    if reply.result.is_ok() {
        return Ok(Response::new().add_attribute("execution", "success"));
    }

    // otherwise transfer all owned funds to the fallback address
    recover_funds(deps, env, &multi_swaps.local_fallback_address)
}

fn recover_funds(
    deps: DepsMut,
    env: &Env,
    fallback_address: &str,
) -> Result<Response, ContractError> {
    let owned_coins = deps
        .querier
        .query_all_balances(env.contract.address.to_string())?;

    if owned_coins.is_empty() {
        return Err(StdError::generic_err("Nothing to recover, forwarding error").into());
    }

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: fallback_address.to_owned(),
            amount: owned_coins,
        })
        .add_attribute("execution", "recovered"))
}
