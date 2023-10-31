use cosmwasm_schema::cw_serde;
use cosmwasm_std::{testing::mock_env, to_binary, BankMsg, Coin, CosmosMsg, Uint128, WasmMsg};
use ibc_tracking::msg::MsgTransfer;
use serde_cw_value::Value;

use crate::{
    msg::{Call, CallAction, MsgReplyId, ProtoMessageType, ReplaceInfo},
    state::MulticallState,
    ContractError,
};

use self::mock_querier::mock_dependencies;

#[test]
fn test_multicall_state() {
    let state = MulticallState::new(&mut [], "addr0000".to_owned());
    match state {
        Err(ContractError::EmptyCallsList {}) => (),
        _ => panic!("expecting ContractError::EmptyCallsList"),
    };

    let state = MulticallState::new(
        &mut [Call {
            msg: Value::String("msg".to_owned()).into(),
            actions: vec![CallAction::IbcTracking {
                channel: "channel-0".to_owned(),
                denom: "usquid".to_owned(),
                amount: None,
                amount_pointer: None,
            }],
        }],
        "addr0000".to_owned(),
    );
    match state {
        Err(ContractError::EitherAmountOfPointerMustBeSet {}) => (),
        _ => panic!("expecting ContractError::EitherAmountOfPointerMustBeSet"),
    };

    let state = MulticallState::new(
        &mut [Call {
            msg: Value::String("msg".to_owned()).into(),
            actions: vec![
                CallAction::IbcTracking {
                    channel: "channel-0".to_owned(),
                    denom: "usquid".to_owned(),
                    amount: Some(Uint128::from(1u128)),
                    amount_pointer: None,
                },
                CallAction::IbcTracking {
                    channel: "channel-0".to_owned(),
                    denom: "usquid".to_owned(),
                    amount: Some(Uint128::from(1u128)),
                    amount_pointer: None,
                },
            ],
        }],
        "addr0000".to_owned(),
    );
    match state {
        Err(ContractError::InvalidCallActionArgument { msg }) => {
            assert_eq!(
                msg,
                "Only one or none IbcTracking call action entries allowed per call".to_owned()
            )
        }
        _ => panic!("expecting ContractError::InvalidCallActionArgument"),
    };

    let state = MulticallState::new(
        &mut [Call {
            msg: Value::String("msg".to_owned()).into(),
            actions: vec![CallAction::NativeBalanceFetch {
                denom: "usquid".to_owned(),
                replacer: "".to_owned(),
            }],
        }],
        "addr0000".to_owned(),
    );
    match state {
        Err(ContractError::InvalidReplacer {}) => (),
        _ => panic!("expecting ContractError::InvalidReplacer"),
    };

    let state = MulticallState::new(
        &mut [Call {
            msg: Value::String("msg".to_owned()).into(),
            actions: vec![CallAction::Cw20BalanceFetch {
                contract: "usquid".to_owned(),
                replacer: "funds/0/amount".to_owned(),
            }],
        }],
        "addr0000".to_owned(),
    );
    match state {
        Err(ContractError::InvalidReplacer {}) => (),
        _ => panic!("expecting ContractError::InvalidReplacer"),
    };

    let state = MulticallState::new(
        &mut [Call {
            msg: Value::String("msg".to_owned()).into(),
            actions: vec![CallAction::IbcTracking {
                channel: "channel-0".to_owned(),
                denom: "usquid".to_owned(),
                amount: None,
                amount_pointer: Some("invalid/replacer".to_owned()),
            }],
        }],
        "addr0000".to_owned(),
    );
    match state {
        Err(ContractError::InvalidReplacer {}) => (),
        _ => panic!("expecting ContractError::InvalidReplacer"),
    };

    let state = MulticallState::new(
        &mut [Call {
            msg: Value::String("msg".to_owned()).into(),
            actions: vec![CallAction::CustomReplaceQuery {
                query_msg: Value::String("msg".to_owned()).into(),
                replacers: vec![
                    ReplaceInfo {
                        response_pointer: "/valid/replacer".to_owned(),
                        replacer: "/valid/replacer".to_owned(),
                    },
                    ReplaceInfo {
                        response_pointer: "invalid/replacer".to_owned(),
                        replacer: "/valid/replacer".to_owned(),
                    },
                ],
            }],
        }],
        "addr0000".to_owned(),
    );
    match state {
        Err(ContractError::InvalidReplacer {}) => (),
        _ => panic!("expecting ContractError::InvalidReplacer"),
    };

    let state = MulticallState::new(
        &mut [Call {
            msg: Value::String("msg".to_owned()).into(),
            actions: vec![CallAction::CustomReplaceQuery {
                query_msg: Value::String("msg".to_owned()).into(),
                replacers: vec![
                    ReplaceInfo {
                        response_pointer: "/valid/replacer".to_owned(),
                        replacer: "invalid/replacer".to_owned(),
                    },
                    ReplaceInfo {
                        response_pointer: "/valid/replacer".to_owned(),
                        replacer: "/valid/replacer".to_owned(),
                    },
                ],
            }],
        }],
        "addr0000".to_owned(),
    );
    match state {
        Err(ContractError::InvalidReplacer {}) => (),
        _ => panic!("expecting ContractError::InvalidReplacer"),
    };

    let valid_state = MulticallState::new(
        &mut [
            Call {
                msg: Value::String("msg".to_owned()).into(),
                actions: vec![
                    CallAction::IbcTracking {
                        channel: "channel-0".to_owned(),
                        denom: "usquid".to_owned(),
                        amount: Some(Uint128::from(1u128)),
                        amount_pointer: None,
                    },
                    CallAction::Cw20BalanceFetch {
                        contract: "usquid".to_owned(),
                        replacer: "/valid/replacer".to_owned(),
                    },
                    CallAction::FieldToProtoBinary {
                        replacer: "/valid/replacer".to_owned(),
                        proto_msg_type: ProtoMessageType::IbcTransfer,
                    },
                    CallAction::NativeBalanceFetch {
                        denom: "usquid".to_owned(),
                        replacer: "/valid/replacer".to_owned(),
                    },
                    CallAction::FieldToBinary {
                        replacer: "/valid/replacer".to_owned(),
                    },
                    CallAction::CustomReplaceQuery {
                        query_msg: Value::String("msg".to_owned()).into(),
                        replacers: vec![
                            ReplaceInfo {
                                response_pointer: "/valid/replacer".to_owned(),
                                replacer: "/valid/replacer".to_owned(),
                            },
                            ReplaceInfo {
                                response_pointer: "/valid/replacer".to_owned(),
                                replacer: "/valid/replacer".to_owned(),
                            },
                        ],
                    },
                ],
            },
            Call {
                msg: Value::String("msg2".to_owned()).into(),
                actions: vec![
                    CallAction::IbcTracking {
                        channel: "channel-0".to_owned(),
                        denom: "usquid".to_owned(),
                        amount: Some(Uint128::from(1u128)),
                        amount_pointer: None,
                    },
                    CallAction::NativeBalanceFetch {
                        denom: "usquid".to_owned(),
                        replacer: "/valid/replacer".to_owned(),
                    },
                ],
            },
            Call {
                msg: Value::String("msg3".to_owned()).into(),
                actions: vec![],
            },
        ],
        "addr0000".to_owned(),
    );
    assert_eq!(valid_state.is_ok(), true);

    let mut valid_state = valid_state.unwrap();
    assert_eq!(valid_state.calls.len(), 3);

    let call1 = valid_state.next_call();
    assert_eq!(
        call1,
        Some(&Call {
            msg: Value::String("msg".to_owned()).into(),
            actions: vec![
                CallAction::NativeBalanceFetch {
                    denom: "usquid".to_owned(),
                    replacer: "/valid/replacer".to_owned(),
                },
                CallAction::Cw20BalanceFetch {
                    contract: "usquid".to_owned(),
                    replacer: "/valid/replacer".to_owned(),
                },
                CallAction::CustomReplaceQuery {
                    query_msg: Value::String("msg".to_owned()).into(),
                    replacers: vec![
                        ReplaceInfo {
                            response_pointer: "/valid/replacer".to_owned(),
                            replacer: "/valid/replacer".to_owned(),
                        },
                        ReplaceInfo {
                            response_pointer: "/valid/replacer".to_owned(),
                            replacer: "/valid/replacer".to_owned(),
                        },
                    ],
                },
                CallAction::IbcTracking {
                    channel: "channel-0".to_owned(),
                    denom: "usquid".to_owned(),
                    amount: Some(Uint128::from(1u128)),
                    amount_pointer: None,
                },
                CallAction::FieldToBinary {
                    replacer: "/valid/replacer".to_owned(),
                },
                CallAction::FieldToProtoBinary {
                    replacer: "/valid/replacer".to_owned(),
                    proto_msg_type: ProtoMessageType::IbcTransfer,
                },
            ]
        })
    );

    let call2 = valid_state.next_call();
    assert_eq!(
        call2,
        Some(&Call {
            msg: Value::String("msg2".to_owned()).into(),
            actions: vec![
                CallAction::NativeBalanceFetch {
                    denom: "usquid".to_owned(),
                    replacer: "/valid/replacer".to_owned(),
                },
                CallAction::IbcTracking {
                    channel: "channel-0".to_owned(),
                    denom: "usquid".to_owned(),
                    amount: Some(Uint128::from(1u128)),
                    amount_pointer: None,
                },
            ],
        })
    );

    let call3 = valid_state.next_call();
    assert_eq!(
        call3,
        Some(&Call {
            msg: Value::String("msg3".to_owned()).into(),
            actions: vec![],
        })
    );

    let call4 = valid_state.next_call();
    assert_eq!(call4, None);
}

