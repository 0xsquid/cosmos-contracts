#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
// use cw2::set_contract_version;

use crate::commands::{self};
use crate::error::ContractError;
use crate::ibc;
use crate::msg::{ExecuteMsg, IBCLifecycleComplete, InstantiateMsg, MsgReplyId, QueryMsg, SudoMsg};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:testme";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

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
        } => commands::swap(
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
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    match MsgReplyId::from_repr(reply.id) {
        Some(MsgReplyId::Swap) => commands::handle_after_swap_action(deps, &env, reply),
        Some(MsgReplyId::IbcTransfer) => commands::handle_ibc_transfer_reply(deps, reply),
        Some(MsgReplyId::MultiSwap) => commands::handle_multiswap_reply(deps, &env),
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
        }) => ibc::receive_ack(deps, channel, sequence, success),
        SudoMsg::IBCLifecycleComplete(IBCLifecycleComplete::IBCTimeout { channel, sequence }) => {
            ibc::receive_timeout(deps, channel, sequence)
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
        } => to_binary(
            &osmosis_router::router::estimate_min_twap_output(
                deps, &env, input_coin, path, slippage,
            )
            .unwrap(),
        ),
    }
}
