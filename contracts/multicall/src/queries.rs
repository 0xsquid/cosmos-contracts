use cosmwasm_std::{Deps, QueryRequest, StdError, StdResult};
use shared::SerializableJson;

use crate::msg::MultiQueryResponse;

/// ## Description
/// Sequentially executes provided queries and returns corresponding responses in the [`MultiQueryResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
///
/// * **queries** is an object of type [`Vec<SerializableJson>`]
pub fn multi_query(
    deps: Deps<SerializableJson>,
    queries: Vec<SerializableJson>,
) -> StdResult<MultiQueryResponse> {
    let mut responses = vec![];

    for query in queries.iter() {
        let query_msg: QueryRequest<SerializableJson> = query
            .clone()
            .0
            .deserialize_into()
            .map_err(|_| StdError::generic_err("Failed to serialize query message"))?;

        let response: serde_cw_value::Value = deps.querier.query(&query_msg)?;
        responses.push(response.into());
    }

    Ok(MultiQueryResponse { responses })
}
