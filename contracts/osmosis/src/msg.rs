use enum_repr::EnumRepr;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Decimal};
use osmosis_router::{OsmosisSimulateSwapResponse, OsmosisSwapMsg};
use osmosis_std::types::osmosis::gamm::v1beta1::SwapAmountInRoute;
use osmosis_std_derive::CosmwasmExt;
use schemars::JsonSchema;

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
}

#[cw_serde]
pub enum IBCLifecycleComplete {
    #[serde(rename = "ibc_ack")]
    IBCAck {
        channel: String,
        sequence: u64,
        ack: String,
        success: bool,
    },
    #[serde(rename = "ibc_timeout")]
    IBCTimeout { channel: String, sequence: u64 },
}

#[derive(
    ::cosmwasm_schema::serde::Serialize,
    ::cosmwasm_schema::serde::Deserialize,
    ::std::clone::Clone,
    ::std::fmt::Debug,
    PartialEq,
    Eq,
)]
pub struct SerializableJson(pub serde_cw_value::Value);

impl JsonSchema for SerializableJson {
    fn schema_name() -> String {
        "JSON".to_string()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        schemars::schema::Schema::from(true)
    }
}

impl SerializableJson {
    pub fn as_value(&self) -> &serde_cw_value::Value {
        &self.0
    }
}

impl From<serde_cw_value::Value> for SerializableJson {
    fn from(value: serde_cw_value::Value) -> Self {
        Self(value)
    }
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/ibc.applications.transfer.v1.MsgTransfer")]
pub struct MsgTransfer {
    #[prost(string, tag = "1")]
    pub source_port: String,
    #[prost(string, tag = "2")]
    pub source_channel: String,
    #[prost(message, optional, tag = "3")]
    pub token: ::core::option::Option<osmosis_std::types::cosmos::base::v1beta1::Coin>,
    #[prost(string, tag = "4")]
    pub sender: String,
    #[prost(string, tag = "5")]
    pub receiver: String,
    #[prost(message, optional, tag = "6")]
    pub timeout_height: Option<IbcCounterpartyHeight>,
    #[prost(uint64, optional, tag = "7")]
    pub timeout_timestamp: ::core::option::Option<u64>,
    #[prost(string, tag = "8")]
    pub memo: String,
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
pub struct IbcCounterpartyHeight {
    #[prost(uint64, optional, tag = "1")]
    revision_number: Option<u64>,
    #[prost(uint64, optional, tag = "2")]
    revision_height: Option<u64>,
}

#[derive(Clone, PartialEq, Eq, ::prost::Message)]
pub struct MsgTransferResponse {
    #[prost(uint64, tag = "1")]
    pub sequence: u64,
}
