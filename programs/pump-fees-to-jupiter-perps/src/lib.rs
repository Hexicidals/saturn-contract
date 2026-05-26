use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    program::invoke,
};

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

pub const PUMP_PROGRAM_ID: Pubkey = pubkey!("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");
pub const PUMP_AMM_PROGRAM_ID: Pubkey = pubkey!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");

pub const PUMP_COLLECT_CREATOR_FEE_V2_DISCRIMINATOR: [u8; 8] =
    [207, 17, 138, 242, 4, 34, 19, 56];
pub const PUMP_AMM_COLLECT_COIN_CREATOR_FEE_DISCRIMINATOR: [u8; 8] =
    [160, 57, 89, 42, 181, 139, 43, 66];
pub const PUMP_AMM_TRANSFER_CREATOR_FEES_TO_PUMP_V2_DISCRIMINATOR: [u8; 8] =
    [1, 33, 78, 185, 33, 67, 44, 92];
pub const PUMP_DISTRIBUTE_CREATOR_FEES_V2_DISCRIMINATOR: [u8; 8] =
    [255, 203, 19, 79, 244, 68, 8, 159];

const SPL_TOKEN_AMOUNT_OFFSET: usize = 64;
const SPL_TOKEN_AMOUNT_END: usize = 72;

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

    pub fn claim_single_and_open(
        ctx: Context<ClaimSingleAndOpen>,
        params: ClaimOpenParams,
        options: ClaimSingleOptions,
    ) -> Result<()> {
        ctx.accounts.validate_config()?;
        options.validate()?;
        params.validate(ctx.accounts.trade_config.max_leverage_bps)?;

        let before_lamports = ctx.accounts.fee_owner.to_account_info().lamports();
        let before_tokens = token_amount(&ctx.accounts.fee_owner_quote_token_account)?;

        if options.collect_bonding_curve {
            pump_collect_creator_fee_v2(PumpCollectCreatorFeeV2Accounts {
                creator: ctx.accounts.fee_owner.to_account_info(),
                creator_token_account: ctx.accounts.fee_owner_quote_token_account.to_account_info(),
                creator_vault: ctx.accounts.creator_vault.to_account_info(),
                creator_vault_token_account: ctx
                    .accounts
                    .creator_vault_token_account
                    .to_account_info(),
                quote_mint: ctx.accounts.quote_mint.to_account_info(),
                quote_token_program: ctx.accounts.quote_token_program.to_account_info(),
                associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                event_authority: ctx.accounts.pump_event_authority.to_account_info(),
                program: ctx.accounts.pump_program.to_account_info(),
            })?;
        }

        if options.collect_amm {
            pump_amm_collect_coin_creator_fee(PumpAmmCollectCoinCreatorFeeAccounts {
                quote_mint: ctx.accounts.quote_mint.to_account_info(),
                quote_token_program: ctx.accounts.quote_token_program.to_account_info(),
                coin_creator: ctx.accounts.fee_owner.to_account_info(),
                coin_creator_vault_authority: ctx
                    .accounts
                    .coin_creator_vault_authority
                    .to_account_info(),
                coin_creator_vault_ata: ctx.accounts.coin_creator_vault_ata.to_account_info(),
                coin_creator_token_account: ctx
                    .accounts
                    .fee_owner_quote_token_account
                    .to_account_info(),
                event_authority: ctx.accounts.pump_amm_event_authority.to_account_info(),
                program: ctx.accounts.pump_amm_program.to_account_info(),
            })?;
        }

        let claimed_amount = claimed_quote_delta(
            before_lamports,
            ctx.accounts.fee_owner.to_account_info().lamports(),
            before_tokens,
            token_amount(&ctx.accounts.fee_owner_quote_token_account)?,
            ctx.accounts.trade_config.quote_mint == WSOL_MINT,
        )?;
        params.validate_claim_amount(claimed_amount)?;

        emit!(FeesClaimed {
            fee_owner: ctx.accounts.fee_owner.key(),
            quote_mint: ctx.accounts.trade_config.quote_mint,
            amount: claimed_amount,
        });

        Ok(())
    }

    pub fn claim_shared_and_open<'info>(
        ctx: Context<'_, '_, '_, 'info, ClaimSharedAndOpen<'info>>,
        params: ClaimOpenParams,
        options: ClaimSharedOptions,
    ) -> Result<()> {
        ctx.accounts.validate_config()?;
        params.validate(ctx.accounts.trade_config.max_leverage_bps)?;

        let before_lamports = ctx.accounts.fee_owner.to_account_info().lamports();
        let before_tokens = token_amount(&ctx.accounts.fee_owner_quote_token_account)?;

        if options.sweep_amm {
            pump_amm_transfer_creator_fees_to_pump_v2(
                PumpAmmTransferCreatorFeesToPumpV2Accounts {
                    payer: ctx.accounts.fee_owner.to_account_info(),
                    quote_mint: ctx.accounts.quote_mint.to_account_info(),
                    token_program: ctx.accounts.quote_token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    associated_token_program: ctx
                        .accounts
                        .associated_token_program
                        .to_account_info(),
                    coin_creator: ctx.accounts.sharing_config.to_account_info(),
                    coin_creator_vault_authority: ctx
                        .accounts
                        .coin_creator_vault_authority
                        .to_account_info(),
                    coin_creator_vault_ata: ctx.accounts.coin_creator_vault_ata.to_account_info(),
                    pump_creator_vault: ctx.accounts.creator_vault.to_account_info(),
                    pump_creator_vault_ata: ctx
                        .accounts
                        .creator_vault_quote_token_account
                        .to_account_info(),
                    event_authority: ctx.accounts.pump_amm_event_authority.to_account_info(),
                    program: ctx.accounts.pump_amm_program.to_account_info(),
                },
            )?;
        }

        pump_distribute_creator_fees_v2(
            PumpDistributeCreatorFeesV2Accounts {
                payer: ctx.accounts.fee_owner.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                bonding_curve: ctx.accounts.bonding_curve.to_account_info(),
                sharing_config: ctx.accounts.sharing_config.to_account_info(),
                creator_vault: ctx.accounts.creator_vault.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                event_authority: ctx.accounts.pump_event_authority.to_account_info(),
                program: ctx.accounts.pump_program.to_account_info(),
                creator_vault_quote_token_account: ctx
                    .accounts
                    .creator_vault_quote_token_account
                    .to_account_info(),
                quote_mint: ctx.accounts.quote_mint.to_account_info(),
                quote_token_program: ctx.accounts.quote_token_program.to_account_info(),
                associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
            },
            options.initialize_shareholder_atas,
            ctx.remaining_accounts,
        )?;

        let claimed_amount = claimed_quote_delta(
            before_lamports,
            ctx.accounts.fee_owner.to_account_info().lamports(),
            before_tokens,
            token_amount(&ctx.accounts.fee_owner_quote_token_account)?,
            ctx.accounts.trade_config.quote_mint == WSOL_MINT,
        )?;
        params.validate_claim_amount(claimed_amount)?;

        emit!(FeesClaimed {
            fee_owner: ctx.accounts.fee_owner.key(),
            quote_mint: ctx.accounts.trade_config.quote_mint,
            amount: claimed_amount,
        });

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
    /// CHECK: SPL Token program for quote_mint.
    pub quote_token_program: UncheckedAccount<'info>,
    /// CHECK: Associated Token program.
    pub associated_token_program: UncheckedAccount<'info>,
    /// CHECK: System program.
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
    /// CHECK: SPL Token program for quote_mint.
    pub quote_token_program: UncheckedAccount<'info>,
    /// CHECK: Associated Token program.
    pub associated_token_program: UncheckedAccount<'info>,
    /// CHECK: System program.
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

impl<'info> ClaimSingleAndOpen<'info> {
    pub fn validate_config(&self) -> Result<()> {
        require!(!self.trade_config.paused, PumpJupiterError::ConfigPaused);
        Ok(())
    }
}

impl<'info> ClaimSharedAndOpen<'info> {
    pub fn validate_config(&self) -> Result<()> {
        require!(!self.trade_config.paused, PumpJupiterError::ConfigPaused);
        Ok(())
    }
}

pub struct PumpCollectCreatorFeeV2Accounts<'info> {
    pub creator: AccountInfo<'info>,
    pub creator_token_account: AccountInfo<'info>,
    pub creator_vault: AccountInfo<'info>,
    pub creator_vault_token_account: AccountInfo<'info>,
    pub quote_mint: AccountInfo<'info>,
    pub quote_token_program: AccountInfo<'info>,
    pub associated_token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub event_authority: AccountInfo<'info>,
    pub program: AccountInfo<'info>,
}

