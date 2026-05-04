use super::*;

fn base_params() -> TradeConfigParams {
    TradeConfigParams {
        quote_mint: USDC_MINT,
        target_market: TargetMarket::Sol,
        side: PositionSide::Short,
        custody: JUPITER_SOL_CUSTODY,
        collateral_custody: JUPITER_USDC_CUSTODY,
        max_leverage_bps: 50_000,
    }
}

fn claim_params() -> ClaimOpenParams {
    ClaimOpenParams {
        leverage_bps: 20_000,
        quote_price_usd_e6: 1_000_000,
        price_slippage_usd_e6: 1_000_000,
        jupiter_minimum_out: 0,
        position_request_counter: 1,
        min_claim_amount: 10,
        max_claim_amount: 20,
    }
}

#[test]
fn validates_supported_short_config() {
    assert!(base_params().validate().is_ok());
}

#[test]
fn validates_long_collateral_matches_market() {
    let mut params = base_params();
    params.side = PositionSide::Long;
    params.collateral_custody = JUPITER_SOL_CUSTODY;

    assert!(params.validate().is_ok());
}

#[test]
fn detects_native_quote_configs() {
    let mut config = TradeConfig {
        admin: Pubkey::new_unique(),
        fee_owner: Pubkey::new_unique(),
        quote_mint: USDC_MINT,
        target_market: TargetMarket::Sol,
        side: PositionSide::Short,
        custody: JUPITER_SOL_CUSTODY,
        collateral_custody: JUPITER_USDC_CUSTODY,
        max_leverage_bps: 50_000,
        paused: false,
        bump: 255,
    };
    assert!(!config.uses_native_quote());

    let mut params = base_params();
    params.quote_mint = WSOL_MINT;
    config.update(params, false);
    assert!(config.uses_native_quote());
}

#[test]
fn rejects_unsupported_quote() {
    let mut params = base_params();
    params.quote_mint = Pubkey::new_unique();

    assert!(matches!(
        params.validate(),
        Err(error) if error == PumpJupiterError::UnsupportedQuoteMint.into()
    ));
}

#[test]
fn rejects_invalid_market_custody() {
    let mut params = base_params();
    params.custody = JUPITER_ETH_CUSTODY;

    assert!(matches!(
        params.validate(),
        Err(error) if error == PumpJupiterError::InvalidMarketCustody.into()
    ));
}

#[test]
fn calculates_wsol_claim_delta_from_lamports_and_tokens() {
    let amount = claimed_quote_delta(100, 175, 20, 45, true).unwrap();
    assert_eq!(amount, 100);
}

#[test]
fn excludes_lamports_for_non_native_quote_delta() {
    let amount = claimed_quote_delta(100, 175, 20, 45, false).unwrap();
    assert_eq!(amount, 25);
}

#[test]
fn validates_claim_bounds() {
    let params = claim_params();

    assert!(params.validate(50_000).is_ok());
    assert!(params.validate_claim_amount(15).is_ok());
    assert!(matches!(
        params.validate_claim_amount(9),
        Err(error) if error == PumpJupiterError::ClaimAmountTooSmall.into()
    ));
    assert!(matches!(
    params.validate_claim_amount(21),
        Err(error) if error == PumpJupiterError::ClaimAmountTooLarge.into()
    ));
}

#[test]
fn rejects_invalid_claim_open_params() {
    let mut params = claim_params();
    params.leverage_bps = MIN_LEVERAGE_BPS - 1;
    assert!(matches!(
        params.validate(50_000),
        Err(error) if error == PumpJupiterError::InvalidLeverage.into()
    ));

    params = claim_params();
    params.quote_price_usd_e6 = 0;
    assert!(matches!(
        params.validate(50_000),
        Err(error) if error == PumpJupiterError::InvalidQuotePrice.into()
    ));

    params = claim_params();
    params.price_slippage_usd_e6 = 0;
    assert!(matches!(
        params.validate(50_000),
        Err(error) if error == PumpJupiterError::InvalidPriceSlippage.into()
    ));

    params = claim_params();
    params.min_claim_amount = 20;
    params.max_claim_amount = 10;
    assert!(matches!(
        params.validate(50_000),
        Err(error) if error == PumpJupiterError::InvalidClaimBounds.into()
    ));
}

#[test]
fn converts_zero_jupiter_minimum_out_to_none() {
    let mut params = claim_params();
    params.min_claim_amount = 0;
    params.max_claim_amount = 0;

    assert_eq!(params.jupiter_minimum_out(), None);

    params.jupiter_minimum_out = 99;
    assert_eq!(params.jupiter_minimum_out(), Some(99));
}

#[test]
fn encodes_distribute_creator_fees_v2_data() {
    let data = encode_pump_distribute_creator_fees_v2(true);

    assert_eq!(data, vec![255, 203, 19, 79, 244, 68, 8, 159, 1]);
    assert_eq!(
        encode_pump_distribute_creator_fees_v2(false),
        vec![255, 203, 19, 79, 244, 68, 8, 159, 0]
    );
}

#[test]
fn calculates_usdc_position_size() {
    let size = position_size_usd_e6(10_000_000, USDC_MINT, 1_000_000, 30_000).unwrap();
    assert_eq!(size, 30_000_000);
}

#[test]
fn calculates_wsol_position_size() {
    let size = position_size_usd_e6(1_500_000_000, WSOL_MINT, 200_000_000, 20_000).unwrap();
    assert_eq!(size, 600_000_000);
}

#[test]
fn encodes_jupiter_create_increase_request_data() {
    let data = encode_jupiter_create_increase_position_market_request(
        JupiterCreateIncreasePositionMarketRequestParams {
            size_usd_delta: 30_000_000,
            collateral_token_delta: 10_000_000,
            side: PositionSide::Short,
            price_slippage_usd_e6: 100_000_000,
            jupiter_minimum_out: Some(9_900_000),
            counter: 42,
        },
    );

    assert_eq!(
        &data[0..8],
        &JUPITER_CREATE_INCREASE_POSITION_MARKET_REQUEST_DISCRIMINATOR
    );
    assert_eq!(
        data[JUPITER_INCREASE_REQUEST_SIDE_OFFSET],
        PositionSide::Short.jupiter_side_discriminator()
    );
    assert_eq!(data[JUPITER_INCREASE_REQUEST_MINIMUM_OUT_OPTION_OFFSET], 1);
    assert_eq!(data.len(), JUPITER_INCREASE_REQUEST_WITH_MINIMUM_OUT_LEN);
}

#[test]
fn encodes_jupiter_create_increase_request_without_minimum_out() {
    let data = encode_jupiter_create_increase_position_market_request(
        JupiterCreateIncreasePositionMarketRequestParams {
            size_usd_delta: 30_000_000,
            collateral_token_delta: 10_000_000,
            side: PositionSide::Long,
            price_slippage_usd_e6: 100_000_000,
            jupiter_minimum_out: None,
            counter: 42,
        },
    );

    assert_eq!(data[JUPITER_INCREASE_REQUEST_MINIMUM_OUT_OPTION_OFFSET], 0);
    assert_eq!(data.len(), JUPITER_INCREASE_REQUEST_WITHOUT_MINIMUM_OUT_LEN);
}
