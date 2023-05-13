use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OsmosisRouterError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("Swap is already in process")]
    SwapIsAlreadyInProcess {},

    #[error("Invalid pool id: {id}")]
    InvalidPoolId { id: String },

    #[error("Pool coin not found: {denom}. {t}")]
    PoolCoinNotFound { denom: String, t: String },

    #[error("Invalid swap path")]
    InvalidPath {},

    #[error("Input denom {denom} not found for pool {pool_id}")]
    InputDenomNotFound { denom: String, pool_id: String },

    #[error("Twap price not found")]
    TwapPriceNotFound {},

    #[error("Invalid twap price")]
    InvalidTwapPrice {},

    #[error("Swap failed. Reason: {reason}")]
    FailedSwap { reason: String },
}
