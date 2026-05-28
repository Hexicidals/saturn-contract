use anchor_lang::prelude::*;

declare_id!("F3WS96pF3QCpovpw4hcEr1cvmroNYuEZqKYg9G8n9Sw1");

pub const TRADE_CONFIG_SEED: &[u8] = b"trade-config";
pub const MIN_LEVERAGE_BPS: u64 = 10_000;
pub const MAX_LEVERAGE_BPS: u64 = 2_500_000;

pub const WSOL_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
pub const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

pub const JUPITER_SOL_CUSTODY: Pubkey = pubkey!("7xS2gz2bTp3fwCC7knJvUWTEU9Tycczu6VhJYKgi1wdz");
pub const JUPITER_ETH_CUSTODY: Pubkey = pubkey!("AQCGyheWPLeo6Qp9WpYS9m3Qj479t7R636N9ey1rEjEn");
pub const JUPITER_BTC_CUSTODY: Pubkey = pubkey!("5Pv3gM9JrFFH883SWAhvJC9RPYmo8UNxuFtv5bMMALkm");
pub const JUPITER_USDC_CUSTODY: Pubkey = pubkey!("G18jKKXQwBbrHeiK3C9MRXhkHsLHf7XgCSisykV46EZa");

#[program]
pub mod pump_fees_to_jupiter_perps {
    use super::*;

    pub fn ping(_ctx: Context<Ping>) -> Result<()> {
        Ok(())
    }

    pub fn initialize_trade_config(
        ctx: Context<InitializeTradeConfig>,
        params: TradeConfigParams,
    ) -> Result<()> {
        params.validate()?;

        let config = &mut ctx.accounts.trade_config;
        config.admin = ctx.accounts.admin.key();
        config.fee_owner = ctx.accounts.fee_owner.key();
        config.quote_mint = params.quote_mint;
        config.target_market = params.target_market;
        config.side = params.side;
        config.custody = params.custody;
        config.collateral_custody = params.collateral_custody;
        config.max_leverage_bps = params.max_leverage_bps;
        config.paused = false;
        config.bump = ctx.bumps.trade_config;

        Ok(())
    }

    pub fn update_trade_config(
        ctx: Context<UpdateTradeConfig>,
        params: TradeConfigParams,
        paused: bool,
    ) -> Result<()> {
        params.validate()?;

        let config = &mut ctx.accounts.trade_config;
        config.quote_mint = params.quote_mint;
        config.target_market = params.target_market;
        config.side = params.side;
        config.custody = params.custody;
        config.collateral_custody = params.collateral_custody;
        config.max_leverage_bps = params.max_leverage_bps;
        config.paused = paused;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Ping {}

#[derive(Accounts)]
pub struct InitializeTradeConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: Stored as the wallet whose claimed fees can be routed into Jupiter.
    pub fee_owner: UncheckedAccount<'info>,
    #[account(
        init,
        payer = admin,
        space = TradeConfig::SPACE,
        seeds = [TRADE_CONFIG_SEED, fee_owner.key().as_ref()],
        bump
    )]
    pub trade_config: Account<'info, TradeConfig>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateTradeConfig<'info> {
    pub admin: Signer<'info>,
    #[account(
        mut,
        has_one = admin @ PumpJupiterError::Unauthorized,
        seeds = [TRADE_CONFIG_SEED, trade_config.fee_owner.as_ref()],
        bump = trade_config.bump
    )]
    pub trade_config: Account<'info, TradeConfig>,
}

#[account]
#[derive(Debug, PartialEq, Eq)]
pub struct TradeConfig {
    pub admin: Pubkey,
    pub fee_owner: Pubkey,
    pub quote_mint: Pubkey,
    pub target_market: TargetMarket,
    pub side: PositionSide,
    pub custody: Pubkey,
    pub collateral_custody: Pubkey,
    pub max_leverage_bps: u64,
    pub paused: bool,
    pub bump: u8,
}

impl TradeConfig {
    pub const SPACE: usize = 8 + 32 + 32 + 32 + 1 + 1 + 32 + 32 + 8 + 1 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetMarket {
    Sol,
    Eth,
    Btc,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PositionSide {
    Long,
    Short,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TradeConfigParams {
    pub quote_mint: Pubkey,
    pub target_market: TargetMarket,
    pub side: PositionSide,
    pub custody: Pubkey,
    pub collateral_custody: Pubkey,
    pub max_leverage_bps: u64,
}

impl TradeConfigParams {
    pub fn validate(&self) -> Result<()> {
        require!(
            self.quote_mint == WSOL_MINT || self.quote_mint == USDC_MINT,
            PumpJupiterError::UnsupportedQuoteMint
        );
        require!(
            (MIN_LEVERAGE_BPS..=MAX_LEVERAGE_BPS).contains(&self.max_leverage_bps),
            PumpJupiterError::InvalidLeverage
        );

        let expected_custody = self.target_market.custody();
        require!(
            self.custody == expected_custody,
            PumpJupiterError::InvalidMarketCustody
        );

        let expected_collateral = match self.side {
            PositionSide::Long => expected_custody,
            PositionSide::Short => JUPITER_USDC_CUSTODY,
        };
        require!(
            self.collateral_custody == expected_collateral,
            PumpJupiterError::InvalidCollateralCustody
        );

        Ok(())
    }
}

impl TargetMarket {
    pub fn custody(self) -> Pubkey {
        match self {
            TargetMarket::Sol => JUPITER_SOL_CUSTODY,
            TargetMarket::Eth => JUPITER_ETH_CUSTODY,
            TargetMarket::Btc => JUPITER_BTC_CUSTODY,
        }
    }
}

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
}

#[cfg(test)]
mod tests {
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
}
