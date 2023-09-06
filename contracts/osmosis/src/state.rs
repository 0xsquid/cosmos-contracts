use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdResult, Storage};
use cw_storage_plus::Item;

use crate::msg::{AfterSwapAction, MultiSwapMsg};

const SWAP_REPLY_STATE: Item<SwapReplyState> = Item::new("swap_reply_state");
const MULTI_SWAP_STATE: Item<MultiSwapState> = Item::new("multi_swap_state");

#[cw_serde]
pub struct SwapReplyState {
    pub after_swap_action: AfterSwapAction,
    pub local_fallback_address: String,
}

#[cw_serde]
pub struct MultiSwapState {
    pub swaps: Vec<MultiSwapMsg>,
    pub local_fallback_address: String,
}

pub fn swap_reply_state_exists(storage: &dyn Storage) -> StdResult<bool> {
    Ok(SWAP_REPLY_STATE.may_load(storage)?.is_some())
}

pub fn store_swap_reply_state(storage: &mut dyn Storage, data: &SwapReplyState) -> StdResult<()> {
    SWAP_REPLY_STATE.save(storage, data)
}

pub fn load_swap_reply_state(storage: &mut dyn Storage) -> StdResult<SwapReplyState> {
    let data = SWAP_REPLY_STATE.load(storage)?;
    SWAP_REPLY_STATE.remove(storage);

    Ok(data)
}

pub fn store_multi_swap_state(storage: &mut dyn Storage, data: &MultiSwapState) -> StdResult<()> {
    MULTI_SWAP_STATE.save(storage, data)
}

pub fn load_multi_swap_state(storage: &dyn Storage) -> StdResult<MultiSwapState> {
    MULTI_SWAP_STATE.load(storage)
}

pub fn remove_multi_swap_state(storage: &mut dyn Storage) {
    MULTI_SWAP_STATE.remove(storage);
}
