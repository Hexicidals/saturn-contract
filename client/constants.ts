import { PublicKey } from "@solana/web3.js";

export const PROGRAM_ID = new PublicKey(
  "FjDSgr7sF8o3rwqnSp9m87xjEX18XxgWELhNVxwVkjDz",
);

export const PUMP_PROGRAM_ID = new PublicKey(
  "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P",
);
export const PUMP_AMM_PROGRAM_ID = new PublicKey(
  "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA",
);
export const PUMP_FEES_PROGRAM_ID = new PublicKey(
  "pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ",
);
export const JUPITER_PERPETUALS_PROGRAM_ID = new PublicKey(
  "PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu",
);
export const JUPITER_PERPETUALS_EVENT_AUTHORITY = new PublicKey(
  "37hJBDnntwqhGbK7L6M1bLyvccj4u55CCUiLPdYkiqBN",
);
export const JLP_POOL_ACCOUNT = new PublicKey(
  "5BUwFW4nRbftYTDMbgxykoFWqWHPzahFSNAaaaJtVKsq",
);

export const WSOL_MINT = new PublicKey(
  "So11111111111111111111111111111111111111112",
);
export const USDC_MINT = new PublicKey(
  "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
);
export const USDT_MINT = new PublicKey(
  "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
);
export const WETH_MINT = new PublicKey(
  "7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs",
);
export const WBTC_MINT = new PublicKey(
  "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
);

export const JUPITER_CUSTODIES = {
  sol: new PublicKey("7xS2gz2bTp3fwCC7knJvUWTEU9Tycczu6VhJYKgi1wdz"),
  eth: new PublicKey("AQCGyheWPLeo6Qp9WpYS9m3Qj479t7R636N9ey1rEjEn"),
  btc: new PublicKey("5Pv3gM9JrFFH883SWAhvJC9RPYmo8UNxuFtv5bMMALkm"),
  usdc: new PublicKey("G18jKKXQwBbrHeiK3C9MRXhkHsLHf7XgCSisykV46EZa"),
  usdt: new PublicKey("4vkNeXiYEUizLdrpdPS1eC2mccyM4NUPRtERrk6ZETkk"),
} as const;

export const JUPITER_MARKET_MINTS = {
  sol: WSOL_MINT,
  eth: WETH_MINT,
  btc: WBTC_MINT,
} as const;

export const TOKEN_PROGRAM_ID = new PublicKey(
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
);
export const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey(
  "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
);
export const SYSTEM_PROGRAM_ID = new PublicKey(
  "11111111111111111111111111111111",
);

export type TargetMarket = keyof typeof JUPITER_MARKET_MINTS;
export type PositionSide = "long" | "short";
export type ClaimMode = "single" | "shared";
