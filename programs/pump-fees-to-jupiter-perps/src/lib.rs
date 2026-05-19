use anchor_lang::prelude::*;

declare_id!("FjDSgr7sF8o3rwqnSp9m87xjEX18XxgWELhNVxwVkjDz");

pub mod constants;
pub mod contexts;
pub mod cpi;
pub mod errors;
pub mod events;
pub mod math;
pub mod state;

pub use constants::*;
pub use contexts::*;
pub use errors::*;
pub use events::*;
pub use math::*;
pub use state::*;

use crate::cpi::*;

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

        let deltas = claim_deltas(
            before_lamports,
            ctx.accounts.fee_owner.to_account_info().lamports(),
            before_tokens,
            token_amount(&ctx.accounts.fee_owner_quote_token_account)?,
            ctx.accounts.trade_config.quote_mint == WSOL_MINT,
        )?;
        params.validate_claim_amount(deltas.total)?;

        let size_usd_delta = create_jupiter_position_request_after_claim(
            ctx.accounts.jupiter_accounts(),
            ctx.accounts.wrap_wsol_accounts(),
            &ctx.accounts.trade_config,
            &params,
            deltas,
        )?;

        emit!(FeesClaimed {
            fee_owner: ctx.accounts.fee_owner.key(),
            quote_mint: ctx.accounts.trade_config.quote_mint,
            amount: deltas.total,
        });
        emit!(JupiterPositionRequestCreated {
            fee_owner: ctx.accounts.fee_owner.key(),
            position: ctx.accounts.position.key(),
            position_request: ctx.accounts.position_request.key(),
            collateral_token_delta: deltas.total,
            size_usd_delta,
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

        let deltas = claim_deltas(
            before_lamports,
            ctx.accounts.fee_owner.to_account_info().lamports(),
            before_tokens,
            token_amount(&ctx.accounts.fee_owner_quote_token_account)?,
            ctx.accounts.trade_config.quote_mint == WSOL_MINT,
        )?;
        params.validate_claim_amount(deltas.total)?;

        let size_usd_delta = create_jupiter_position_request_after_claim(
            ctx.accounts.jupiter_accounts(),
            ctx.accounts.wrap_wsol_accounts(),
            &ctx.accounts.trade_config,
            &params,
            deltas,
        )?;

        emit!(FeesClaimed {
            fee_owner: ctx.accounts.fee_owner.key(),
            quote_mint: ctx.accounts.trade_config.quote_mint,
            amount: deltas.total,
        });
        emit!(JupiterPositionRequestCreated {
            fee_owner: ctx.accounts.fee_owner.key(),
            position: ctx.accounts.position.key(),
            position_request: ctx.accounts.position_request.key(),
            collateral_token_delta: deltas.total,
            size_usd_delta,
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests;
