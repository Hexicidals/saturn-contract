import { PublicKey } from "@solana/web3.js";
import { USDC_MINT, WSOL_MINT } from "./constants";

const BPS_DENOMINATOR = 10_000n;
const USDC_DENOMINATOR = 1_000_000n;
const WSOL_DENOMINATOR = 1_000_000_000n;

export function quoteDenominator(quoteMint: PublicKey): bigint {
  if (quoteMint.equals(USDC_MINT)) return USDC_DENOMINATOR;
  if (quoteMint.equals(WSOL_MINT)) return WSOL_DENOMINATOR;
  throw new Error(`Unsupported quote mint ${quoteMint.toBase58()}`);
}

export function positionSizeUsdE6(args: {
  collateralTokenDelta: bigint;
  quoteMint: PublicKey;
  quotePriceUsdE6: bigint;
  leverageBps: bigint;
}): bigint {
  const size =
    (args.collateralTokenDelta *
      args.quotePriceUsdE6 *
      args.leverageBps) /
    quoteDenominator(args.quoteMint) /
    BPS_DENOMINATOR;
  if (size <= 0n) throw new Error("Calculated Jupiter position size is zero");
  return size;
}

