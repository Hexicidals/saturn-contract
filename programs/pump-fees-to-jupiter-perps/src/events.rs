use anchor_lang::prelude::*;

use crate::math::ClaimDeltas;

#[event]
pub struct FeesClaimed {
    pub fee_owner: Pubkey,
    pub quote_mint: Pubkey,
    pub amount: u64,
}

#[event]
pub struct JupiterPositionRequestCreated {
    pub fee_owner: Pubkey,
    pub position: Pubkey,
    pub position_request: Pubkey,
    pub collateral_token_delta: u64,
    pub size_usd_delta: u64,
}

pub struct ClaimOpenEventFields {
    pub fee_owner: Pubkey,
    pub quote_mint: Pubkey,
    pub position: Pubkey,
    pub position_request: Pubkey,
    pub deltas: ClaimDeltas,
    pub size_usd_delta: u64,
}

pub fn emit_claim_open_events(fields: ClaimOpenEventFields) {
    emit!(FeesClaimed {
        fee_owner: fields.fee_owner,
        quote_mint: fields.quote_mint,
        amount: fields.deltas.total,
    });
    emit!(JupiterPositionRequestCreated {
        fee_owner: fields.fee_owner,
        position: fields.position,
        position_request: fields.position_request,
        collateral_token_delta: fields.deltas.total,
        size_usd_delta: fields.size_usd_delta,
    });
}
