use anchor_lang::prelude::*;

use crate::constants::*;
use crate::errors::PumpJupiterError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClaimDeltas {
    pub lamports: u64,
    pub tokens: u64,
    pub total: u64,
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

pub fn claim_deltas(
    before_lamports: u64,
    after_lamports: u64,
    before_tokens: u64,
    after_tokens: u64,
    include_lamports: bool,
) -> Result<ClaimDeltas> {
    let tokens = after_tokens.saturating_sub(before_tokens);
    let lamports = if include_lamports {
        after_lamports.saturating_sub(before_lamports)
    } else {
        0
    };
    let total = tokens
        .checked_add(lamports)
        .ok_or_else(|| error!(PumpJupiterError::MathOverflow))?;

    Ok(ClaimDeltas {
        lamports,
        tokens,
        total,
    })
}

pub fn claimed_quote_delta(
    before_lamports: u64,
    after_lamports: u64,
    before_tokens: u64,
    after_tokens: u64,
    include_lamports: bool,
) -> Result<u64> {
    Ok(claim_deltas(
        before_lamports,
        after_lamports,
        before_tokens,
        after_tokens,
        include_lamports,
    )?
    .total)
}

pub fn position_size_usd_e6(
    collateral_token_delta: u64,
    quote_mint: Pubkey,
    quote_price_usd_e6: u64,
    leverage_bps: u64,
) -> Result<u64> {
    let quote_denominator = quote_amount_denominator(quote_mint)?;
    let size = u128::from(collateral_token_delta)
        .checked_mul(u128::from(quote_price_usd_e6))
        .and_then(|value| value.checked_mul(u128::from(leverage_bps)))
        .and_then(|value| value.checked_div(quote_denominator))
        .and_then(|value| value.checked_div(BPS_DENOMINATOR))
        .ok_or_else(|| error!(PumpJupiterError::MathOverflow))?;

    require!(size > 0, PumpJupiterError::PositionSizeTooSmall);
    u64::try_from(size).map_err(|_| error!(PumpJupiterError::MathOverflow))
}

pub fn quote_amount_denominator(quote_mint: Pubkey) -> Result<u128> {
    if quote_mint == WSOL_MINT {
        Ok(WSOL_DECIMALS_DENOMINATOR)
    } else if quote_mint == USDC_MINT {
        Ok(USD_DECIMALS_DENOMINATOR)
    } else {
        err!(PumpJupiterError::UnsupportedQuoteMint)
    }
}
