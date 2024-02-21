use std::str::FromStr;

use cosmwasm_std::{
    Coin, CosmosMsg, Decimal, Deps, Env, Reply, Storage, SubMsgResponse, SubMsgResult, Uint128,
};
use osmosis_std::types::osmosis::gamm::v1beta1::{
    MsgSwapExactAmountIn, MsgSwapExactAmountInResponse,
};
use osmosis_std::types::osmosis::poolmanager::v1beta1::SwapAmountInRoute;

use crate::{
    error::OsmosisRouterError,
    state::{has_processing_swap, load_processing_swap, store_processing_swap, ProcessingSwap},
    OsmosisPath, OsmosisSimulateSwapResponse, OsmosisSwapMsg, OsmosisSwapReply,
};

pub fn build_swap_msg(
    storage: &mut dyn Storage,
    env: &Env,
    input_coin: Coin,
    msg: OsmosisSwapMsg,
) -> Result<CosmosMsg, OsmosisRouterError> {
    if has_processing_swap(storage)? {
        return Err(OsmosisRouterError::SwapIsAlreadyInProcess {});
    }

    let pool_path = OsmosisPath(msg.path);
    pool_path.validate_path(&input_coin.denom)?;

    store_processing_swap(
        storage,
        &ProcessingSwap {
            output_denom: pool_path.get_path_output_denom(),
        },
    )?;

    let swap_msg = MsgSwapExactAmountIn {
        sender: env.contract.address.to_string(),
        routes: pool_path.0,
        token_in: Some(input_coin.into()),
        token_out_min_amount: msg.token_out_min_amount,
    };

    Ok(swap_msg.into())
}

pub fn get_swap_amount_out_response(
    storage: &mut dyn Storage,
    msg: Reply,
) -> Result<OsmosisSwapReply, OsmosisRouterError> {
    if let SubMsgResult::Ok(SubMsgResponse { data: Some(b), .. }) = msg.result {
        let res: MsgSwapExactAmountInResponse = b.try_into().map_err(OsmosisRouterError::Std)?;
        let processing_swap = load_processing_swap(storage)?;

        return Ok(OsmosisSwapReply {
            output_coin: Coin {
                denom: processing_swap.output_denom,
                amount: Uint128::from_str(&res.token_out_amount)?,
            },
        });
    }

    Err(OsmosisRouterError::FailedSwap {
        reason: msg.result.unwrap_err(),
    })
}

pub fn estimate_min_twap_output(
    deps: Deps,
    env: &Env,
    input_coin: Coin,
    path: Vec<SwapAmountInRoute>,
    slippage: Decimal,
) -> Result<OsmosisSimulateSwapResponse, OsmosisRouterError> {
    let pool_path = OsmosisPath(path);
    pool_path.validate_path(&input_coin.denom)?;

    let output_coin = pool_path.calculate_twap_output_amount(
        &deps.querier,
        &input_coin,
        slippage,
        &env.block.time,
    )?;

    Ok(OsmosisSimulateSwapResponse { output_coin })
}