pub fn pump_collect_creator_fee_v2(accounts: PumpCollectCreatorFeeV2Accounts<'_>) -> Result<()> {
    let ix = Instruction {
        program_id: PUMP_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*accounts.creator.key, false),
            AccountMeta::new(*accounts.creator_token_account.key, false),
            AccountMeta::new(*accounts.creator_vault.key, false),
            AccountMeta::new(*accounts.creator_vault_token_account.key, false),
            AccountMeta::new_readonly(*accounts.quote_mint.key, false),
            AccountMeta::new_readonly(*accounts.quote_token_program.key, false),
            AccountMeta::new_readonly(*accounts.associated_token_program.key, false),
            AccountMeta::new_readonly(*accounts.system_program.key, false),
            AccountMeta::new_readonly(*accounts.event_authority.key, false),
            AccountMeta::new_readonly(*accounts.program.key, false),
        ],
        data: PUMP_COLLECT_CREATOR_FEE_V2_DISCRIMINATOR.to_vec(),
    };

    invoke(
        &ix,
        &[
            accounts.creator,
            accounts.creator_token_account,
            accounts.creator_vault,
            accounts.creator_vault_token_account,
            accounts.quote_mint,
            accounts.quote_token_program,
            accounts.associated_token_program,
            accounts.system_program,
            accounts.event_authority,
            accounts.program,
        ],
    )?;

    Ok(())
}

