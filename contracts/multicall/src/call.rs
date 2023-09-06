use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Env, QuerierWrapper, QueryRequest, Storage, SubMsg, Uint128,
    WasmQuery,
};
use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20QueryMsg};
use ibc_tracking::{
    msg::{CwIbcMessage, MsgTransfer},
    state::{store_ibc_transfer_reply_state, IbcTransferReplyState},
    util::insert_callback_key,
};
use shared::{util::json_pointer, SerializableJson};

use crate::{
    msg::{Call, CallAction, MsgReplyId, ProtoMessageType},
    ContractError,
};

impl Call {
    pub fn try_into_msg(
        &self,
        storage: &mut dyn Storage,
        querier: &QuerierWrapper<SerializableJson>,
        env: &Env,
        fallback_address: &Option<String>,
    ) -> Result<SubMsg<SerializableJson>, ContractError> {
        let mut cosmos_msg = self.msg.0.clone();
        let mut reply_id = MsgReplyId::ProcessCall.repr();

        for action in self.actions.iter() {
            match action {
                CallAction::NativeBalanceFetch { denom, replacer } => {
                    let balance = querier.query_balance(&env.contract.address, denom)?.amount;
                    if balance.is_zero() {
                        return Err(ContractError::ZeroBalanceFetched {
                            token: denom.to_owned(),
                        });
                    }

                    self.replace_value(&mut cosmos_msg, replacer, &balance.to_string())?;
                }
                CallAction::Cw20BalanceFetch { contract, replacer } => {
                    let balance = querier
                        .query(&QueryRequest::Wasm(WasmQuery::Smart {
                            contract_addr: contract.to_owned(),
                            msg: to_binary(&Cw20QueryMsg::Balance {
                                address: env.contract.address.to_string(),
                            })?,
                        }))
                        .unwrap_or_else(|_| Cw20BalanceResponse {
                            balance: Uint128::zero(),
                        })
                        .balance;

                    if balance.is_zero() {
                        return Err(ContractError::ZeroBalanceFetched {
                            token: contract.to_owned(),
                        });
                    }

                    self.replace_value(&mut cosmos_msg, replacer, &balance.to_string())?;
                }
                CallAction::CustomReplaceQuery {
                    query_msg,
                    replacers,
                } => {
                    let query_msg: QueryRequest<SerializableJson> = query_msg
                        .clone()
                        .0
                        .deserialize_into()
                        .map_err(|_| ContractError::SerializationError {})?;

                    let mut response: serde_cw_value::Value = querier.query(&query_msg)?;
                    for replacer in replacers.iter() {
                        let Some(query_field) = json_pointer(&mut response, &replacer.response_pointer) else {
                            return Err(ContractError::ReplacerFieldNotFound { replacer: replacer.response_pointer.to_owned() });
                        };

                        let Some(msg_field) = json_pointer(&mut cosmos_msg, &replacer.replacer) else {
                            return Err(ContractError::ReplacerFieldNotFound { replacer: replacer.replacer.to_owned() });
                        };

                        *msg_field = query_field.clone();
                    }
                }
                CallAction::IbcTracking {} => {
                    reply_id = MsgReplyId::IbcTransferTracking.repr();
                }
                CallAction::FieldToBinary { replacer } => {
                    let Some(binary_field) = json_pointer(&mut cosmos_msg, replacer) else {
                        return Err(ContractError::ReplacerFieldNotFound { replacer: replacer.to_owned() });
                    };

                    let binary = to_binary(&binary_field)?;
                    self.replace_value(&mut cosmos_msg, replacer, &binary.to_base64())?;
                }
                CallAction::FieldToProtoBinary {
                    replacer,
                    proto_msg_type,
                } => {
                    let Some(binary_field) = json_pointer(&mut cosmos_msg, replacer) else {
                        return Err(ContractError::ReplacerFieldNotFound { replacer: replacer.to_owned() });
                    };

                    let binary = match proto_msg_type {
                        ProtoMessageType::IbcTransfer => {
                            let mut ibc: CwIbcMessage = binary_field
                                .clone()
                                .deserialize_into()
                                .map_err(|_| ContractError::ProtoSerializationError {})?;

                            if reply_id == MsgReplyId::IbcTransferTracking.repr() {
                                // if ibc tracking is enabled store msg info
                                let memo = if ibc.memo.is_empty() {
                                    "{}".to_owned()
                                } else {
                                    ibc.memo.to_owned()
                                };

                                let memo: serde_cw_value::Value = serde_json_wasm::from_str(&memo)
                                    .map_err(|_| ContractError::SerializationError {})?;

                                let memo = insert_callback_key(memo, env);
                                ibc.memo = serde_json_wasm::to_string(&memo)
                                    .map_err(|_e| ContractError::InvalidMemo {})?;

                                let Some(local_fallback_address) = fallback_address.clone() else {
                                    return Err(ContractError::FallbackAddressMustBeSetForIbcTracking {});
                                };

                                store_ibc_transfer_reply_state(
                                    storage,
                                    &IbcTransferReplyState {
                                        local_fallback_address,
                                        channel: ibc.source_channel.clone(),
                                        denom: ibc
                                            .token
                                            .clone()
                                            .map(|t| t.denom)
                                            .unwrap_or_default(),
                                        amount: ibc
                                            .token
                                            .clone()
                                            .map(|t| t.amount)
                                            .unwrap_or_default(),
                                    },
                                )?;
                            }

                            let ibc: MsgTransfer = ibc.into();

                            let mut bytes = Vec::new();
                            prost::Message::encode(&ibc, &mut bytes)
                                .map_err(|_| ContractError::ProtoSerializationError {})?;

                            Binary(bytes)
                        }
                    };

                    self.replace_value(&mut cosmos_msg, replacer, &binary.to_base64())?;
                }
            }
        }

        Ok(SubMsg::reply_on_success(
            cosmos_msg
                .deserialize_into::<CosmosMsg<SerializableJson>>()
                .map_err(|_| ContractError::SerializationError {})?,
            reply_id,
        ))
    }

    fn replace_value(
        &self,
        cosmos_msg: &mut serde_cw_value::Value,
        replacer: &str,
        value: &str,
    ) -> Result<(), ContractError> {
        let Some(field) = json_pointer(cosmos_msg, replacer) else {
            return Err(ContractError::ReplacerFieldNotFound { replacer: replacer.to_owned() });
        };

        *field = serde_cw_value::Value::String(value.to_owned());
        Ok(())
    }
}
