#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;
use ibc_tracking::{ibc, msg::IBCLifecycleComplete};
use shared::SerializableJson;

use crate::{
    commands,
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, MsgReplyId, QueryMsg, SudoMsg},
    queries, ContractError,
};

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "squid-multicall";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// ## Description
/// Creates a new contract with the specified parameters in the [`InstantiateMsg`].
/// Returns the default [`Response`] object if the operation was successful, otherwise returns
/// the [`ContractError`] if the contract was not created.
/// ## Params
/// * **_deps** is an object of type [`DepsMut`].
///
/// * **_env** is an object of type [`Env`].
///
/// * **_info** is an object of type [`MessageInfo`].
///
/// * **_msg** is a message of type [`InstantiateMsg`] which contains the basic settings for creating a contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}

/// ## Description
/// Available execute messages of the contract
/// ## Params
/// * **deps** is an object of type [`Deps<SerializableJson>`].
///
/// * **env** is an object of type [`Env`].
///
/// * **info** is an object of type [`MessageInfo`].
///
/// * **msg** is an object of type [`ExecuteMsg`].
///
/// ## Messages
///
/// * **ExecuteMsg::Multicall {
///         calls,
///         fallback_address,
///     }** Executes a set of cosmos messages specified in the calls array
///
/// * **ExecuteMsg::ProcessNextCall {}** Internal action, can be called only by the contract itself
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

/// ## Description
/// Handles callbacks returned to the contract
/// ## Params
/// * **deps** is an object of type [`Deps`].
///
/// * **env** is an object of type [`Env`].
///
/// * **reply** is an object of type [`Reply`]
///
/// ## Reply id's
///
/// * **MsgReplyId::ProcessCall** Callback from the current call leading to the next call
///
/// * **MsgReplyId::IbcTransferTracking** Callback for enabling ibc tracking
///
/// * **MsgReplyId::ExecutionFallback** Callback for catching execution error and attempting to recover funds locally
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    match MsgReplyId::from_repr(reply.id) {
        Some(MsgReplyId::ProcessCall) => commands::handle_call_reply(&env),
        Some(MsgReplyId::IbcTransferTracking) => {
            commands::handle_ibc_tracking_reply(deps, &env, reply)
        }
        Some(MsgReplyId::ExecutionFallback) => {
            commands::handle_execution_fallback_reply(deps, &env, reply)
        }
        None => Err(ContractError::InvalidReplyId {}),
    }
}

/// ## Description
/// Handles sudo message reply
/// ## Params
/// * **deps** is an object of type [`Deps`].
///
/// * **env** is an object of type [`Env`].
///
/// * **msg** is an object of type [`SudoMsg`]
///
/// ## Messages
///
/// * **SudoMsg::IBCLifecycleComplete**
///
/// * **SudoMsg::IBCLifecycleComplete**
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

/// ## Description
/// Available query messages of the contract
/// ## Params
/// * **_deps** is an object of type [`Deps`].
///
/// * **_env** is an object of type [`Env`].
///
/// * **_msg** is an object of type [`ExecuteMsg`].
///
/// * **QueryMsg::MultiQuery {}** Sequentially executes provided queries and returns corresponding responses
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<SerializableJson>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::MultiQuery { queries } => to_json_binary(&queries::multi_query(deps, queries)?),
    }
}

/// ## Description
/// Used for migration of contract. Returns the default object of type [`Response`].
/// ## Params
/// * **_deps** is an object of type [`Deps`].
///
/// * **_env** is an object of type [`Env`].
///
/// * **_msg** is an object of type [`MigrateMsg`].
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attributes(vec![
        ("new_contract_name", CONTRACT_NAME),
        ("new_contract_version", CONTRACT_VERSION),
    ]))
}