pub struct PumpAmmCollectCoinCreatorFeeAccounts<'info> {
    pub quote_mint: AccountInfo<'info>,
    pub quote_token_program: AccountInfo<'info>,
    pub coin_creator: AccountInfo<'info>,
    pub coin_creator_vault_authority: AccountInfo<'info>,
    pub coin_creator_vault_ata: AccountInfo<'info>,
    pub coin_creator_token_account: AccountInfo<'info>,
    pub event_authority: AccountInfo<'info>,
    pub program: AccountInfo<'info>,
}

pub fn pump_amm_collect_coin_creator_fee(
    accounts: PumpAmmCollectCoinCreatorFeeAccounts<'_>,
) -> Result<()> {
    let ix = Instruction {
        program_id: PUMP_AMM_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(*accounts.quote_mint.key, false),
            AccountMeta::new_readonly(*accounts.quote_token_program.key, false),
            AccountMeta::new_readonly(*accounts.coin_creator.key, false),
            AccountMeta::new_readonly(*accounts.coin_creator_vault_authority.key, false),
            AccountMeta::new(*accounts.coin_creator_vault_ata.key, false),
            AccountMeta::new(*accounts.coin_creator_token_account.key, false),
            AccountMeta::new_readonly(*accounts.event_authority.key, false),
            AccountMeta::new_readonly(*accounts.program.key, false),
        ],
        data: PUMP_AMM_COLLECT_COIN_CREATOR_FEE_DISCRIMINATOR.to_vec(),
    };

    invoke(
        &ix,
        &[
            accounts.quote_mint,
            accounts.quote_token_program,
            accounts.coin_creator,
            accounts.coin_creator_vault_authority,
            accounts.coin_creator_vault_ata,
            accounts.coin_creator_token_account,
            accounts.event_authority,
            accounts.program,
        ],
    )?;

    Ok(())
}

