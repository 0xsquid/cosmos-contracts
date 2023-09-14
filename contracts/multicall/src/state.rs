use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::Item;

use crate::{
    msg::{Call, CallAction},
    ContractError,
};

/// ## Description
/// Stores multicall information of type [`MulticallState`] at the given key.
/// Value is set at the beggining of the tx and dropped at the end of execution.
const MULTICALL_STATE: Item<MulticallState> = Item::new("multicall_state");

/// ## Description
/// This structure describes the provided calls for execution
#[cw_serde]
pub struct MulticallState {
    /// calls array current index
    current: u64,

    /// onchain calls to perform
    pub calls: Vec<Call>,
    /// fallback address for failed/timeout rejected ibc transfers
    pub fallback_address: Option<String>,
}

impl MulticallState {
    /// ## Description
    /// Creates new instance of [`MulticallState`] struct
    pub fn new(
        calls: &mut [Call],
        fallback_address: Option<String>,
    ) -> Result<Self, ContractError> {
        calls.iter_mut().for_each(|call| call.actions.sort());

        let state = Self {
            current: 0,
            calls: calls.to_owned(),
            fallback_address,
        };

        state.validate()?;
        Ok(state)
    }

    /// ## Description
    /// Returns next call from the calls sequence. If no more calls left [`None`] will be returned.
    pub fn next_call(&mut self) -> Option<&Call> {
        let next_call = self.calls.get(self.current as usize);
        self.current += 1;

        next_call
    }

    /// ## Description
    /// Validates provided calls
    fn validate(&self) -> Result<(), ContractError> {
        if self.calls.is_empty() {
            return Err(ContractError::EmptyCallsList {});
        }

        // if ibc tracking is required then fallback address must be set
        let has_ibc_tracking = self
            .calls
            .iter()
            .flat_map(|call| call.actions.clone())
            .any(|action| matches!(action, CallAction::IbcTracking { .. }));

        if has_ibc_tracking && self.fallback_address.is_none() {
            return Err(ContractError::FallbackAddressMustBeSetForIbcTracking {});
        }

        for call in self.calls.iter() {
            let ibc_tracking_count = call
                .actions
                .iter()
                .filter(|action| matches!(action, &&CallAction::IbcTracking { .. }))
                .count();

            if ibc_tracking_count > 1 {
                return Err(ContractError::InvalidCallActionArgument {
                    msg: "Only one or none IbcTracking call action entries allowed per call"
                        .to_owned(),
                });
            }

            // validate replacer strings to have valid replacer string format
            for call_action in call.actions.iter() {
                match call_action {
                    CallAction::NativeBalanceFetch { replacer, .. } => {
                        self.validate_replacer(replacer)?;
                    }
                    CallAction::Cw20BalanceFetch { replacer, .. } => {
                        self.validate_replacer(replacer)?;
                    }
                    CallAction::CustomReplaceQuery { replacers, .. } => {
                        for replacer_info in replacers.iter() {
                            self.validate_replacer(&replacer_info.response_pointer)?;
                            self.validate_replacer(&replacer_info.replacer)?;
                        }
                    }
                    CallAction::IbcTracking {
                        amount,
                        amount_pointer,
                        ..
                    } => {
                        if amount.is_none() && amount_pointer.is_none() {
                            return Err(ContractError::EitherAmountOfPointerMustBeSet {});
                        }

                        if let Some(pointer) = amount_pointer {
                            self.validate_replacer(pointer)?;
                        }
                    }
                    CallAction::FieldToBinary { replacer } => {
                        self.validate_replacer(replacer)?;
                    }
                    CallAction::FieldToProtoBinary { replacer, .. } => {
                        self.validate_replacer(replacer)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_replacer(&self, replacer: &str) -> Result<(), ContractError> {
        if replacer.is_empty() || !replacer.starts_with('/') {
            return Err(ContractError::InvalidReplacer {});
        }

        Ok(())
    }
}

/// ## Description
/// Checks whether state exists or not
/// ## Params
/// * **storage** is an object of type [`Storage`]
pub fn multicall_state_exists(storage: &mut dyn Storage) -> StdResult<bool> {
    Ok(MULTICALL_STATE.may_load(storage)?.is_some())
}

/// ## Description
/// Saves changes of [`MulticallState`] struct in [`MULTICALL_STATE`] storage
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **data** updated config struct of type [`MulticallState`]
pub fn store_multicall_state(storage: &mut dyn Storage, data: &MulticallState) -> StdResult<()> {
    MULTICALL_STATE.save(storage, data)
}

/// ## Description
/// Returns state struct of type [`MulticallState`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
pub fn load_multicall_state(storage: &mut dyn Storage) -> StdResult<MulticallState> {
    MULTICALL_STATE.load(storage)
}

/// ## Description
/// Removes [`MulticallState`] information from [`MULTICALL_STATE`] storage
/// ## Params
/// * **storage** is an object of type [`Storage`]
pub fn remove_multicall_state(storage: &mut dyn Storage) -> StdResult<()> {
    MULTICALL_STATE.remove(storage);
    Ok(())
}