#[test]
fn test_call_into_msg() {
    let mut deps = mock_dependencies(&[]);
    let deps = deps.as_mut();
    let env = mock_env();

    let bank_send_call = Call {
        msg: serde_json_wasm::from_str(
            r#"
        {
            "bank": {
              "send": {
                "to_address": "squid19he7u694v4ekp6gm489e7skyz7lwdzmsjvduvq",
                "amount": [
                  {
                    "denom": "usquid",
                    "amount": "0"
                  }
                ]
              }
            }
          }
        "#,
        )
        .unwrap(),
        actions: vec![CallAction::NativeBalanceFetch {
            denom: "usquid".to_owned(),
            replacer: "/bank/send/amount/0/amount".to_owned(),
        }],
    };

    let bank_send_msg = bank_send_call
        .try_into_msg(deps.storage, &deps.querier, &env, "addr0000")
        .unwrap();

    assert_eq!(bank_send_msg.id, MsgReplyId::ProcessCall.repr());
    assert_eq!(
        bank_send_msg.msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "squid19he7u694v4ekp6gm489e7skyz7lwdzmsjvduvq".to_owned(),
            amount: vec![Coin {
                denom: "usquid".to_owned(),
                amount: Uint128::from(333u128),
            }]
        })
    );

    #[cw_serde]
    struct WasmStakeMsg {
        pub amount: String,
        pub on_behalf: String,
    }

    let wasm_call = Call {
        msg: serde_json_wasm::from_str(
            r#"
        {
            "wasm": {
                "execute": {
                    "contract_addr": "addr0001",
                    "msg": {
                        "on_behalf": "addr0002",
                        "amount": "0"
                    },
                    "funds": [
                        {
                            "denom": "usquid",
                            "amount": "0"
                        }
                    ]
                }
            }
        }
        "#,
        )
        .unwrap(),
        actions: vec![
            CallAction::Cw20BalanceFetch {
                contract: "cw20".to_owned(),
                replacer: "/wasm/execute/msg/amount".to_owned(),
            },
            CallAction::Cw20BalanceFetch {
                contract: "cw20".to_owned(),
                replacer: "/wasm/execute/funds/0/amount".to_owned(),
            },
            CallAction::FieldToBinary {
                replacer: "/wasm/execute/msg".to_owned(),
            },
        ],
    };

    let wasm_msg = wasm_call
        .try_into_msg(deps.storage, &deps.querier, &env, "addr0000")
        .unwrap();

    assert_eq!(wasm_msg.id, MsgReplyId::ProcessCall.repr());
    assert_eq!(
        wasm_msg.msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "addr0001".to_owned(),
            msg: to_binary(&WasmStakeMsg {
                on_behalf: "addr0002".to_owned(),
                amount: "1337".to_owned(),
            })
            .unwrap(),
            funds: vec![Coin {
                denom: "usquid".to_owned(),
                amount: Uint128::from(1337u128),
            }]
        })
    );

    let custom_query_call = Call {
        msg: serde_json_wasm::from_str(
            r#"
        {
            "bank": {
              "send": {
                "to_address": "squid19he7u694v4ekp6gm489e7skyz7lwdzmsjvduvq",
                "amount": [
                  {
                    "denom": "u",
                    "amount": "0"
                  }
                ]
              }
            }
          }
        "#,
        )
        .unwrap(),
        actions: vec![CallAction::CustomReplaceQuery {
            query_msg: serde_json_wasm::from_str(
                r#"
            {
                "wasm": {
                    "smart": {
                        "contract_addr": "fee",
                        "msg": "eyJmZWUiOnt9fQ=="
                    }
                }
            }
            "#,
            )
            .unwrap(),
            replacers: vec![
                ReplaceInfo {
                    response_pointer: "/denom".to_owned(),
                    replacer: "/bank/send/amount/0/denom".to_owned(),
                },
                ReplaceInfo {
                    response_pointer: "/fee".to_owned(),
                    replacer: "/bank/send/amount/0/amount".to_owned(),
                },
            ],
        }],
    };

    let custom_query_msg = custom_query_call
        .try_into_msg(deps.storage, &deps.querier, &env, "addr0000")
        .unwrap();

    assert_eq!(custom_query_msg.id, MsgReplyId::ProcessCall.repr());
    assert_eq!(
        custom_query_msg.msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "squid19he7u694v4ekp6gm489e7skyz7lwdzmsjvduvq".to_owned(),
            amount: vec![Coin {
                denom: "ufee".to_owned(),
                amount: Uint128::from(1312u128),
            }]
        })
    );

    let ibc_call = Call {
        msg: serde_json_wasm::from_str(
            r#"
        {
            "stargate": {
                "type_url": "/ibc.applications.transfer.v1.MsgTransfer",
                "value": {
                    "source_port": "transfer",
                    "source_channel": "channel-3",
                    "token": {
                        "denom": "usquid",
                        "amount": "111111"
                    },
                    "sender": "osmo1vmpds4p8grwz54dygeljhq9vffssw5caydyj3heqd02f2seckk3smlug7w",
                    "receiver": "axelar15t9awn6jnxheckur5vc6dqv6pqlpph0hw24vwf",
                    "timeout_timestamp": 1693856646000000000,
                    "memo": "{\"ibc_callback\":\"addr0000\"}"
                }
            }
        }
        "#,
        )
        .unwrap(),
        actions: vec![
            CallAction::IbcTracking {
                channel: "channel-3".to_owned(),
                denom: "usquid".to_owned(),
                amount: Some(Uint128::from(111111u128)),
                amount_pointer: None,
            },
            CallAction::FieldToProtoBinary {
                replacer: "/stargate/value".to_owned(),
                proto_msg_type: ProtoMessageType::IbcTransfer,
            },
        ],
    };

    let ibc_msg = ibc_call
        .try_into_msg(deps.storage, &deps.querier, &env, "addr0004")
        .unwrap();

    assert_eq!(ibc_msg.id, MsgReplyId::IbcTransferTracking.repr());
    assert_eq!(
        ibc_msg.msg,
        MsgTransfer {
            source_port: "transfer".to_owned(),
            source_channel: "channel-3".to_owned(),
            token: Some(osmosis_std::types::cosmos::base::v1beta1::Coin {
                denom: "usquid".to_owned(),
                amount: "111111".to_owned(),
            }),
            sender: "osmo1vmpds4p8grwz54dygeljhq9vffssw5caydyj3heqd02f2seckk3smlug7w".to_owned(),
            receiver: "axelar15t9awn6jnxheckur5vc6dqv6pqlpph0hw24vwf".to_owned(),
            timeout_height: None,
            timeout_timestamp: Some(1693856646000000000),
            memo: "{\"ibc_callback\":\"addr0000\"}".to_owned(),
        }
        .into()
    );

    let ibc_call = Call {
        msg: serde_json_wasm::from_str(
            r#"
        {
            "stargate": {
                "type_url": "/ibc.applications.transfer.v1.MsgTransfer",
                "value": {
                    "source_port": "transfer",
                    "source_channel": "channel-3",
                    "token": {
                        "denom": "usquid",
                        "amount": "111111"
                    },
                    "sender": "osmo1vmpds4p8grwz54dygeljhq9vffssw5caydyj3heqd02f2seckk3smlug7w",
                    "receiver": "axelar15t9awn6jnxheckur5vc6dqv6pqlpph0hw24vwf",
                    "timeout_timestamp": 1693856646000000000,
                    "memo": "{\"ibc_callback\":\"addr0000\"}"
                }
            }
        }
        "#,
        )
        .unwrap(),
        actions: vec![
            CallAction::IbcTracking {
                channel: "channel-3".to_owned(),
                denom: "usquid".to_owned(),
                amount: None,
                amount_pointer: Some("/stargate/value/token/amount".to_owned()),
            },
            CallAction::FieldToProtoBinary {
                replacer: "/stargate/value".to_owned(),
                proto_msg_type: ProtoMessageType::IbcTransfer,
            },
        ],
    };

    let ibc_msg = ibc_call
        .try_into_msg(deps.storage, &deps.querier, &env, "addr0004")
        .unwrap();

    assert_eq!(ibc_msg.id, MsgReplyId::IbcTransferTracking.repr());
    assert_eq!(
        ibc_msg.msg,
        MsgTransfer {
            source_port: "transfer".to_owned(),
            source_channel: "channel-3".to_owned(),
            token: Some(osmosis_std::types::cosmos::base::v1beta1::Coin {
                denom: "usquid".to_owned(),
                amount: "111111".to_owned(),
            }),
            sender: "osmo1vmpds4p8grwz54dygeljhq9vffssw5caydyj3heqd02f2seckk3smlug7w".to_owned(),
            receiver: "axelar15t9awn6jnxheckur5vc6dqv6pqlpph0hw24vwf".to_owned(),
            timeout_height: None,
            timeout_timestamp: Some(1693856646000000000),
            memo: "{\"ibc_callback\":\"addr0000\"}".to_owned(),
        }
        .into()
    );

    let invalid_replacer_call = Call {
        msg: serde_json_wasm::from_str(
            r#"
        {
            "bank": {
              "send": {
                "to_address": "squid19he7u694v4ekp6gm489e7skyz7lwdzmsjvduvq",
                "amount": [
                  {
                    "denom": "usquid",
                    "amount": "0"
                  }
                ]
              }
            }
          }
        "#,
        )
        .unwrap(),
        actions: vec![CallAction::NativeBalanceFetch {
            denom: "usquid".to_owned(),
            replacer: "/bank/send/amount/1/amount".to_owned(),
        }],
    };

    let invalid_replacer_msg_err = invalid_replacer_call
        .try_into_msg(deps.storage, &deps.querier, &env, "addr0004")
        .unwrap_err();

    match invalid_replacer_msg_err {
        ContractError::ReplacerFieldNotFound { replacer } => {
            assert_eq!(replacer, "/bank/send/amount/1/amount".to_owned());
        }
        _ => panic!("unexpected error"),
    };

    let zero_balance_call = Call {
        msg: serde_json_wasm::from_str(
            r#"
        {
            "bank": {
              "send": {
                "to_address": "squid19he7u694v4ekp6gm489e7skyz7lwdzmsjvduvq",
                "amount": [
                  {
                    "denom": "uzero",
                    "amount": "0"
                  }
                ]
              }
            }
          }
        "#,
        )
        .unwrap(),
        actions: vec![CallAction::NativeBalanceFetch {
            denom: "uzero".to_owned(),
            replacer: "/bank/send/amount/0/amount".to_owned(),
        }],
    };

    let zero_balance_msg_err = zero_balance_call
        .try_into_msg(deps.storage, &deps.querier, &env, "addr0004")
        .unwrap_err();

    match zero_balance_msg_err {
        ContractError::ZeroBalanceFetched { token } => {
            assert_eq!(token, "uzero".to_owned());
        }

        _ => panic!("unexpected error"),
    }
}