pub struct PumpAmmTransferCreatorFeesToPumpV2Accounts<'info> {
    pub payer: AccountInfo<'info>,
    pub quote_mint: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub associated_token_program: AccountInfo<'info>,
    pub coin_creator: AccountInfo<'info>,
    pub coin_creator_vault_authority: AccountInfo<'info>,
    pub coin_creator_vault_ata: AccountInfo<'info>,
    pub pump_creator_vault: AccountInfo<'info>,
    pub pump_creator_vault_ata: AccountInfo<'info>,
    pub event_authority: AccountInfo<'info>,
    pub program: AccountInfo<'info>,
}

pub fn pump_amm_transfer_creator_fees_to_pump_v2(
    accounts: PumpAmmTransferCreatorFeesToPumpV2Accounts<'_>,
) -> Result<()> {
    let ix = Instruction {
        program_id: PUMP_AMM_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*accounts.payer.key, true),
            AccountMeta::new_readonly(*accounts.quote_mint.key, false),
            AccountMeta::new_readonly(*accounts.token_program.key, false),
            AccountMeta::new_readonly(*accounts.system_program.key, false),
            AccountMeta::new_readonly(*accounts.associated_token_program.key, false),
            AccountMeta::new_readonly(*accounts.coin_creator.key, false),
            AccountMeta::new(*accounts.coin_creator_vault_authority.key, false),
            AccountMeta::new(*accounts.coin_creator_vault_ata.key, false),
            AccountMeta::new(*accounts.pump_creator_vault.key, false),
            AccountMeta::new(*accounts.pump_creator_vault_ata.key, false),
            AccountMeta::new_readonly(*accounts.event_authority.key, false),
            AccountMeta::new_readonly(*accounts.program.key, false),
        ],
        data: PUMP_AMM_TRANSFER_CREATOR_FEES_TO_PUMP_V2_DISCRIMINATOR.to_vec(),
    };

    invoke(
        &ix,
        &[
            accounts.payer,
            accounts.quote_mint,
            accounts.token_program,
            accounts.system_program,
            accounts.associated_token_program,
            accounts.coin_creator,
            accounts.coin_creator_vault_authority,
            accounts.coin_creator_vault_ata,
            accounts.pump_creator_vault,
            accounts.pump_creator_vault_ata,
            accounts.event_authority,
            accounts.program,
        ],
    )?;

    Ok(())
}

pub struct PumpDistributeCreatorFeesV2Accounts<'info> {
    pub payer: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub bonding_curve: AccountInfo<'info>,
    pub sharing_config: AccountInfo<'info>,
    pub creator_vault: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub event_authority: AccountInfo<'info>,
    pub program: AccountInfo<'info>,
    pub creator_vault_quote_token_account: AccountInfo<'info>,
    pub quote_mint: AccountInfo<'info>,
    pub quote_token_program: AccountInfo<'info>,
    pub associated_token_program: AccountInfo<'info>,
}

