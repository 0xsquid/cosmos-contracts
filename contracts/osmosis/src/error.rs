use cosmwasm_std::StdError;
use osmosis_router::error::OsmosisRouterError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OsmosisRouterError(#[from] OsmosisRouterError),

    #[error("{0}")]
    PaymentError(#[from] cw_utils::PaymentError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid memo, serialization failed")]
    InvalidMemo {},

    #[error("Invalid reply id")]
    InvalidReplyId {},

    #[error("Ibc transfer failed: {msg:?}")]
    FailedIBCTransfer { msg: String },
}
