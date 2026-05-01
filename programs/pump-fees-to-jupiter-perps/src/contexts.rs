use anchor_lang::prelude::*;

use crate::constants::*;
use crate::cpi::{
    JupiterCreateIncreasePositionMarketRequestAccounts, PumpAmmCollectCoinCreatorFeeAccounts,
    PumpAmmTransferCreatorFeesToPumpV2Accounts, PumpCollectCreatorFeeV2Accounts,
    PumpDistributeCreatorFeesV2Accounts, WrapWsolAccounts,
};
use crate::errors::PumpJupiterError;
use crate::state::TradeConfig;

#[derive(Accounts)]
pub struct Ping {}

#[derive(Accounts)]
pub struct InitializeTradeConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    pub fee_owner: Signer<'info>,
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

#[derive(Accounts)]
pub struct ClaimSingleAndOpen<'info> {
    #[account(mut)]
    pub fee_owner: Signer<'info>,
    #[account(
        seeds = [TRADE_CONFIG_SEED, fee_owner.key().as_ref()],
        bump = trade_config.bump,
        constraint = trade_config.fee_owner == fee_owner.key() @ PumpJupiterError::Unauthorized
    )]
    pub trade_config: Account<'info, TradeConfig>,
    #[account(constraint = quote_mint.key() == trade_config.quote_mint @ PumpJupiterError::UnsupportedQuoteMint)]
    /// CHECK: Validated against trade_config.quote_mint.
    pub quote_mint: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Fee owner's ATA for the configured quote mint.
    pub fee_owner_quote_token_account: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Pump PDA [b"creator-vault", fee_owner].
    pub creator_vault: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: ATA for (creator_vault, quote_token_program, quote_mint).
    pub creator_vault_token_account: UncheckedAccount<'info>,
    #[account(address = SPL_TOKEN_PROGRAM_ID)]
    /// CHECK: Canonical SPL Token program for supported quote mints.
    pub quote_token_program: UncheckedAccount<'info>,
    #[account(address = ASSOCIATED_TOKEN_PROGRAM_ID)]
    /// CHECK: Canonical Associated Token program.
    pub associated_token_program: UncheckedAccount<'info>,
    #[account(address = SYSTEM_PROGRAM_ID)]
    /// CHECK: Canonical System program.
    pub system_program: UncheckedAccount<'info>,
    /// CHECK: Pump event authority PDA.
    pub pump_event_authority: UncheckedAccount<'info>,
    #[account(address = PUMP_PROGRAM_ID)]
    /// CHECK: Pump program.
    pub pump_program: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Pump AMM PDA [b"creator_vault", fee_owner].
    pub coin_creator_vault_authority: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: ATA for (coin_creator_vault_authority, quote_token_program, quote_mint).
    pub coin_creator_vault_ata: UncheckedAccount<'info>,
    /// CHECK: Pump AMM event authority PDA.
    pub pump_amm_event_authority: UncheckedAccount<'info>,
    #[account(address = PUMP_AMM_PROGRAM_ID)]
    /// CHECK: Pump AMM program.
    pub pump_amm_program: UncheckedAccount<'info>,
    /// CHECK: Jupiter perpetuals PDA.
    pub perpetuals: UncheckedAccount<'info>,
    #[account(address = JLP_POOL_ACCOUNT)]
    /// CHECK: Jupiter JLP pool account.
    pub pool: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Jupiter position PDA.
    pub position: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Jupiter position request PDA.
    pub position_request: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: ATA owned by the Jupiter position request PDA for input_mint.
    pub position_request_ata: UncheckedAccount<'info>,
    #[account(constraint = custody.key() == trade_config.custody @ PumpJupiterError::InvalidMarketCustody)]
    /// CHECK: Jupiter market custody from trade_config.
    pub custody: UncheckedAccount<'info>,
    #[account(constraint = collateral_custody.key() == trade_config.collateral_custody @ PumpJupiterError::InvalidCollateralCustody)]
    /// CHECK: Jupiter collateral custody from trade_config.
    pub collateral_custody: UncheckedAccount<'info>,
    /// CHECK: Optional Jupiter referral account placeholder.
    pub referral: UncheckedAccount<'info>,
    #[account(address = JUPITER_PERPETUALS_EVENT_AUTHORITY)]
    /// CHECK: Jupiter Perps event authority.
    pub jupiter_event_authority: UncheckedAccount<'info>,
    #[account(address = JUPITER_PERPETUALS_PROGRAM_ID)]
    /// CHECK: Jupiter Perps program.
    pub jupiter_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct ClaimSharedAndOpen<'info> {
    #[account(mut)]
    pub fee_owner: Signer<'info>,
    #[account(
        seeds = [TRADE_CONFIG_SEED, fee_owner.key().as_ref()],
        bump = trade_config.bump,
        constraint = trade_config.fee_owner == fee_owner.key() @ PumpJupiterError::Unauthorized
    )]
    pub trade_config: Account<'info, TradeConfig>,
    /// CHECK: Pump base token mint.
    pub mint: UncheckedAccount<'info>,
    /// CHECK: Pump bonding curve PDA for mint.
    pub bonding_curve: UncheckedAccount<'info>,
    /// CHECK: Pump Fees sharing config PDA for mint.
    pub sharing_config: UncheckedAccount<'info>,
    #[account(constraint = quote_mint.key() == trade_config.quote_mint @ PumpJupiterError::UnsupportedQuoteMint)]
    /// CHECK: Validated against trade_config.quote_mint.
    pub quote_mint: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Fee owner's ATA for the configured quote mint.
    pub fee_owner_quote_token_account: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Pump PDA [b"creator-vault", sharing_config].
    pub creator_vault: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: ATA for (creator_vault, quote_token_program, quote_mint).
    pub creator_vault_quote_token_account: UncheckedAccount<'info>,
    #[account(address = SPL_TOKEN_PROGRAM_ID)]
    /// CHECK: Canonical SPL Token program for supported quote mints.
    pub quote_token_program: UncheckedAccount<'info>,
    #[account(address = ASSOCIATED_TOKEN_PROGRAM_ID)]
    /// CHECK: Canonical Associated Token program.
    pub associated_token_program: UncheckedAccount<'info>,
    #[account(address = SYSTEM_PROGRAM_ID)]
    /// CHECK: Canonical System program.
    pub system_program: UncheckedAccount<'info>,
    /// CHECK: Pump event authority PDA.
    pub pump_event_authority: UncheckedAccount<'info>,
    #[account(address = PUMP_PROGRAM_ID)]
    /// CHECK: Pump program.
    pub pump_program: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Pump AMM PDA [b"creator_vault", sharing_config].
    pub coin_creator_vault_authority: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: ATA for (coin_creator_vault_authority, quote_token_program, quote_mint).
    pub coin_creator_vault_ata: UncheckedAccount<'info>,
    /// CHECK: Pump AMM event authority PDA.
    pub pump_amm_event_authority: UncheckedAccount<'info>,
    #[account(address = PUMP_AMM_PROGRAM_ID)]
    /// CHECK: Pump AMM program.
    pub pump_amm_program: UncheckedAccount<'info>,
    /// CHECK: Jupiter perpetuals PDA.
    pub perpetuals: UncheckedAccount<'info>,
    #[account(address = JLP_POOL_ACCOUNT)]
    /// CHECK: Jupiter JLP pool account.
    pub pool: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Jupiter position PDA.
    pub position: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Jupiter position request PDA.
    pub position_request: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: ATA owned by the Jupiter position request PDA for input_mint.
    pub position_request_ata: UncheckedAccount<'info>,
    #[account(constraint = custody.key() == trade_config.custody @ PumpJupiterError::InvalidMarketCustody)]
    /// CHECK: Jupiter market custody from trade_config.
    pub custody: UncheckedAccount<'info>,
    #[account(constraint = collateral_custody.key() == trade_config.collateral_custody @ PumpJupiterError::InvalidCollateralCustody)]
    /// CHECK: Jupiter collateral custody from trade_config.
    pub collateral_custody: UncheckedAccount<'info>,
    /// CHECK: Optional Jupiter referral account placeholder.
    pub referral: UncheckedAccount<'info>,
    #[account(address = JUPITER_PERPETUALS_EVENT_AUTHORITY)]
    /// CHECK: Jupiter Perps event authority.
    pub jupiter_event_authority: UncheckedAccount<'info>,
    #[account(address = JUPITER_PERPETUALS_PROGRAM_ID)]
    /// CHECK: Jupiter Perps program.
    pub jupiter_program: UncheckedAccount<'info>,
}