#[cfg(test)]
mod mock_querier {
    use std::marker::PhantomData;

    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{
        from_binary, from_slice, to_binary, BalanceResponse, BankQuery, Coin, ContractResult,
        OwnedDeps, Querier, QuerierResult, QueryRequest, SystemError, SystemResult, Uint128,
        WasmQuery,
    };
    use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20QueryMsg};
    use shared::SerializableJson;

    #[cw_serde]
    pub enum CustomQueryTestMsg {
        Fee {},
    }

    #[cw_serde]
    pub struct TestFeeResponse {
        pub denom: String,
        pub fee: Uint128,
    }

    pub fn mock_dependencies(
        contract_balance: &[Coin],
    ) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier, SerializableJson> {
        let custom_querier: WasmMockQuerier =
            WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));

        OwnedDeps {
            api: MockApi::default(),
            storage: MockStorage::default(),
            querier: custom_querier,
            custom_query_type: PhantomData,
        }
    }

    pub struct WasmMockQuerier {
        base: MockQuerier<SerializableJson>,
    }

    impl Querier for WasmMockQuerier {
        fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
            // MockQuerier doesn't support Custom, so we ignore it completely here
            let request: QueryRequest<SerializableJson> = match from_slice(bin_request) {
                Ok(v) => v,
                Err(e) => {
                    return SystemResult::Err(SystemError::InvalidRequest {
                        error: format!("Parsing query request: {}", e),
                        request: bin_request.into(),
                    })
                }
            };
            self.handle_query(&request)
        }
    }

    impl WasmMockQuerier {
        pub fn handle_query(&self, request: &QueryRequest<SerializableJson>) -> QuerierResult {
            match &request {
                QueryRequest::Bank(BankQuery::Balance { denom, .. }) => match denom.as_str() {
                    "usquid" => {
                        SystemResult::Ok(ContractResult::from(to_binary(&BalanceResponse {
                            amount: Coin {
                                denom: "usquid".to_owned(),
                                amount: Uint128::from(333u128),
                            },
                        })))
                    }
                    "uzero" => {
                        SystemResult::Ok(ContractResult::from(to_binary(&BalanceResponse {
                            amount: Coin {
                                denom: "uzero".to_owned(),
                                amount: Uint128::zero(),
                            },
                        })))
                    }
                    _ => panic!("query not mocked"),
                },
                QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                    match contract_addr.as_str() {
                        "cw20" => match from_binary(msg).unwrap() {
                            Cw20QueryMsg::Balance { .. } => SystemResult::Ok(ContractResult::Ok(
                                to_binary(&Cw20BalanceResponse {
                                    balance: Uint128::from(1337u128),
                                })
                                .unwrap(),
                            )),
                            _ => panic!("query not mocked"),
                        },
                        "fee" => match from_binary(msg).unwrap() {
                            CustomQueryTestMsg::Fee {} => SystemResult::Ok(ContractResult::Ok(
                                to_binary(&TestFeeResponse {
                                    denom: "ufee".to_owned(),
                                    fee: Uint128::from(1312u128),
                                })
                                .unwrap(),
                            )),
                        },
                        _ => panic!("query not mocked"),
                    }
                }
                _ => self.base.handle_query(request),
            }
        }
    }

    impl WasmMockQuerier {
        pub fn new(base: MockQuerier<SerializableJson>) -> Self {
            WasmMockQuerier { base }
        }
    }
}
