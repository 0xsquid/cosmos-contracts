use cosmwasm_std::{CustomMsg, CustomQuery};
use schemars::JsonSchema;

pub mod util;

#[cfg(test)]
mod tests;

#[derive(
    ::cosmwasm_schema::serde::Serialize,
    ::cosmwasm_schema::serde::Deserialize,
    ::std::clone::Clone,
    ::std::fmt::Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub struct SerializableJson(pub serde_cw_value::Value);

impl SerializableJson {
    pub fn as_value(&self) -> &serde_cw_value::Value {
        &self.0
    }
}

impl JsonSchema for SerializableJson {
    fn schema_name() -> String {
        "JSON".to_string()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        schemars::schema::Schema::from(true)
    }
}

impl From<serde_cw_value::Value> for SerializableJson {
    fn from(value: serde_cw_value::Value) -> Self {
        Self(value)
    }
}

impl CustomMsg for SerializableJson {}

impl CustomQuery for SerializableJson {}