impl<'info> ClaimSingleAndOpen<'info> {
    pub fn validate_config(&self) -> Result<()> {
        require!(!self.trade_config.paused, PumpJupiterError::ConfigPaused);
        Ok(())
    }

    pub fn jupiter_accounts(&self) -> JupiterCreateIncreasePositionMarketRequestAccounts<'info> {
        JupiterCreateIncreasePositionMarketRequestAccounts {
            owner: self.fee_owner.to_account_info(),
            funding_account: self.fee_owner_quote_token_account.to_account_info(),
            perpetuals: self.perpetuals.to_account_info(),
            pool: self.pool.to_account_info(),
            position: self.position.to_account_info(),
            position_request: self.position_request.to_account_info(),
            position_request_ata: self.position_request_ata.to_account_info(),
            custody: self.custody.to_account_info(),
            collateral_custody: self.collateral_custody.to_account_info(),
            input_mint: self.quote_mint.to_account_info(),
            referral: self.referral.to_account_info(),
            token_program: self.quote_token_program.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
            event_authority: self.jupiter_event_authority.to_account_info(),
            program: self.jupiter_program.to_account_info(),
        }
    }

    pub fn wrap_wsol_accounts(&self) -> WrapWsolAccounts<'info> {
        WrapWsolAccounts {
            payer: self.fee_owner.to_account_info(),
            owner: self.fee_owner.to_account_info(),
            wsol_token_account: self.fee_owner_quote_token_account.to_account_info(),
            wsol_mint: self.quote_mint.to_account_info(),
            token_program: self.quote_token_program.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
        }
    }

    pub fn pump_collect_creator_fee_v2_accounts(&self) -> PumpCollectCreatorFeeV2Accounts<'info> {
        PumpCollectCreatorFeeV2Accounts {
            creator: self.fee_owner.to_account_info(),
            creator_token_account: self.fee_owner_quote_token_account.to_account_info(),
            creator_vault: self.creator_vault.to_account_info(),
            creator_vault_token_account: self.creator_vault_token_account.to_account_info(),
            quote_mint: self.quote_mint.to_account_info(),
            quote_token_program: self.quote_token_program.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
            event_authority: self.pump_event_authority.to_account_info(),
            program: self.pump_program.to_account_info(),
        }
    }

    pub fn pump_amm_collect_coin_creator_fee_accounts(
        &self,
    ) -> PumpAmmCollectCoinCreatorFeeAccounts<'info> {
        PumpAmmCollectCoinCreatorFeeAccounts {
            quote_mint: self.quote_mint.to_account_info(),
            quote_token_program: self.quote_token_program.to_account_info(),
            coin_creator: self.fee_owner.to_account_info(),
            coin_creator_vault_authority: self.coin_creator_vault_authority.to_account_info(),
            coin_creator_vault_ata: self.coin_creator_vault_ata.to_account_info(),
            coin_creator_token_account: self.fee_owner_quote_token_account.to_account_info(),
            event_authority: self.pump_amm_event_authority.to_account_info(),
            program: self.pump_amm_program.to_account_info(),
        }
    }
}

