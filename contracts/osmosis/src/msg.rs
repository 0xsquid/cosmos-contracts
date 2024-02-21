use enum_repr::EnumRepr;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Decimal};
use ibc_tracking::msg::IBCLifecycleComplete;
use osmosis_router::{OsmosisSimulateSwapResponse, OsmosisSwapMsg};
use osmosis_std::types::osmosis::poolmanager::v1beta1::SwapAmountInRoute;
use shared::SerializableJson;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    SwapWithAction {
        swap_msg: OsmosisSwapMsg,
        after_swap_action: AfterSwapAction,
        local_fallback_address: String,
    },
    MultiSwap {
        swaps: Vec<MultiSwapMsg>,
        local_fallback_address: String,
    },
    ProcessSwap {
        swap_msg: OsmosisSwapMsg,
    },
    ProcessMultiSwap {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(OsmosisSimulateSwapResponse)]
    EstimateTwapMinOutput {
        input_coin: cosmwasm_std::Coin,
        path: Vec<SwapAmountInRoute>,
        slippage: Decimal,
    },
}

#[cw_serde]
pub enum SudoMsg {
    #[serde(rename = "ibc_lifecycle_complete")]
    IBCLifecycleComplete(IBCLifecycleComplete),
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum AfterSwapAction {
    BankSend {
        receiver: String,
    },
    CustomCall {
        contract_address: String,
        msg: SerializableJson,
    },
    IbcTransfer {
        receiver: String,
        channel: String,
        next_memo: Option<SerializableJson>,
    },
}

#[cw_serde]
pub struct MultiSwapMsg {
    pub amount_in: Coin,
    pub swap_msg: OsmosisSwapMsg,
    pub after_swap_action: AfterSwapAction,
}

#[EnumRepr(type = "u64")]
pub enum MsgReplyId {
    Swap = 1,
    IbcTransfer = 2,
    MultiSwap = 3,
    SwapWithActionFallback = 4,
    MultiSwapFallback = 5,
}
