use cosmwasm_schema::cw_serde;
use osmosis_std_derive::CosmwasmExt;

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

#[cw_serde]
pub struct CwIbcMessage {
    pub source_port: String,
    pub source_channel: String,
    pub token: Option<cosmwasm_std::Coin>,
    pub sender: String,
    pub receiver: String,
    pub timeout_height: Option<IbcCounterpartyHeight>,
    pub timeout_timestamp: Option<u64>,
    pub memo: String,
}

impl From<CwIbcMessage> for MsgTransfer {
    fn from(value: CwIbcMessage) -> Self {
        Self {
            source_port: value.source_port,
            source_channel: value.source_channel,
            token: value.token.map(|v| v.into()),
            sender: value.sender,
            receiver: value.receiver,
            timeout_height: value.timeout_height,
            timeout_timestamp: value.timeout_timestamp,
            memo: value.memo,
        }
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
