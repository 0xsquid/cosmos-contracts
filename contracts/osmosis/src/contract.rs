#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use ibc_tracking::msg::IBCLifecycleComplete;
use ibc_tracking::{ibc, reply as ibc_tracking_reply};

use crate::commands::{self};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, MsgReplyId, QueryMsg, SudoMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SwapWithAction {
            swap_msg,
            after_swap_action,
            local_fallback_address,
        } => commands::handle_swap_with_action(
            deps,
            &env,
            &info,
            swap_msg,
            after_swap_action,
            local_fallback_address,
        ),
        ExecuteMsg::MultiSwap {
            swaps,
            local_fallback_address,
        } => commands::handle_multiswap(deps, &env, swaps, local_fallback_address),
        ExecuteMsg::ProcessSwap { swap_msg } => {
            commands::handle_process_swap(deps, &env, &info, swap_msg)
        }
        ExecuteMsg::ProcessMultiSwap {} => {
            commands::handle_multiswap_reply(deps, &env, Some(&info))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    match MsgReplyId::from_repr(reply.id) {
        Some(MsgReplyId::Swap) => commands::handle_after_swap_action(deps, &env, reply),
        Some(MsgReplyId::IbcTransfer) => {
            ibc_tracking_reply::handle_ibc_transfer_reply(deps, reply).map_err(|e| e.into())
        }
        Some(MsgReplyId::MultiSwap) => commands::handle_multiswap_reply(deps, &env, None),
        Some(MsgReplyId::SwapWithActionFallback) => {
            commands::handle_swap_with_action_fallback_reply(deps, &env, reply)
        }
        Some(MsgReplyId::MultiSwapFallback) => {
            commands::handle_multiswap_fallback_reply(deps, &env, reply)
        }
        None => Err(ContractError::InvalidReplyId {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::IBCLifecycleComplete(IBCLifecycleComplete::IBCAck {
            channel,
            sequence,
            success,
            ..
        }) => ibc::receive_ack(deps, channel, sequence, success).map_err(|e| e.into()),
        SudoMsg::IBCLifecycleComplete(IBCLifecycleComplete::IBCTimeout { channel, sequence }) => {
            ibc::receive_timeout(deps, channel, sequence).map_err(|e| e.into())
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::EstimateTwapMinOutput {
            input_coin,
            path,
            slippage,
        } => to_json_binary(
            &osmosis_router::router::estimate_min_twap_output(
                deps, &env, input_coin, path, slippage,
            )
            .unwrap(),
        ),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