impl<'info> ClaimSharedAndOpen<'info> {
    pub fn validate_config(&self) -> Result<()> {
        require!(!self.trade_config.paused, PumpJupiterError::ConfigPaused);
        Ok(())
    }

    pub fn jupiter_accounts(&self) -> JupiterCreateIncreasePositionMarketRequestAccounts<'info> {
        JupiterCreateIncreasePositionMarketRequestAccounts {
            owner: self.fee_owner.to_account_info(),
            funding_account: self.fee_owner_quote_token_account.to_account_info(),
            perpetuals: self.perpetuals.to_account_info(),
            pool: self.pool.to_account_info(),
            position: self.position.to_account_info(),
            position_request: self.position_request.to_account_info(),
            position_request_ata: self.position_request_ata.to_account_info(),
            custody: self.custody.to_account_info(),
            collateral_custody: self.collateral_custody.to_account_info(),
            input_mint: self.quote_mint.to_account_info(),
            referral: self.referral.to_account_info(),
            token_program: self.quote_token_program.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
            event_authority: self.jupiter_event_authority.to_account_info(),
            program: self.jupiter_program.to_account_info(),
        }
    }

    pub fn wrap_wsol_accounts(&self) -> WrapWsolAccounts<'info> {
        WrapWsolAccounts {
            payer: self.fee_owner.to_account_info(),
            owner: self.fee_owner.to_account_info(),
            wsol_token_account: self.fee_owner_quote_token_account.to_account_info(),
            wsol_mint: self.quote_mint.to_account_info(),
            token_program: self.quote_token_program.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
        }
    }

    pub fn pump_amm_transfer_creator_fees_to_pump_v2_accounts(
        &self,
    ) -> PumpAmmTransferCreatorFeesToPumpV2Accounts<'info> {
        PumpAmmTransferCreatorFeesToPumpV2Accounts {
            payer: self.fee_owner.to_account_info(),
            quote_mint: self.quote_mint.to_account_info(),
            token_program: self.quote_token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
            coin_creator: self.sharing_config.to_account_info(),
            coin_creator_vault_authority: self.coin_creator_vault_authority.to_account_info(),
            coin_creator_vault_ata: self.coin_creator_vault_ata.to_account_info(),
            pump_creator_vault: self.creator_vault.to_account_info(),
            pump_creator_vault_ata: self.creator_vault_quote_token_account.to_account_info(),
            event_authority: self.pump_amm_event_authority.to_account_info(),
            program: self.pump_amm_program.to_account_info(),
        }
    }

    pub fn pump_distribute_creator_fees_v2_accounts(
        &self,
    ) -> PumpDistributeCreatorFeesV2Accounts<'info> {
        PumpDistributeCreatorFeesV2Accounts {
            payer: self.fee_owner.to_account_info(),
            mint: self.mint.to_account_info(),
            bonding_curve: self.bonding_curve.to_account_info(),
            sharing_config: self.sharing_config.to_account_info(),
            creator_vault: self.creator_vault.to_account_info(),
            system_program: self.system_program.to_account_info(),
            event_authority: self.pump_event_authority.to_account_info(),
            program: self.pump_program.to_account_info(),
            creator_vault_quote_token_account: self
                .creator_vault_quote_token_account
                .to_account_info(),
            quote_mint: self.quote_mint.to_account_info(),
            quote_token_program: self.quote_token_program.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
        }
    }
}
