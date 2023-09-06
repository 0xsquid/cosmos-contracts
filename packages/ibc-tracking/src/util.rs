use cosmwasm_std::Env;
use serde_cw_value::Value;

const IBC_CALLBACK: &str = "ibc_callback";

pub fn insert_callback_key(memo: Value, env: &Env) -> Value {
    let serde_cw_value::Value::Map(mut m) = memo else { unreachable!() };
    m.insert(
        serde_cw_value::Value::String(IBC_CALLBACK.to_owned()),
        serde_cw_value::Value::String(env.contract.address.to_string()),
    );

    serde_cw_value::Value::Map(m)
}
