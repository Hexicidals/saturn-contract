use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    program::invoke,
    system_instruction,
};

use crate::constants::*;
use crate::math::{position_size_usd_e6, ClaimDeltas};
use crate::state::{ClaimOpenParams, PositionSide, TradeConfig};

pub struct JupiterCreateIncreasePositionMarketRequestAccounts<'info> {
    pub owner: AccountInfo<'info>,
    pub funding_account: AccountInfo<'info>,
    pub perpetuals: AccountInfo<'info>,
    pub pool: AccountInfo<'info>,
    pub position: AccountInfo<'info>,
    pub position_request: AccountInfo<'info>,
    pub position_request_ata: AccountInfo<'info>,
    pub custody: AccountInfo<'info>,
    pub collateral_custody: AccountInfo<'info>,
    pub input_mint: AccountInfo<'info>,
    pub referral: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub associated_token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub event_authority: AccountInfo<'info>,
    pub program: AccountInfo<'info>,
}

pub struct WrapWsolAccounts<'info> {
    pub payer: AccountInfo<'info>,
    pub owner: AccountInfo<'info>,
    pub wsol_token_account: AccountInfo<'info>,
    pub wsol_mint: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub associated_token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
}

pub fn create_jupiter_position_request_after_claim<'info>(
    jupiter_accounts: JupiterCreateIncreasePositionMarketRequestAccounts<'info>,
    wrap_accounts: WrapWsolAccounts<'info>,
    config: &TradeConfig,
    params: &ClaimOpenParams,
    deltas: ClaimDeltas,
) -> Result<u64> {
    if config.quote_mint == WSOL_MINT && deltas.lamports > 0 {
        wrap_lamports_to_wsol(wrap_accounts, deltas.lamports)?;
    }

    let size_usd_delta = position_size_usd_e6(
        deltas.total,
        config.quote_mint,
        params.quote_price_usd_e6,
        params.leverage_bps,
    )?;

    jupiter_create_increase_position_market_request(
        jupiter_accounts,
        JupiterCreateIncreasePositionMarketRequestParams {
            size_usd_delta,
            collateral_token_delta: deltas.total,
            side: config.side,
            price_slippage_usd_e6: params.price_slippage_usd_e6,
            jupiter_minimum_out: if params.jupiter_minimum_out == 0 {
                None
            } else {
                Some(params.jupiter_minimum_out)
            },
            counter: params.position_request_counter,
        },
    )?;

    Ok(size_usd_delta)
}

pub fn wrap_lamports_to_wsol(accounts: WrapWsolAccounts<'_>, lamports: u64) -> Result<()> {
    let create_ata_ix = Instruction {
        program_id: *accounts.associated_token_program.key,
        accounts: vec![
            AccountMeta::new(*accounts.payer.key, true),
            AccountMeta::new(*accounts.wsol_token_account.key, false),
            AccountMeta::new_readonly(*accounts.owner.key, false),
            AccountMeta::new_readonly(*accounts.wsol_mint.key, false),
            AccountMeta::new_readonly(*accounts.system_program.key, false),
            AccountMeta::new_readonly(*accounts.token_program.key, false),
        ],
        data: vec![ASSOCIATED_TOKEN_CREATE_IDEMPOTENT_DISCRIMINATOR],
    };
    invoke(
        &create_ata_ix,
        &[
            accounts.payer.clone(),
            accounts.wsol_token_account.clone(),
            accounts.owner.clone(),
            accounts.wsol_mint.clone(),
            accounts.system_program.clone(),
            accounts.token_program.clone(),
            accounts.associated_token_program.clone(),
        ],
    )?;

    let transfer_ix = system_instruction::transfer(
        accounts.owner.key,
        accounts.wsol_token_account.key,
        lamports,
    );
    invoke(
        &transfer_ix,
        &[
            accounts.owner.clone(),
            accounts.wsol_token_account.clone(),
            accounts.system_program.clone(),
        ],
    )?;

    let sync_native_ix = Instruction {
        program_id: *accounts.token_program.key,
        accounts: vec![AccountMeta::new(*accounts.wsol_token_account.key, false)],
        data: vec![SPL_TOKEN_SYNC_NATIVE_DISCRIMINATOR],
    };
    invoke(
        &sync_native_ix,
        &[accounts.wsol_token_account, accounts.token_program],
    )?;

    Ok(())
}

pub struct JupiterCreateIncreasePositionMarketRequestParams {
    pub size_usd_delta: u64,
    pub collateral_token_delta: u64,
    pub side: PositionSide,
    pub price_slippage_usd_e6: u64,
    pub jupiter_minimum_out: Option<u64>,
    pub counter: u64,
}

pub fn jupiter_create_increase_position_market_request(
    accounts: JupiterCreateIncreasePositionMarketRequestAccounts<'_>,
    params: JupiterCreateIncreasePositionMarketRequestParams,
) -> Result<()> {
    let data = encode_jupiter_create_increase_position_market_request(params);
    let ix = Instruction {
        program_id: JUPITER_PERPETUALS_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*accounts.owner.key, true),
            AccountMeta::new(*accounts.funding_account.key, false),
            AccountMeta::new_readonly(*accounts.perpetuals.key, false),
            AccountMeta::new_readonly(*accounts.pool.key, false),
            AccountMeta::new(*accounts.position.key, false),
            AccountMeta::new(*accounts.position_request.key, false),
            AccountMeta::new(*accounts.position_request_ata.key, false),
            AccountMeta::new_readonly(*accounts.custody.key, false),
            AccountMeta::new_readonly(*accounts.collateral_custody.key, false),
            AccountMeta::new_readonly(*accounts.input_mint.key, false),
            AccountMeta::new_readonly(*accounts.referral.key, false),
            AccountMeta::new_readonly(*accounts.token_program.key, false),
            AccountMeta::new_readonly(*accounts.associated_token_program.key, false),
            AccountMeta::new_readonly(*accounts.system_program.key, false),
            AccountMeta::new_readonly(*accounts.event_authority.key, false),
            AccountMeta::new_readonly(*accounts.program.key, false),
        ],
        data,
    };

    invoke(
        &ix,
        &[
            accounts.owner,
            accounts.funding_account,
            accounts.perpetuals,
            accounts.pool,
            accounts.position,
            accounts.position_request,
            accounts.position_request_ata,
            accounts.custody,
            accounts.collateral_custody,
            accounts.input_mint,
            accounts.referral,
            accounts.token_program,
            accounts.associated_token_program,
            accounts.system_program,
            accounts.event_authority,
            accounts.program,
        ],
    )?;

    Ok(())
}

pub fn encode_jupiter_create_increase_position_market_request(
    params: JupiterCreateIncreasePositionMarketRequestParams,
) -> Vec<u8> {
    let mut data = JUPITER_CREATE_INCREASE_POSITION_MARKET_REQUEST_DISCRIMINATOR.to_vec();
    data.extend_from_slice(&params.size_usd_delta.to_le_bytes());
    data.extend_from_slice(&params.collateral_token_delta.to_le_bytes());
    data.push(params.side.jupiter_side_discriminator());
    data.extend_from_slice(&params.price_slippage_usd_e6.to_le_bytes());
    match params.jupiter_minimum_out {
        Some(minimum_out) => {
            data.push(1);
            data.extend_from_slice(&minimum_out.to_le_bytes());
        }
        None => data.push(0),
    }
    data.extend_from_slice(&params.counter.to_le_bytes());
    data
}
