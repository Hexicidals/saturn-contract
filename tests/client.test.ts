import { BN } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import assert from "node:assert/strict";
import {
  JLP_POOL_ACCOUNT,
  JUPITER_CUSTODIES,
  USDC_MINT,
  WSOL_MINT,
} from "../client/constants";
import {
  configForMarket,
  resolveSharedAccounts,
  resolveSingleAccounts,
  shareholderRemainingAccounts,
} from "../client/accounts";
import { positionSizeUsdE6 } from "../client/math";
import {
  bondingCurvePda,
  jupiterPositionPda,
  jupiterPositionRequestPda,
  sharingConfigPda,
  tradeConfigPda,
} from "../client/pdas";
import { decodeSharingConfig, SHARING_CONFIG_DISCRIMINATOR } from "../client/sharingConfig";

function encodeSharingConfig(args: {
  mint: PublicKey;
  admin: PublicKey;
  shareholders: { address: PublicKey; shareBps: number }[];
}): Buffer {
  const count = Buffer.alloc(4);
  count.writeUInt32LE(args.shareholders.length);
  const shareholderBytes = args.shareholders.map((shareholder) => {
    const share = Buffer.alloc(2);
    share.writeUInt16LE(shareholder.shareBps);
    return Buffer.concat([shareholder.address.toBuffer(), share]);
  });
  return Buffer.concat([
    SHARING_CONFIG_DISCRIMINATOR,
    Buffer.from([255, 1, 1]),
    args.mint.toBuffer(),
    args.admin.toBuffer(),
    Buffer.from([0]),
    count,
    ...shareholderBytes,
  ]);
}

describe("client account resolution", () => {
  const feeOwner = new PublicKey("11111111111111111111111111111112");
  const mint = new PublicKey("11111111111111111111111111111113");

  it("derives the trade config from fee owner", () => {
    const first = tradeConfigPda(feeOwner);
    const second = tradeConfigPda(feeOwner);
    assert.equal(first.toBase58(), second.toBase58());
  });

  it("rejects non-u64 Jupiter position request counters", () => {
    assert.throws(
      () =>
        jupiterPositionRequestPda({
          position: feeOwner,
          counter: new BN(-1),
        }),
      /must be a u64/,
    );
  });

  it("resolves single-recipient Jupiter accounts from config", () => {
    const config = configForMarket({
      feeOwner,
      quoteMint: USDC_MINT,
      market: "sol",
      side: "short",
    });
    const accounts = resolveSingleAccounts({ config, counter: new BN(7) });

    assert.equal(accounts.pool.toBase58(), JLP_POOL_ACCOUNT.toBase58());
    assert.equal(accounts.custody.toBase58(), JUPITER_CUSTODIES.sol.toBase58());
    assert.equal(
      accounts.collateralCustody.toBase58(),
      JUPITER_CUSTODIES.usdc.toBase58(),
    );
    assert.equal(
      accounts.position.toBase58(),
      jupiterPositionPda({ owner: feeOwner, market: "sol", side: "short" }).toBase58(),
    );
  });

  it("resolves shared Pump accounts from mint", () => {
    const config = configForMarket({
      feeOwner,
      quoteMint: WSOL_MINT,
      market: "sol",
      side: "long",
    });
    const accounts = resolveSharedAccounts({ config, mint, counter: new BN(8) });

    assert.equal(accounts.bondingCurve.toBase58(), bondingCurvePda(mint).toBase58());
    assert.equal(accounts.sharingConfig.toBase58(), sharingConfigPda(mint).toBase58());
  });

  it("marks SOL shareholder wallets writable and USDC ATAs writable", () => {
    const shareholders = [{ address: feeOwner, shareBps: 10_000 }];
    const solMetas = shareholderRemainingAccounts({
      shareholders,
      quoteMint: WSOL_MINT,
    });
    const usdcMetas = shareholderRemainingAccounts({
      shareholders,
      quoteMint: USDC_MINT,
    });

    assert.equal(solMetas.length, 1);
    assert.equal(solMetas[0].isWritable, true);
    assert.equal(usdcMetas.length, 2);
    assert.equal(usdcMetas[0].isWritable, false);
    assert.equal(usdcMetas[1].isWritable, true);
  });
});

describe("sharing config decoding", () => {
  const mint = new PublicKey("11111111111111111111111111111113");
  const admin = new PublicKey("11111111111111111111111111111114");
  const shareholder = new PublicKey("11111111111111111111111111111115");

  it("validates and decodes Pump Fees sharing config data", () => {
    const decoded = decodeSharingConfig(
      encodeSharingConfig({
        mint,
        admin,
        shareholders: [{ address: shareholder, shareBps: 2_500 }],
      }),
    );

    assert.equal(decoded.mint.toBase58(), mint.toBase58());
    assert.equal(decoded.admin.toBase58(), admin.toBase58());
    assert.equal(decoded.shareholders.length, 1);
    assert.equal(decoded.shareholders[0].shareBps, 2_500);
  });

  it("rejects malformed sharing config data", () => {
    assert.throws(() => decodeSharingConfig(Buffer.alloc(16)), /too small/);

    const data = encodeSharingConfig({
      mint,
      admin,
      shareholders: [{ address: shareholder, shareBps: 10_001 }],
    });
    data[0] = 0;
    assert.throws(() => decodeSharingConfig(data), /discriminator mismatch/);

    assert.throws(
      () =>
        decodeSharingConfig(
          encodeSharingConfig({
            mint,
            admin,
            shareholders: [{ address: shareholder, shareBps: 10_001 }],
          }),
        ),
      /exceeds 10000/,
    );
  });
});

describe("client math", () => {
  it("matches on-chain USDC position size math", () => {
    const size = positionSizeUsdE6({
      collateralTokenDelta: 10_000_000n,
      quoteMint: USDC_MINT,
      quotePriceUsdE6: 1_000_000n,
      leverageBps: 30_000n,
    });

    assert.equal(size, 30_000_000n);
  });
});
