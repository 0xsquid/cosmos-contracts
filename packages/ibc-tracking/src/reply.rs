use ::prost::Message;
use cosmwasm_std::{DepsMut, Reply, Response, SubMsgResponse, SubMsgResult};

use crate::{
    error::IbcTrackingError,
    msg::MsgTransferResponse,
    state::{load_ibc_transfer_reply_state, store_awaiting_ibc_transfer},
};

pub fn handle_ibc_transfer_reply(
    deps: DepsMut,
    reply: Reply,
) -> Result<Response, IbcTrackingError> {
    let SubMsgResult::Ok(SubMsgResponse { data: Some(b), .. }) = reply.result else {
        return Err(IbcTrackingError::FailedIBCTransfer { msg: format!("Reply failed: {:?}", reply.result) })
    };

    let ibc_transfer_response =
        MsgTransferResponse::decode(&b[..]).map_err(|_e| IbcTrackingError::FailedIBCTransfer {
            msg: format!("Failed to decode ibc transfer response: {b}"),
        })?;

    let ibc_transfer_info = load_ibc_transfer_reply_state(deps.storage)?;
    store_awaiting_ibc_transfer(
        deps.storage,
        ibc_transfer_response.sequence,
        &ibc_transfer_info,
    )?;

    Ok(Response::new())
}
