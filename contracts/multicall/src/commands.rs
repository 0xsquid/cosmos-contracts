use cosmwasm_std::{to_binary, DepsMut, Env, MessageInfo, Reply, Response, WasmMsg};
use ibc_tracking::reply::handle_ibc_transfer_reply;
use shared::SerializableJson;

use crate::{
    msg::{Call, ExecuteMsg},
    state::{
        load_multicall_state, multicall_state_exists, remove_multicall_state,
        store_multicall_state, MulticallState,
    },
    ContractError,
};

pub fn handle_multicall(
    deps: DepsMut<SerializableJson>,
    env: &Env,
    calls: &[Call],
    fallback_address: &Option<String>,
) -> Result<Response<SerializableJson>, ContractError> {
    if multicall_state_exists(deps.storage)? {
        return Err(ContractError::ContractLocked {
            msg: "Another multicall action is in progress".to_owned(),
        });
    }

    let state = MulticallState::new(calls.to_owned().as_mut(), fallback_address.clone());
    state.validate()?;

    store_multicall_state(deps.storage, &state)?;
    Ok(Response::new().add_message(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::ProcessNextCall {})?,
        funds: vec![],
    }))
}

pub fn handle_call(
    deps: DepsMut<SerializableJson>,
    env: &Env,
    info: &MessageInfo,
) -> Result<Response<SerializableJson>, ContractError> {
    // internal action, can be called only by the contract itself
    if info.sender.ne(&env.contract.address) {
        return Err(ContractError::CanBeCalledOnlyByContractItself {});
    }

    let mut state = load_multicall_state(deps.storage)?;
    let fallback_address = state.fallback_address.clone();

    let Some(call) = state.next_call() else {
        // if there is no calls left then finish the execution here
        remove_multicall_state(deps.storage)?;
        return Ok(Response::default());
    };

    let submsg = call.try_into_msg(deps.storage, &deps.querier, env, &fallback_address)?;

    store_multicall_state(deps.storage, &state)?;
    Ok(Response::new().add_submessage(submsg))
}

pub fn handle_call_reply(env: &Env) -> Result<Response, ContractError> {
    // proceed to the next call here
    Ok(Response::new().add_message(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::ProcessNextCall {})?,
        funds: vec![],
    }))
}

pub fn handle_ibc_tracking_reply(
    deps: DepsMut,
    env: &Env,
    reply: Reply,
) -> Result<Response, ContractError> {
    // register ibc tracking info
    let _ = handle_ibc_transfer_reply(deps, reply)?;

    handle_call_reply(env)
}
