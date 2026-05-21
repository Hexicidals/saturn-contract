use anchor_lang::prelude::*;

use crate::constants::*;
use crate::errors::PumpJupiterError;

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

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClaimOpenParams {
    pub leverage_bps: u64,
    pub quote_price_usd_e6: u64,
    pub price_slippage_usd_e6: u64,
    pub jupiter_minimum_out: u64,
    pub position_request_counter: u64,
    pub min_claim_amount: u64,
    pub max_claim_amount: u64,
}

impl ClaimOpenParams {
    pub fn validate(&self, max_leverage_bps: u64) -> Result<()> {
        require!(
            (MIN_LEVERAGE_BPS..=max_leverage_bps).contains(&self.leverage_bps),
            PumpJupiterError::InvalidLeverage
        );
        require!(
            self.quote_price_usd_e6 > 0,
            PumpJupiterError::InvalidQuotePrice
        );
        require!(
            self.price_slippage_usd_e6 > 0,
            PumpJupiterError::InvalidPriceSlippage
        );
        require!(
            self.max_claim_amount == 0 || self.max_claim_amount >= self.min_claim_amount,
            PumpJupiterError::InvalidClaimBounds
        );
        Ok(())
    }

    pub fn validate_claim_amount(&self, claimed_amount: u64) -> Result<()> {
        require!(
            claimed_amount >= self.min_claim_amount,
            PumpJupiterError::ClaimAmountTooSmall
        );
        if self.max_claim_amount > 0 {
            require!(
                claimed_amount <= self.max_claim_amount,
                PumpJupiterError::ClaimAmountTooLarge
            );
        }
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClaimSingleOptions {
    pub collect_bonding_curve: bool,
    pub collect_amm: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClaimSharedOptions {
    pub sweep_amm: bool,
    pub initialize_shareholder_atas: bool,
}

impl ClaimSingleOptions {
    pub fn validate(&self) -> Result<()> {
        require!(
            self.collect_bonding_curve || self.collect_amm,
            PumpJupiterError::NoClaimSource
        );
        Ok(())
    }
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

impl PositionSide {
    pub fn jupiter_side_discriminator(self) -> u8 {
        match self {
            PositionSide::Long => 1,
            PositionSide::Short => 2,
        }
    }
}
