use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use enum_repr::EnumRepr;
use ibc_tracking::msg::IBCLifecycleComplete;
use shared::SerializableJson;

/// ## InstantiateMsg
/// This structure describes the basic settings for creating a contract.
#[cw_serde]
pub struct InstantiateMsg {}

/// ## ExecuteMsg
/// This structure describes the execute messages of the contract.
#[cw_serde]
pub enum ExecuteMsg {
    /// ## Description
    /// Executes a set of cosmos messages specified in the calls array
    Multicall {
        /// onchain calls to perform
        calls: Vec<Call>,
        /// fallback address for failed/timeout rejected ibc transfers
        fallback_address: Option<String>,
    },
    /// ## Description
    /// Internal action, can be called only by the contract itself
    ProcessNextCall {},
}

/// ## QueryMsg
/// This structure describes the query messages of the contract
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}

/// ## SudoMsg
/// This structure describes the sudo messages of the contract
#[cw_serde]
pub enum SudoMsg {
    /// ## Description
    /// Ibc tracking sudo callback
    #[serde(rename = "ibc_lifecycle_complete")]
    IBCLifecycleComplete(IBCLifecycleComplete),
}

/// ## MigrateMsg
/// This structure describes a migration message.
#[cw_serde]
pub struct MigrateMsg {}

/// ## MsgReplyId
/// This structure describes reply callback keys for the multicall contract
#[EnumRepr(type = "u64")]
pub enum MsgReplyId {
    /// Callback from the current call leading to the next call
    ProcessCall = 1,
    /// Callback for enabling ibc tracking
    IbcTransferTracking = 2,
}

/// ## Call
/// This structure describes the fields for call object structure
#[cw_serde]
pub struct Call {
    /// valid json message of type [`cosmwasm_std::CosmosMsg`]
    pub msg: SerializableJson,
    /// a set of actions to perform before executing cosmos message
    pub actions: Vec<CallAction>,
}

/// ## CallAction
/// This structure describes the fields for call action object structure
#[cw_serde]
#[derive(Eq, PartialOrd, Ord)]
pub enum CallAction {
    /// ## Description
    /// Queries bank module contract's balance and replaces received value in the message
    NativeBalanceFetch {
        /// coin denom to query
        denom: String,
        /// path to a field in the message for replacement
        replacer: String,
    },
    /// ## Description
    /// Queries cw20 token contract's balance and replaces received value in the message
    Cw20BalanceFetch {
        /// cw20 contract address
        contract: String,
        /// path to a field in the message for replacement
        replacer: String,
    },
    /// ## Description
    /// Makes a custom query and replaces msg values using data from the query response
    /// Both [`CallAction::NativeBalanceFetch`] & [`CallAction::Cw20BalanceFetch`] can be done via this call action type
    CustomReplaceQuery {
        /// valid json message of type [`cosmwasm_std::QueryRequest`]
        query_msg: SerializableJson,
        /// list of replacer paths
        replacers: Vec<ReplaceInfo>,
    },
    /// ## Description
    /// Enables ibc tracking for sent ibc transfer messages from the multicall contract
    IbcTracking {
        /// ibc channel
        channel: String,
        /// send denom
        denom: String,
        /// send amount, either amount or replacer must be set
        amount: Option<Uint128>,
        /// path to amount field in the message for replacement
        amount_pointer: Option<String>,
    },
    /// ## Description
    /// Converts specified field into [`Binary`] type
    FieldToBinary {
        /// path to a field in the message for replacement
        replacer: String,
    },
    /// ## Description
    /// Converts specified field into [`Binary`] type encoded using [`prost::Message::encode`] method
    FieldToProtoBinary {
        /// path to a field in the message for replacement
        replacer: String,
        /// Protobuf message type
        proto_msg_type: ProtoMessageType,
    },
}

/// ## ReplaceInfo
/// This structure describes the fields for replacer info object structure
#[cw_serde]
#[derive(Eq, PartialOrd, Ord)]
pub struct ReplaceInfo {
    /// path to a field in the query response struct to retrieve
    pub response_pointer: String,
    /// path to a field in the message for replacement
    pub replacer: String,
}

/// ## ProtoMessageType
/// This structure describes the fields for protobuf message type object structure
#[cw_serde]
#[derive(Eq, PartialOrd, Ord)]
pub enum ProtoMessageType {
    /// ibc message type
    IbcTransfer,
}
