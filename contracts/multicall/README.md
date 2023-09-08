# Multicall

Multicall contract is designed for performing multiple actions in a matter of one transaction.
For example sending tokens to multiple addresses or staking some funds in exchange for another and then sending it to some other cosmos chain via ibc message.

## Execute Msg

### `multicall`

Executes a set of cosmos messages specified in the calls array

```json
{
  "multicall": {
    "calls": [
      {
        "msg": {},
        "actions": []
      }
    ]
  }
}
```

Note: theres another execute msg type `process_next_call` - it can only be called by the contract itself, otherwise transaction will always be reverted.

## Multicall Call structure

Multicall action accepts an array of `Call` objects. Each `Call` object has two fields - `msg` and `actions`.

- `msg` field is a valid JSON object of type `CosmosMsg`, so providing JSON that won't serialize into `CosmosMsg` will cause a serialization error and will revert transaction execution process.

- `actions` array is a set of instructions that can be performed before serializing `msg` into `CosmosMsg` type and sending it to the node.

## Call Actions

### `native_balance_fetch`
Queries bank module contract's balance and replaces received value in the message.

```json
{
    "native_balance_fetch": {
        "denom": "usquid",
        "replacer": "/path/to/field/for/replacement"
    }
}
```

### `cw20_balance_fetch`
Queries cw20 token contract's balance and replaces received value in the message

```json
{
    "cw20_balance_fetch": {
        "contract": "squid1...",
        "replacer": "/path/to/field/for/replacement"
    }
}
```

### `custom_replace_query`
Makes a custom query and replaces msg values using data from the query response.
Both [`CallAction::NativeBalanceFetch`] & [`CallAction::Cw20BalanceFetch`] can be done via this call action type.
`query_msg` field must be a valid json message of type [`cosmwasm_std::QueryRequest`]

```json
{
    "custom_replace_query": {
        "query_msg": {},
        "replacers": [
            {
                "response_pointer": "/path/to/query/response/field",
                "replacer": "/path/to/field/for/replacement"
            }
        ]
    }
}
```

### `ibc_tracking`
Enables ibc tracking for sent ibc transfer messages from the multicall contract.
Can be passed when sending ibc transfer message.
Note: either 'amount' or 'pointer' field must be set, otherwise validation error will be returned.

```json
{
    "ibc_tracking": {
        "channel": "channel-0",
        "denom": "usquid",
        "amount": "1" || null,
        "amount_pointer" "/path/to/amount/field" || null
    }
}
```

### `field_to_binary`
Converts specified field into [`Binary`] type

```json
{
    "field_to_binary": {
        "replacer": "/path/to/field/for/replacement"
    }
}
```

### `field_to_proto_binary`
Converts specified field into [`Binary`] type encoded using [`prost::Message::encode`] method.
Note: since the type of the message should be known to the contract at the moment only ibc transfer is supported.

```json
{
    "field_to_proto_binary": {
        "replacer": "/path/to/field/for/replacement",
        "proto_msg_type": "ibc_transfer"
    }
}
```

## Replacer
Replacer is basically a path from the root of the `msg` object to a field. For example here is the valid multicall message for doing a bank send action:

```json
{
  "multicall": {
    "calls": [
      {
        "msg": {
          "bank": {
            "send": {
              "to_address": "osmo19he7u694v4ekp6gm489e7skyz7lwdzmsjvduvq",
              "amount": [
                {
                  "denom": "uosmo",
                  "amount": "0"
                }
              ]
            }
          }
        },
        "actions": [
          {
            "native_balance_fetch": {
              "denom": "uosmo",
              "replacer": "/bank/send/amount/0/amount"
            }
          }
        ]
      }
    ]
  }
}
```

`replacer` value is equals to a path to `amount` field inside bank message. This field will be replaced with fetched contract balance of `uosmo` coin. Numbers in the path are equal to an index in the array.

## Fallback address
Fallback address is an optional field that must be set only when `ibc_tracking` call action is selected, otherwise it can be null.

## Example calls

### Sending and ibc transfer message:

