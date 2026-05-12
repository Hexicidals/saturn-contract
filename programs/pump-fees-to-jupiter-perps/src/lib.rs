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

        ctx.accounts.trade_config.initialize(
            ctx.accounts.admin.key(),
            ctx.accounts.fee_owner.key(),
            params,
            ctx.bumps.trade_config,
        );

        Ok(())
    }

    pub fn update_trade_config(
        ctx: Context<UpdateTradeConfig>,
        params: TradeConfigParams,
        paused: bool,
    ) -> Result<()> {
        params.validate()?;

        ctx.accounts.trade_config.update(params, paused);

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

        let before_claim = ClaimBalanceSnapshot::capture(
            &ctx.accounts.fee_owner.to_account_info(),
            &ctx.accounts.fee_owner_quote_token_account,
        )?;

        if options.collect_bonding_curve {
            pump_collect_creator_fee_v2(ctx.accounts.pump_collect_creator_fee_v2_accounts())?;
        }

        if options.collect_amm {
            pump_amm_collect_coin_creator_fee(
                ctx.accounts.pump_amm_collect_coin_creator_fee_accounts(),
            )?;
        }

        let deltas = before_claim.deltas(
            &ctx.accounts.fee_owner.to_account_info(),
            &ctx.accounts.fee_owner_quote_token_account,
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

        emit_claim_open_events(ClaimOpenEventFields {
            fee_owner: ctx.accounts.fee_owner.key(),
            quote_mint: ctx.accounts.trade_config.quote_mint,
            position: ctx.accounts.position.key(),
            position_request: ctx.accounts.position_request.key(),
            deltas,
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

        let before_claim = ClaimBalanceSnapshot::capture(
            &ctx.accounts.fee_owner.to_account_info(),
            &ctx.accounts.fee_owner_quote_token_account,
        )?;

        if options.sweep_amm {
            pump_amm_transfer_creator_fees_to_pump_v2(
                ctx.accounts
                    .pump_amm_transfer_creator_fees_to_pump_v2_accounts(),
            )?;
        }

        pump_distribute_creator_fees_v2(
            ctx.accounts.pump_distribute_creator_fees_v2_accounts(),
            options.initialize_shareholder_atas,
            ctx.remaining_accounts,
        )?;

        let deltas = before_claim.deltas(
            &ctx.accounts.fee_owner.to_account_info(),
            &ctx.accounts.fee_owner_quote_token_account,
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

        emit_claim_open_events(ClaimOpenEventFields {
            fee_owner: ctx.accounts.fee_owner.key(),
            quote_mint: ctx.accounts.trade_config.quote_mint,
            position: ctx.accounts.position.key(),
            position_request: ctx.accounts.position_request.key(),
            deltas,
            size_usd_delta,
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests;
