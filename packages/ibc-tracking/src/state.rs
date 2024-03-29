use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};

const IBC_TRANSFER_REPLY_STATE: Item<IbcTransferReplyState> = Item::new("ibc_transfer_reply_state");
const AWAITING_IBC_TRANSFERS: Map<(&str, u64), IbcTransferReplyState> =
    Map::new("awaiting_ibc_transfers");

#[cw_serde]
pub struct IbcTransferReplyState {
    pub local_fallback_address: String,
    pub channel: String,
    pub denom: String,
    pub amount: Uint128,
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
