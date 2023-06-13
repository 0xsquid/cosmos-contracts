use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};

use crate::msg::{AfterSwapAction, MultiSwapMsg};

const SWAP_REPLY_STATE: Item<SwapReplyState> = Item::new("swap_reply_state");
const IBC_TRANSFER_REPLY_STATE: Item<IbcTransferReplyState> = Item::new("ibc_transfer_reply_state");
const AWAITING_IBC_TRANSFERS: Map<(&str, u64), IbcTransferReplyState> =
    Map::new("awaiting_ibc_transfers");

const MULTI_SWAP_STATE: Item<MultiSwapState> = Item::new("multi_swap_state");

#[cw_serde]
pub struct SwapReplyState {
    pub after_swap_action: AfterSwapAction,
    pub local_fallback_address: String,
}

#[cw_serde]
pub struct IbcTransferReplyState {
    pub local_fallback_address: String,
    pub channel: String,
    pub denom: String,
    pub amount: Uint128,
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

pub fn store_ibc_transfer_reply_state(
    storage: &mut dyn Storage,
    data: &IbcTransferReplyState,
) -> StdResult<()> {
    IBC_TRANSFER_REPLY_STATE.save(storage, data)
}

pub fn load_ibc_transfer_reply_state(
    storage: &mut dyn Storage,
) -> StdResult<IbcTransferReplyState> {
    let data = IBC_TRANSFER_REPLY_STATE.load(storage)?;
    IBC_TRANSFER_REPLY_STATE.remove(storage);

    Ok(data)
}

pub fn store_awaiting_ibc_transfer(
    storage: &mut dyn Storage,
    sequence: u64,
    data: &IbcTransferReplyState,
) -> StdResult<()> {
    AWAITING_IBC_TRANSFERS.save(storage, (&data.channel, sequence), data)
}

pub fn load_awaiting_ibc_transfer_optional(
    storage: &mut dyn Storage,
    channel: &str,
    sequence: u64,
) -> StdResult<Option<IbcTransferReplyState>> {
    let data = AWAITING_IBC_TRANSFERS.may_load(storage, (channel, sequence))?;
    AWAITING_IBC_TRANSFERS.remove(storage, (channel, sequence));

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
