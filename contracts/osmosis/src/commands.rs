use ::prost::Message;

use cosmwasm_std::{
    to_binary, BankMsg, DepsMut, Env, MessageInfo, Reply, Response, SubMsg, SubMsgResponse,
    SubMsgResult, WasmMsg,
};
use cw_utils::one_coin;
use osmosis_router::{
    router::{build_swap_msg, get_swap_amount_out_response},
    OsmosisSwapMsg,
};

use crate::{
    msg::{AfterSwapAction, MsgReplyId, MsgTransfer, MsgTransferResponse},
    state::{
        load_ibc_transfer_reply_state, load_swap_reply_state, store_awaiting_ibc_transfer,
        store_ibc_transfer_reply_state, store_swap_reply_state, IbcTransferReplyState,
        SwapReplyState,
    },
    ContractError,
};

const TRANSFER_PORT: &'static str = "transfer";
const IBC_PACKET_LIFITIME: u64 = 604_800u64;

pub fn swap(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    swap_msg: OsmosisSwapMsg,
    after_swap_action: AfterSwapAction,
    local_fallback_address: String,
) -> Result<Response, ContractError> {
    let input_coin = one_coin(info)?;
    let swap_msg = build_swap_msg(deps.storage, env, input_coin, swap_msg)?;

    store_swap_reply_state(
        deps.storage,
        &SwapReplyState {
            after_swap_action,
            local_fallback_address,
        },
    )?;

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
                msg: to_binary(&msg)?,
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
            let memo = serde_json_wasm::to_string(&next_memo)
                .map_err(|_e| ContractError::InvalidMemo {})?;

            let ibc_transfer = MsgTransfer {
                source_port: TRANSFER_PORT.to_owned(),
                source_channel: channel.clone(),
                token: Some(output_token_info.output_coin.clone().into()),
                sender: env.contract.address.to_string(),
                receiver,
                timeout_height: None,
                timeout_timestamp: Some(env.block.time.plus_seconds(IBC_PACKET_LIFITIME).nanos()),
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

pub fn handle_ibc_transfer_reply(deps: DepsMut, reply: Reply) -> Result<Response, ContractError> {
    let SubMsgResult::Ok(SubMsgResponse { data: Some(b), .. }) = reply.result else {
        return Err(ContractError::FailedIBCTransfer { msg: format!("failed reply: {:?}", reply.result) })
    };

    let ibc_transfer_response =
        MsgTransferResponse::decode(&b[..]).map_err(|_e| ContractError::FailedIBCTransfer {
            msg: format!("could not decode response: {b}"),
        })?;

    let ibc_transfer_info = load_ibc_transfer_reply_state(deps.storage)?;
    store_awaiting_ibc_transfer(
        deps.storage,
        ibc_transfer_response.sequence,
        &ibc_transfer_info,
    )?;

    Ok(Response::new())
}
