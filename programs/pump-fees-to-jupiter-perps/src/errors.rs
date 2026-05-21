use anchor_lang::prelude::*;

#[error_code]
pub enum PumpJupiterError {
    #[msg("Only the configured admin may perform this action")]
    Unauthorized,
    #[msg("Only WSOL and USDC quote mints are supported")]
    UnsupportedQuoteMint,
    #[msg("Leverage is outside the supported range")]
    InvalidLeverage,
    #[msg("Custody does not match the configured target market")]
    InvalidMarketCustody,
    #[msg("Collateral custody does not match the configured side")]
    InvalidCollateralCustody,
    #[msg("Trade config is paused")]
    ConfigPaused,
    #[msg("At least one claim source must be enabled")]
    NoClaimSource,
    #[msg("Quote price must be greater than zero")]
    InvalidQuotePrice,
    #[msg("Price slippage must be greater than zero")]
    InvalidPriceSlippage,
    #[msg("Claim bounds are invalid")]
    InvalidClaimBounds,
    #[msg("Claimed amount is below the configured minimum")]
    ClaimAmountTooSmall,
    #[msg("Claimed amount is above the configured maximum")]
    ClaimAmountTooLarge,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Token account data is invalid")]
    InvalidTokenAccount,
    #[msg("Calculated Jupiter position size is too small")]
    PositionSizeTooSmall,
}
