import { BN } from "@coral-xyz/anchor";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import { PublicKey } from "@solana/web3.js";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  JLP_POOL_ACCOUNT,
  JUPITER_CUSTODIES,
  JUPITER_PERPETUALS_PROGRAM_ID,
  PROGRAM_ID,
  PUMP_AMM_PROGRAM_ID,
  PUMP_FEES_PROGRAM_ID,
  PUMP_PROGRAM_ID,
  PositionSide,
  TOKEN_PROGRAM_ID,
  TargetMarket,
} from "./constants";

export function tradeConfigPda(feeOwner: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("trade-config"), feeOwner.toBuffer()],
    PROGRAM_ID,
  )[0];
}

export function pumpEventAuthority(): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("__event_authority")],
    PUMP_PROGRAM_ID,
  )[0];
}

export function pumpAmmEventAuthority(): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("__event_authority")],
    PUMP_AMM_PROGRAM_ID,
  )[0];
}

export function bondingCurvePda(mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("bonding-curve"), mint.toBuffer()],
    PUMP_PROGRAM_ID,
  )[0];
}

export function creatorVaultPda(creator: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("creator-vault"), creator.toBuffer()],
    PUMP_PROGRAM_ID,
  )[0];
}

export function sharingConfigPda(mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("sharing-config"), mint.toBuffer()],
    PUMP_FEES_PROGRAM_ID,
  )[0];
}

export function coinCreatorVaultAuthorityPda(coinCreator: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("creator_vault"), coinCreator.toBuffer()],
    PUMP_AMM_PROGRAM_ID,
  )[0];
}

export function jupiterPerpetualsPda(): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("perpetuals")],
    JUPITER_PERPETUALS_PROGRAM_ID,
  )[0];
}

export function jupiterPositionPda(args: {
  owner: PublicKey;
  market: TargetMarket;
  side: PositionSide;
}): PublicKey {
  const custody = JUPITER_CUSTODIES[args.market];
  const collateralCustody =
    args.side === "long" ? custody : JUPITER_CUSTODIES.usdc;

  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("position"),
      args.owner.toBuffer(),
      JLP_POOL_ACCOUNT.toBuffer(),
      custody.toBuffer(),
      collateralCustody.toBuffer(),
      Buffer.from([args.side === "long" ? 1 : 2]),
    ],
    JUPITER_PERPETUALS_PROGRAM_ID,
  )[0];
}

export function jupiterPositionRequestPda(args: {
  position: PublicKey;
  counter: BN;
}): PublicKey {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("position_request"),
      args.position.toBuffer(),
      args.counter.toArrayLike(Buffer, "le", 8),
      Buffer.from([1]),
    ],
    JUPITER_PERPETUALS_PROGRAM_ID,
  )[0];
}

export function ata(args: {
  mint: PublicKey;
  owner: PublicKey;
  tokenProgram?: PublicKey;
  allowOwnerOffCurve?: boolean;
}): PublicKey {
  return getAssociatedTokenAddressSync(
    args.mint,
    args.owner,
    args.allowOwnerOffCurve ?? true,
    args.tokenProgram ?? TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );
}

