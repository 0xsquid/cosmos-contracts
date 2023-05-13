use std::collections::HashSet;
use std::ops::{Div, Mul};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, QuerierWrapper, Timestamp, Uint128};
use error::OsmosisRouterError;
use osmosis_std::types::osmosis::gamm::v1beta1::SwapAmountInRoute;
use osmosis_std::{
    shim::Timestamp as OsmosisTimestamp, types::osmosis::twap::v1beta1::TwapQuerier,
};

pub mod error;
pub mod router;
pub mod state;

const TWAP_WINDOW: u64 = 3600;

pub struct OsmosisPath(Vec<SwapAmountInRoute>);

impl OsmosisPath {
    pub fn validate_path(&self, input_denom: &str) -> Result<(), OsmosisRouterError> {
        let mut next_input_denom = input_denom;
        let mut seen_denoms: HashSet<&str> = [next_input_denom].iter().cloned().collect();

        for step in self.0.iter() {
            if seen_denoms.contains(step.token_out_denom.as_str()) {
                return Err(OsmosisRouterError::InvalidPath {});
            }

            next_input_denom = &step.token_out_denom;
            seen_denoms.insert(next_input_denom);
        }

        Ok(())
    }

    pub fn calculate_twap_output_amount(
        &self,
        querier: &QuerierWrapper,
        input_coin: &cosmwasm_std::Coin,
        slippage: Decimal,
        now: &Timestamp,
    ) -> Result<cosmwasm_std::Coin, OsmosisRouterError> {
        let (start_time, end_time) = self.get_twap_window(&now);

        let mut price = Decimal::one();
        let mut next_input_denom = input_coin.denom.as_str();

        for step in self.0.iter() {
            let output_denom = step.token_out_denom.as_str();

            let pool_price = self.get_arithmetic_twap_price(
                step.pool_id,
                querier,
                next_input_denom,
                output_denom,
                start_time.clone(),
                end_time.clone(),
            )?;

            price = price.checked_mul(pool_price)?;
            next_input_denom = output_denom;
        }

        let price_with_slippage = price - price.mul(slippage.div(Uint128::new(100)));

        Ok(cosmwasm_std::Coin {
            denom: next_input_denom.to_owned(),
            amount: input_coin.amount.mul(price_with_slippage),
        })
    }

    pub fn get_path_output_denom(&self) -> String {
        self.0.last().cloned().unwrap().token_out_denom
    }

    fn get_arithmetic_twap_price(
        &self,
        pool_id: u64,
        querier: &QuerierWrapper,
        input_denom: &str,
        output_denom: &str,
        start_time: OsmosisTimestamp,
        end_time: OsmosisTimestamp,
    ) -> Result<Decimal, OsmosisRouterError> {
        let twap_price = TwapQuerier::new(querier)
            .arithmetic_twap(
                pool_id,
                input_denom.to_owned(),
                output_denom.to_owned(),
                Some(start_time),
                Some(end_time),
            )
            .map_err(|_| OsmosisRouterError::TwapPriceNotFound {})?
            .arithmetic_twap;

        let twap_price: Decimal = twap_price
            .parse()
            .map_err(|_| OsmosisRouterError::InvalidTwapPrice {})?;

        Ok(twap_price)
    }

    fn get_twap_window(&self, now: &Timestamp) -> (OsmosisTimestamp, OsmosisTimestamp) {
        let start_time = now.minus_seconds(TWAP_WINDOW);
        let start_time = OsmosisTimestamp {
            seconds: start_time.seconds() as i64,
            nanos: 0_i32,
        };

        let end_time = OsmosisTimestamp {
            seconds: now.seconds() as i64,
            nanos: 0_i32,
        };

        (start_time, end_time)
    }
}

#[cw_serde]
pub struct OsmosisSwapMsg {
    pub token_out_min_amount: String,
    pub path: Vec<SwapAmountInRoute>,
}

#[cw_serde]
pub struct OsmosisSwapReply {
    pub output_coin: cosmwasm_std::Coin,
}

#[cw_serde]
pub struct OsmosisSimulateSwapResponse {
    pub output_coin: cosmwasm_std::Coin,
}
