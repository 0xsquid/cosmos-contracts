use cosmwasm_schema::cw_serde;
use osmosis_std::types::ibc::{applications::transfer::v1::MsgTransfer, core::client::v1::Height};

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
    pub timeout_height: Option<CwHeight>,
    pub timeout_timestamp: u64,
    pub memo: String,
}

#[cw_serde]
pub struct CwHeight {
    pub revision_number: u64,
    pub revision_height: u64,
}

impl From<CwIbcMessage> for MsgTransfer {
    fn from(value: CwIbcMessage) -> Self {
        Self {
            source_port: value.source_port,
            source_channel: value.source_channel,
            token: value.token.map(|v| v.into()),
            sender: value.sender,
            receiver: value.receiver,
            timeout_height: value.timeout_height.map(|h| Height {
                revision_number: h.revision_number,
                revision_height: h.revision_height,
            }),
            timeout_timestamp: value.timeout_timestamp,
            memo: value.memo,
        }
    }
}

#[derive(Clone, PartialEq, Eq, ::prost::Message)]
pub struct MsgTransferResponse {
    #[prost(uint64, tag = "1")]
    pub sequence: u64,
}
