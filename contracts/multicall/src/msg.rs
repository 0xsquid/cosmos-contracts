use cosmwasm_schema::{cw_serde, QueryResponses};
use enum_repr::EnumRepr;
use ibc_tracking::msg::IBCLifecycleComplete;
use shared::SerializableJson;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Multicall {
        calls: Vec<Call>,
        fallback_address: Option<String>,
    },
    ProcessNextCall {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}

#[cw_serde]
pub enum SudoMsg {
    #[serde(rename = "ibc_lifecycle_complete")]
    IBCLifecycleComplete(IBCLifecycleComplete),
}

#[cw_serde]
pub struct MigrateMsg {}

#[EnumRepr(type = "u64")]
pub enum MsgReplyId {
    ProcessCall = 1,
    IbcTransferTracking = 2,
}

#[cw_serde]
pub struct Call {
    pub msg: SerializableJson,
    pub actions: Vec<CallAction>,
}

#[cw_serde]
#[derive(Eq, PartialOrd, Ord)]
pub enum CallAction {
    NativeBalanceFetch {
        denom: String,
        replacer: String,
    },
    Cw20BalanceFetch {
        contract: String,
        replacer: String,
    },
    CustomReplaceQuery {
        query_msg: SerializableJson,
        replacers: Vec<ReplaceInfo>,
    },
    IbcTracking {},
    FieldToBinary {
        replacer: String,
    },
    FieldToProtoBinary {
        replacer: String,
        proto_msg_type: ProtoMessageType,
    },
}

#[cw_serde]
#[derive(Eq, PartialOrd, Ord)]
pub struct ReplaceInfo {
    pub response_pointer: String,
    pub replacer: String,
}

#[cw_serde]
#[derive(Eq, PartialOrd, Ord)]
pub enum ProtoMessageType {
    IbcTransfer,
}
