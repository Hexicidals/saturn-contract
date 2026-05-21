use anchor_lang::prelude::*;

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
