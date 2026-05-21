use anchor_lang::prelude::*;

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
pub const JUPITER_PERPETUALS_PROGRAM_ID: Pubkey =
    pubkey!("PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu");
pub const JUPITER_PERPETUALS_EVENT_AUTHORITY: Pubkey =
    pubkey!("37hJBDnntwqhGbK7L6M1bLyvccj4u55CCUiLPdYkiqBN");
pub const JLP_POOL_ACCOUNT: Pubkey = pubkey!("5BUwFW4nRbftYTDMbgxykoFWqWHPzahFSNAaaaJtVKsq");

pub const PUMP_COLLECT_CREATOR_FEE_V2_DISCRIMINATOR: [u8; 8] = [207, 17, 138, 242, 4, 34, 19, 56];
pub const PUMP_AMM_COLLECT_COIN_CREATOR_FEE_DISCRIMINATOR: [u8; 8] =
    [160, 57, 89, 42, 181, 139, 43, 66];
pub const PUMP_AMM_TRANSFER_CREATOR_FEES_TO_PUMP_V2_DISCRIMINATOR: [u8; 8] =
    [1, 33, 78, 185, 33, 67, 44, 92];
pub const PUMP_DISTRIBUTE_CREATOR_FEES_V2_DISCRIMINATOR: [u8; 8] =
    [255, 203, 19, 79, 244, 68, 8, 159];
pub const JUPITER_CREATE_INCREASE_POSITION_MARKET_REQUEST_DISCRIMINATOR: [u8; 8] =
    [184, 85, 199, 24, 105, 171, 156, 56];
pub const ASSOCIATED_TOKEN_CREATE_IDEMPOTENT_DISCRIMINATOR: u8 = 1;
pub const SPL_TOKEN_SYNC_NATIVE_DISCRIMINATOR: u8 = 17;

pub(crate) const SPL_TOKEN_AMOUNT_OFFSET: usize = 64;
pub(crate) const SPL_TOKEN_AMOUNT_END: usize = 72;
pub(crate) const BPS_DENOMINATOR: u128 = 10_000;
pub(crate) const USD_DECIMALS_DENOMINATOR: u128 = 1_000_000;
pub(crate) const WSOL_DECIMALS_DENOMINATOR: u128 = 1_000_000_000;