```json
{
  "multicall": {
    "calls": [
      {
        "msg": {
            "stargate": {
                "type_url": "/ibc.applications.transfer.v1.MsgTransfer",
                "value": {
                    "source_port": "transfer",
                    "source_channel": "channel-3",
                    "token": {
                        "denom": "ibc/6F34E1BD664C36CE49ACC28E60D62559A5F96C4F9A6CCE4FC5A67B2852E24CFE",
                        "amount": "0"
                    },
                    "sender": "osmo1vmpds4p8grwz54dygeljhq9vffssw5caydyj3heqd02f2seckk3smlug7w",
                    "receiver": "axelar15t9awn6jnxheckur5vc6dqv6pqlpph0hw24vwf",
                    "timeout_timestamp": 1693856646000000000,
                    "memo": "{\"ibc_callback\":\"osmo1vmpds4p8grwz54dygeljhq9vffssw5caydyj3heqd02f2seckk3smlug7w\"}"
                }
            }
        },
        "actions": [
          {
            "native_balance_fetch": {
              "denom": "ibc/6F34E1BD664C36CE49ACC28E60D62559A5F96C4F9A6CCE4FC5A67B2852E24CFE",
              "replacer": "/stargate/value/token/amount"
            }
          },
          {
            "ibc_tracking": {
                "channel": "channel-3",
                "denom": "ibc/6F34E1BD664C36CE49ACC28E60D62559A5F96C4F9A6CCE4FC5A67B2852E24CFE",
                "amount_pointer" "/stargate/value/token/amount"
            }
          },
          {
            "msg_to_proto_binary": {
                "replacer": "/stargate/value",
                "proto_msg_type": "ibc_transfer"
            }
          }
        ]
      }
    ],
    "fallback_address": "osmo1eaztm3pqrkw2xgt0lxppahtx5v5pndmjg6yfrh"
  }
}
```

First the contract will fetch the balance of axlUsdc on osmosis and update the amount field in the message.

Second it will enable ibc tracking mechanism for this call.

IMPORTANT NOTE: `ibc_callback` field must be set by the sender, otherwise tracking won't work since the destination chain will not send IBC ACK back to the source chain.

Third - the contract will serialize whole message from the `value` field into Binary format and send it.

### Creating an ICA

```json
{
  "multicall": {
    "calls": [
      {
        "msg": {
            "custom": {
                "register_interchain_account": {
                    "connection_id": "connection-8",
                    "interchain_account_id": "squid0001"
                }
            }
        },
        "actions": []
      }
    ]
  }
}
```

Actions array can also be empty if there is no actions required. In this case the contract will simply proxy provided message.

### Sending a WASM contract call

```json
{
  "multicall": {
    "calls": [
      {
        "msg": {
            "wasm": {
                "execute": {
                    "contract_addr": "osmo1",
                    "msg": {
                        "deposit": {
                            "on_behalf": "osmo1"
                        }
                    },
                    "funds": [
                        {
                            "denom": "uosmo",
                            "amount": "0"
                        }
                    ]
                }
            }
        },
        "actions": [
            {
                "native_balance_fetch": {
                    "denom": "uosmo",
                    "replacer": "/wasm/execute/funds/0/amount"
                }
            },
            {
                "field_to_binary": {
                    "replacer": "/wasm/execute/msg"
                }
            }
        ]
      }
    ]
  }
}
```

First the contract will fetch `uosmo` balance and update the funds information for a wasm call.

Second - since `msg` field in a wasm call must be a base64 encoded Binary object, but not a JSON object we need to use `field_to_binary` action. It will take `msg` field at the specified path and convert it into Binary. Resulting message that will be sent will look like this:

```json
{
    "wasm": {
        "execute": {
            "contract_addr": "osmo1",
            "msg": "eyJkZXBvc2l0Ijp7Im9uX2JlaGFsZiI6Im9zbW8xIn19",
            "funds": [
                {
                    "denom": "uosmo",
                    "amount": "111111"
                }
            ]
        }
    }
}
```

Note: since there is no updates for wasm message field it can be alredy specified as a binary value. Here it is shown for explanaition purposes.