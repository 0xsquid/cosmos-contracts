use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::Item;

const PROCESSING_SWAP: Item<ProcessingSwap> = Item::new("processing_swap");

#[cw_serde]
pub struct ProcessingSwap {
    pub output_denom: String,
}

pub(crate) fn has_processing_swap(storage: &mut dyn Storage) -> StdResult<bool> {
    Ok(PROCESSING_SWAP.may_load(storage)?.is_some())
}

pub(crate) fn store_processing_swap(
    storage: &mut dyn Storage,
    data: &ProcessingSwap,
) -> StdResult<()> {
    PROCESSING_SWAP.save(storage, data)
}

pub(crate) fn load_processing_swap(storage: &mut dyn Storage) -> StdResult<ProcessingSwap> {
    let data = PROCESSING_SWAP.load(storage)?;
    PROCESSING_SWAP.remove(storage);

    Ok(data)
}
