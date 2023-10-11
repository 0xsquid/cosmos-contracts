use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Env, QuerierWrapper, QueryRequest, Storage, SubMsg, Uint128,
    WasmQuery,
};
use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20QueryMsg};
use ibc_tracking::{
    msg::{CwIbcMessage, MsgTransfer},
    state::{store_ibc_transfer_reply_state, IbcTransferReplyState},
};
use osmosis_std::types::osmosis::gamm::v1beta1::MsgSwapExactAmountIn;
use shared::{util::json_pointer, SerializableJson};
use std::str::FromStr;

use crate::{
    msg::{Call, CallAction, MsgReplyId, ProtoMessageType},
    ContractError,
};

impl Call {
    /// ## Description
    /// Converts [`Call`] struct into valid [`SubMsg`] object that will be sent to the node.
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
                        let query_field = json_pointer(&mut response, &replacer.response_pointer)
                            .ok_or(ContractError::ReplacerFieldNotFound {
                            replacer: replacer.response_pointer.to_owned(),
                        })?;

                        let msg_field = json_pointer(&mut cosmos_msg, &replacer.replacer).ok_or(
                            ContractError::ReplacerFieldNotFound {
                                replacer: replacer.replacer.to_owned(),
                            },
                        )?;

                        *msg_field = query_field.clone();
                    }
                }
                CallAction::IbcTracking {
                    channel,
                    denom,
                    amount,
                    amount_pointer,
                } => {
                    reply_id = MsgReplyId::IbcTransferTracking.repr();

                    let Some(local_fallback_address) = fallback_address.clone() else {
                        return Err(ContractError::FallbackAddressMustBeSetForIbcTracking {});
                    };

                    let amount = if let Some(pointer) = amount_pointer {
                        let amount_field = json_pointer(&mut cosmos_msg, pointer).ok_or(
                            ContractError::ReplacerFieldNotFound {
                                replacer: pointer.to_owned(),
                            },
                        )?;

                        let serde_cw_value::Value::String(amount) = amount_field else {
                            return Err(ContractError::InvalidAmountPointer {  });
                        };

                        Uint128::from_str(amount)?
                    } else {
                        amount.ok_or(ContractError::EitherAmountOfPointerMustBeSet {})?
                    };

                    store_ibc_transfer_reply_state(
                        storage,
                        &IbcTransferReplyState {
                            local_fallback_address,
                            channel: channel.clone(),
                            denom: denom.clone(),
                            amount,
                        },
                    )?;
                }
                CallAction::FieldToBinary { replacer } => {
                    let binary_field = json_pointer(&mut cosmos_msg, replacer).ok_or(
                        ContractError::ReplacerFieldNotFound {
                            replacer: replacer.to_owned(),
                        },
                    )?;

                    let binary = to_binary(&binary_field)?;
                    self.replace_value(&mut cosmos_msg, replacer, &binary.to_base64())?;
                }
                CallAction::FieldToProtoBinary {
                    replacer,
                    proto_msg_type,
                } => {
                    let binary_field = json_pointer(&mut cosmos_msg, replacer).ok_or(
                        ContractError::ReplacerFieldNotFound {
                            replacer: replacer.to_owned(),
                        },
                    )?;

                    let binary = match proto_msg_type {
                        ProtoMessageType::IbcTransfer => {
                            let ibc: MsgTransfer = binary_field
                                .clone()
                                .deserialize_into::<CwIbcMessage>()
                                .map_err(|_| ContractError::ProtoSerializationError {})?
                                .into();

                            self.encode_proto_msg(&ibc)?
                        }
                        ProtoMessageType::OsmosisSwapExactAmtIn => {
                            let swap: MsgSwapExactAmountIn = binary_field
                                .clone()
                                .deserialize_into::<MsgSwapExactAmountIn>()
                                .map_err(|_| ContractError::ProtoSerializationError {})?;

                            self.encode_proto_msg(&swap)?
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
        let field =
            json_pointer(cosmos_msg, replacer).ok_or(ContractError::ReplacerFieldNotFound {
                replacer: replacer.to_owned(),
            })?;

        *field = serde_cw_value::Value::String(value.to_owned());
        Ok(())
    }

    fn encode_proto_msg<T: prost::Message>(&self, msg: &T) -> Result<Binary, ContractError> {
        let mut bytes = Vec::new();
        prost::Message::encode(msg, &mut bytes)
            .map_err(|_| ContractError::ProtoSerializationError {})?;

        Ok(Binary(bytes))
    }
}