pub fn pump_distribute_creator_fees_v2<'info>(
    accounts: PumpDistributeCreatorFeesV2Accounts<'info>,
    initialize_ata: bool,
    remaining_accounts: &[AccountInfo<'info>],
) -> Result<()> {
    let mut data = PUMP_DISTRIBUTE_CREATOR_FEES_V2_DISCRIMINATOR.to_vec();
    data.push(u8::from(initialize_ata));

    let mut metas = vec![
        AccountMeta::new(*accounts.payer.key, true),
        AccountMeta::new_readonly(*accounts.mint.key, false),
        AccountMeta::new_readonly(*accounts.bonding_curve.key, false),
        AccountMeta::new_readonly(*accounts.sharing_config.key, false),
        AccountMeta::new(*accounts.creator_vault.key, false),
        AccountMeta::new_readonly(*accounts.system_program.key, false),
        AccountMeta::new_readonly(*accounts.event_authority.key, false),
        AccountMeta::new_readonly(*accounts.program.key, false),
        AccountMeta::new(*accounts.creator_vault_quote_token_account.key, false),
        AccountMeta::new_readonly(*accounts.quote_mint.key, false),
        AccountMeta::new_readonly(*accounts.quote_token_program.key, false),
        AccountMeta::new_readonly(*accounts.associated_token_program.key, false),
    ];
    metas.extend(remaining_account_metas(remaining_accounts));

    let ix = Instruction {
        program_id: PUMP_PROGRAM_ID,
        accounts: metas,
        data,
    };

    let mut account_infos = vec![
        accounts.payer,
        accounts.mint,
        accounts.bonding_curve,
        accounts.sharing_config,
        accounts.creator_vault,
        accounts.system_program,
        accounts.event_authority,
        accounts.program,
        accounts.creator_vault_quote_token_account,
        accounts.quote_mint,
        accounts.quote_token_program,
        accounts.associated_token_program,
    ];
    account_infos.extend_from_slice(remaining_accounts);

    invoke(&ix, &account_infos)?;

    Ok(())
}

pub fn remaining_account_metas(accounts: &[AccountInfo<'_>]) -> Vec<AccountMeta> {
    accounts
        .iter()
        .map(|account| {
            if account.is_writable {
                AccountMeta::new(*account.key, account.is_signer)
            } else {
                AccountMeta::new_readonly(*account.key, account.is_signer)
            }
        })
        .collect()
}

pub fn token_amount(account: &UncheckedAccount<'_>) -> Result<u64> {
    let data = account.try_borrow_data()?;
    if data.is_empty() {
        return Ok(0);
    }
    require!(
        data.len() >= SPL_TOKEN_AMOUNT_END,
        PumpJupiterError::InvalidTokenAccount
    );

    let amount_bytes: [u8; 8] = data[SPL_TOKEN_AMOUNT_OFFSET..SPL_TOKEN_AMOUNT_END]
        .try_into()
        .map_err(|_| error!(PumpJupiterError::InvalidTokenAccount))?;
    Ok(u64::from_le_bytes(amount_bytes))
}

pub fn claimed_quote_delta(
    before_lamports: u64,
    after_lamports: u64,
    before_tokens: u64,
    after_tokens: u64,
    include_lamports: bool,
) -> Result<u64> {
    let token_delta = after_tokens.saturating_sub(before_tokens);
    let lamport_delta = if include_lamports {
        after_lamports.saturating_sub(before_lamports)
    } else {
        0
    };

    token_delta
        .checked_add(lamport_delta)
        .ok_or_else(|| error!(PumpJupiterError::MathOverflow))
}

#[event]
pub struct FeesClaimed {
    pub fee_owner: Pubkey,
    pub quote_mint: Pubkey,
    pub amount: u64,
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
    #[msg("Trade config is paused")]
    ConfigPaused,
    #[msg("At least one claim source must be enabled")]
    NoClaimSource,
    #[msg("Quote price must be greater than zero")]
    InvalidQuotePrice,
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
        let params = ClaimOpenParams {
            leverage_bps: 20_000,
            quote_price_usd_e6: 1_000_000,
            price_slippage_usd_e6: 1_000_000,
            jupiter_minimum_out: 0,
            position_request_counter: 1,
            min_claim_amount: 10,
            max_claim_amount: 20,
        };

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
    fn encodes_distribute_creator_fees_v2_data() {
        let mut data = PUMP_DISTRIBUTE_CREATOR_FEES_V2_DISCRIMINATOR.to_vec();
        data.push(1);

        assert_eq!(data, vec![255, 203, 19, 79, 244, 68, 8, 159, 1]);
    }
}
