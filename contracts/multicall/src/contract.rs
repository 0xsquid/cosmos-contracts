#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult};
use ibc_tracking::{ibc, msg::IBCLifecycleComplete};
use shared::SerializableJson;

use crate::{
    commands,
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, MsgReplyId, QueryMsg, SudoMsg},
    ContractError,
};

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
    deps: DepsMut<SerializableJson>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<SerializableJson>, ContractError> {
    match msg {
        ExecuteMsg::Multicall {
            calls,
            fallback_address,
        } => commands::handle_multicall(deps, &env, &calls, &fallback_address),
        ExecuteMsg::ProcessNextCall {} => commands::handle_call(deps, &env, &info),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    match MsgReplyId::from_repr(reply.id) {
        Some(MsgReplyId::ProcessCall) => commands::handle_call_reply(&env),
        Some(MsgReplyId::IbcTransferTracking) => {
            commands::handle_ibc_tracking_reply(deps, &env, reply)
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
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
