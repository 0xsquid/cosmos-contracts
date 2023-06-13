use cosmwasm_std::{BankMsg, Coin, DepsMut, Response};

use crate::{state::load_awaiting_ibc_transfer_optional, ContractError};

pub fn receive_ack(
    deps: DepsMut,
    source_channel: String,
    sequence: u64,
    success: bool,
) -> Result<Response, ContractError> {
    if success {
        // need to load awaiting transfer in order to remove it
        let _ = load_awaiting_ibc_transfer_optional(deps.storage, &source_channel, sequence)?;
        return Ok(Response::new());
    }

    send_funds_to_fallback_address(deps, source_channel, sequence)
}

pub fn receive_timeout(
    deps: DepsMut,
    source_channel: String,
    sequence: u64,
) -> Result<Response, ContractError> {
    send_funds_to_fallback_address(deps, source_channel, sequence)
}

fn send_funds_to_fallback_address(
    deps: DepsMut,
    source_channel: String,
    sequence: u64,
) -> Result<Response, ContractError> {
    let Some(ibc_transfer_info) =
        load_awaiting_ibc_transfer_optional(deps.storage, &source_channel, sequence)? else {
            return Ok(Response::new());
        };

    Ok(Response::new().add_message(BankMsg::Send {
        to_address: ibc_transfer_info.local_fallback_address,
        amount: vec![Coin {
            denom: ibc_transfer_info.denom,
            amount: ibc_transfer_info.amount,
        }],
    }))
}
