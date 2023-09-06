use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::Item;

use crate::{
    msg::{Call, CallAction},
    ContractError,
};

const MULTICALL_STATE: Item<MulticallState> = Item::new("multicall_state");

#[cw_serde]
pub struct MulticallState {
    current: u64,

    pub calls: Vec<Call>,
    pub fallback_address: Option<String>,
}

impl MulticallState {
    pub fn new(calls: &mut [Call], fallback_address: Option<String>) -> Self {
        calls.iter_mut().for_each(|call| call.actions.sort());

        Self {
            current: 0,
            calls: calls.to_owned(),
            fallback_address,
        }
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if self.calls.is_empty() {
            return Err(ContractError::EmptyCallsList {});
        }

        // if ibc tracking is required then fallback address must be set
        let has_ibc_tracking = self
            .calls
            .iter()
            .flat_map(|call| call.actions.clone())
            .any(|action| matches!(action, CallAction::IbcTracking {}));

        if has_ibc_tracking && self.fallback_address.is_none() {
            return Err(ContractError::FallbackAddressMustBeSetForIbcTracking {});
        }

        for call in self.calls.iter() {
            let ibc_tracking_count = call
                .actions
                .iter()
                .filter(|action| action == &&CallAction::IbcTracking {})
                .count();

            if ibc_tracking_count > 1 {
                return Err(ContractError::InvalidCallActionArgument {
                    msg: "Only one or none IbcTracking call action entries allowed per call"
                        .to_owned(),
                });
            }
        }

        Ok(())
    }

    pub fn next_call(&mut self) -> Option<&Call> {
        let next_call = self.calls.get(self.current as usize);
        self.current += 1;

        next_call
    }
}

pub fn multicall_state_exists(storage: &mut dyn Storage) -> StdResult<bool> {
    Ok(MULTICALL_STATE.may_load(storage)?.is_some())
}

pub fn store_multicall_state(storage: &mut dyn Storage, data: &MulticallState) -> StdResult<()> {
    MULTICALL_STATE.save(storage, data)
}

pub fn load_multicall_state(storage: &mut dyn Storage) -> StdResult<MulticallState> {
    MULTICALL_STATE.load(storage)
}

pub fn remove_multicall_state(storage: &mut dyn Storage) -> StdResult<()> {
    MULTICALL_STATE.remove(storage);
    Ok(())
}
