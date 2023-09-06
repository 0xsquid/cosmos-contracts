use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IbcTrackingError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Ibc transfer failed: {msg:?}")]
    FailedIBCTransfer { msg: String },
}
