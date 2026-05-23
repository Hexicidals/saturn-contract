import { BN } from "@coral-xyz/anchor";
import { AccountMeta, PublicKey } from "@solana/web3.js";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  JLP_POOL_ACCOUNT,
  JUPITER_CUSTODIES,
  JUPITER_PERPETUALS_EVENT_AUTHORITY,
  JUPITER_PERPETUALS_PROGRAM_ID,
  PUMP_AMM_PROGRAM_ID,
  PUMP_PROGRAM_ID,
  PositionSide,
  SYSTEM_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  TargetMarket,
  WSOL_MINT,
} from "./constants";
import {
  ata,
  bondingCurvePda,
  coinCreatorVaultAuthorityPda,
  creatorVaultPda,
  jupiterPerpetualsPda,
  jupiterPositionPda,
  jupiterPositionRequestPda,
  pumpAmmEventAuthority,
  pumpEventAuthority,
  sharingConfigPda,
  tradeConfigPda,
} from "./pdas";
import { Shareholder } from "./sharingConfig";

export type TradeConfigLike = {
  feeOwner: PublicKey;
  quoteMint: PublicKey;
  targetMarket: TargetMarket;
  side: PositionSide;
  custody: PublicKey;
  collateralCustody: PublicKey;
};

export function configForMarket(args: {
  feeOwner: PublicKey;
  quoteMint: PublicKey;
  market: TargetMarket;
  side: PositionSide;
}): TradeConfigLike {
  const custody = JUPITER_CUSTODIES[args.market];
  return {
    feeOwner: args.feeOwner,
    quoteMint: args.quoteMint,
    targetMarket: args.market,
    side: args.side,
    custody,
    collateralCustody: args.side === "long" ? custody : JUPITER_CUSTODIES.usdc,
  };
}

export function resolveSingleAccounts(args: {
  config: TradeConfigLike;
  counter: BN;
}) {
  const feeOwner = args.config.feeOwner;
  const quoteMint = args.config.quoteMint;
  const ownerQuoteAta = ata({ mint: quoteMint, owner: feeOwner });
  const creatorVault = creatorVaultPda(feeOwner);
  const coinCreatorVaultAuthority = coinCreatorVaultAuthorityPda(feeOwner);
  const position = jupiterPositionPda({
    owner: feeOwner,
    market: args.config.targetMarket,
    side: args.config.side,
  });
  const positionRequest = jupiterPositionRequestPda({
    position,
    counter: args.counter,
  });

  return {
    feeOwner,
    tradeConfig: tradeConfigPda(feeOwner),
    quoteMint,
    feeOwnerQuoteTokenAccount: ownerQuoteAta,
    creatorVault,
    creatorVaultTokenAccount: ata({
      mint: quoteMint,
      owner: creatorVault,
      allowOwnerOffCurve: true,
    }),
    quoteTokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    systemProgram: SYSTEM_PROGRAM_ID,
    pumpEventAuthority: pumpEventAuthority(),
    pumpProgram: PUMP_PROGRAM_ID,
    coinCreatorVaultAuthority,
    coinCreatorVaultAta: ata({
      mint: quoteMint,
      owner: coinCreatorVaultAuthority,
      allowOwnerOffCurve: true,
    }),
    pumpAmmEventAuthority: pumpAmmEventAuthority(),
    pumpAmmProgram: PUMP_AMM_PROGRAM_ID,
    perpetuals: jupiterPerpetualsPda(),
    pool: JLP_POOL_ACCOUNT,
    position,
    positionRequest,
    positionRequestAta: ata({
      mint: quoteMint,
      owner: positionRequest,
      allowOwnerOffCurve: true,
    }),
    custody: args.config.custody,
    collateralCustody: args.config.collateralCustody,
    referral: JUPITER_PERPETUALS_PROGRAM_ID,
    jupiterEventAuthority: JUPITER_PERPETUALS_EVENT_AUTHORITY,
    jupiterProgram: JUPITER_PERPETUALS_PROGRAM_ID,
  };
}

export function resolveSharedAccounts(args: {
  config: TradeConfigLike;
  mint: PublicKey;
  counter: BN;
}) {
  const sharingConfig = sharingConfigPda(args.mint);
  const quoteMint = args.config.quoteMint;
  const feeOwner = args.config.feeOwner;
  const creatorVault = creatorVaultPda(sharingConfig);
  const coinCreatorVaultAuthority = coinCreatorVaultAuthorityPda(sharingConfig);
  const position = jupiterPositionPda({
    owner: feeOwner,
    market: args.config.targetMarket,
    side: args.config.side,
  });
  const positionRequest = jupiterPositionRequestPda({
    position,
    counter: args.counter,
  });

  return {
    feeOwner,
    tradeConfig: tradeConfigPda(feeOwner),
    mint: args.mint,
    bondingCurve: bondingCurvePda(args.mint),
    sharingConfig,
    quoteMint,
    feeOwnerQuoteTokenAccount: ata({ mint: quoteMint, owner: feeOwner }),
    creatorVault,
    creatorVaultQuoteTokenAccount: ata({
      mint: quoteMint,
      owner: creatorVault,
      allowOwnerOffCurve: true,
    }),
    quoteTokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    systemProgram: SYSTEM_PROGRAM_ID,
    pumpEventAuthority: pumpEventAuthority(),
    pumpProgram: PUMP_PROGRAM_ID,
    coinCreatorVaultAuthority,
    coinCreatorVaultAta: ata({
      mint: quoteMint,
      owner: coinCreatorVaultAuthority,
      allowOwnerOffCurve: true,
    }),
    pumpAmmEventAuthority: pumpAmmEventAuthority(),
    pumpAmmProgram: PUMP_AMM_PROGRAM_ID,
    perpetuals: jupiterPerpetualsPda(),
    pool: JLP_POOL_ACCOUNT,
    position,
    positionRequest,
    positionRequestAta: ata({
      mint: quoteMint,
      owner: positionRequest,
      allowOwnerOffCurve: true,
    }),
    custody: args.config.custody,
    collateralCustody: args.config.collateralCustody,
    referral: JUPITER_PERPETUALS_PROGRAM_ID,
    jupiterEventAuthority: JUPITER_PERPETUALS_EVENT_AUTHORITY,
    jupiterProgram: JUPITER_PERPETUALS_PROGRAM_ID,
  };
}

export function shareholderRemainingAccounts(args: {
  shareholders: Shareholder[];
  quoteMint: PublicKey;
}): AccountMeta[] {
  const walletMetas = args.shareholders.map((shareholder) => ({
    pubkey: shareholder.address,
    isSigner: false,
    isWritable: args.quoteMint.equals(WSOL_MINT),
  }));
  if (args.quoteMint.equals(WSOL_MINT)) return walletMetas;

  const ataMetas = args.shareholders.map((shareholder) => ({
    pubkey: ata({ mint: args.quoteMint, owner: shareholder.address }),
    isSigner: false,
    isWritable: true,
  }));
  return [...walletMetas, ...ataMetas];
}

export function publicKeyRecord(record: Record<string, PublicKey>): Record<string, string> {
  return Object.fromEntries(
    Object.entries(record).map(([key, value]) => [key, value.toBase58()]),
  );
}

