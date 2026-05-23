#!/usr/bin/env ts-node
import * as anchor from "@coral-xyz/anchor";
import { BN, Idl, Program } from "@coral-xyz/anchor";
import {
  Connection,
  PublicKey,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import { Command } from "commander";
import "dotenv/config";
import fs from "fs";
import path from "path";
import {
  ClaimMode,
  PositionSide,
  TargetMarket,
  USDC_MINT,
  WSOL_MINT,
} from "./constants";
import {
  configForMarket,
  publicKeyRecord,
  resolveSharedAccounts,
  resolveSingleAccounts,
  shareholderRemainingAccounts,
  TradeConfigLike,
} from "./accounts";
import { tradeConfigPda } from "./pdas";
import { decodeSharingConfig } from "./sharingConfig";
import { positionSizeUsdE6 } from "./math";

type ProgramClient = Program<Idl>;

function publicKey(value: string): PublicKey {
  return new PublicKey(value);
}

function quoteMint(value: string): PublicKey {
  const normalized = value.toLowerCase();
  if (normalized === "wsol" || normalized === "sol") return WSOL_MINT;
  if (normalized === "usdc") return USDC_MINT;
  return publicKey(value);
}

function enumVariant(name: string) {
  return { [name]: {} };
}

function parseMarket(value: string): TargetMarket {
  const normalized = value.toLowerCase();
  if (normalized === "sol" || normalized === "eth" || normalized === "btc") {
    return normalized;
  }
  throw new Error("market must be one of: sol, eth, btc");
}

function parseSide(value: string): PositionSide {
  const normalized = value.toLowerCase();
  if (normalized === "long" || normalized === "short") return normalized;
  throw new Error("side must be long or short");
}

function loadIdl(): Idl {
  const idlPath = path.resolve(
    process.cwd(),
    "target/idl/pump_fees_to_jupiter_perps.json",
  );
  if (!fs.existsSync(idlPath)) {
    throw new Error("IDL not found. Run `anchor build` before using the CLI.");
  }
  return JSON.parse(fs.readFileSync(idlPath, "utf8")) as Idl;
}

function provider(): anchor.AnchorProvider {
  const rpcUrl =
    process.env.RPC_URL || process.env.ANCHOR_PROVIDER_URL || "http://127.0.0.1:8899";
  const connection = new Connection(rpcUrl, "confirmed");
  const wallet = anchor.Wallet.local();
  return new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed",
    preflightCommitment: "confirmed",
  });
}

function programClient(provider: anchor.AnchorProvider): ProgramClient {
  return new Program(loadIdl(), provider) as ProgramClient;
}

async function simulateOrSend(args: {
  provider: anchor.AnchorProvider;
  instructions: TransactionInstruction[];
  send: boolean;
}) {
  const tx = new Transaction();
  tx.add(...args.instructions);
  tx.feePayer = args.provider.wallet.publicKey;
  const latest = await args.provider.connection.getLatestBlockhash();
  tx.recentBlockhash = latest.blockhash;
  const wallet = args.provider.wallet as anchor.Wallet;
  tx.sign(wallet.payer);

  if (!args.send) {
    const simulation = await args.provider.connection.simulateTransaction(tx);
    console.log(
      JSON.stringify(
        {
          mode: "simulate",
          err: simulation.value.err,
          logs: simulation.value.logs,
          unitsConsumed: simulation.value.unitsConsumed,
        },
        null,
        2,
      ),
    );
    return;
  }

  const signature = await args.provider.connection.sendRawTransaction(
    tx.serialize(),
    { skipPreflight: false },
  );
  console.log(JSON.stringify({ mode: "send", signature }, null, 2));
}

function tradeConfigParams(args: {
  quoteMint: PublicKey;
  market: TargetMarket;
  side: PositionSide;
  maxLeverageBps: string;
}) {
  const config = configForMarket({
    feeOwner: PublicKey.default,
    quoteMint: args.quoteMint,
    market: args.market,
    side: args.side,
  });
  return {
    quoteMint: args.quoteMint,
    targetMarket: enumVariant(args.market),
    side: enumVariant(args.side),
    custody: config.custody,
    collateralCustody: config.collateralCustody,
    maxLeverageBps: new BN(args.maxLeverageBps),
  };
}

function claimOpenParams(args: {
  leverageBps: string;
  quotePriceUsdE6: string;
  priceSlippageUsdE6: string;
  jupiterMinimumOut: string;
  counter: string;
  minClaimAmount: string;
  maxClaimAmount: string;
}) {
  return {
    leverageBps: new BN(args.leverageBps),
    quotePriceUsdE6: new BN(args.quotePriceUsdE6),
    priceSlippageUsdE6: new BN(args.priceSlippageUsdE6),
    jupiterMinimumOut: new BN(args.jupiterMinimumOut),
    positionRequestCounter: new BN(args.counter),
    minClaimAmount: new BN(args.minClaimAmount),
    maxClaimAmount: new BN(args.maxClaimAmount),
  };
}

function requiredCollateralMint(config: TradeConfigLike): PublicKey {
  if (config.side === "short") return USDC_MINT;
  if (config.targetMarket === "sol") return WSOL_MINT;
  throw new Error(
    "Long ETH/BTC require a swap into wETH/wBTC collateral; pass --jupiter-minimum-out from a quote.",
  );
}

function ensureMinimumOut(args: {
  config: TradeConfigLike;
  jupiterMinimumOut: string;
}) {
  if (args.jupiterMinimumOut !== "0") return;
  const requiredMint = requiredCollateralMint(args.config);
  if (!args.config.quoteMint.equals(requiredMint)) {
    throw new Error(
      "This quote mint differs from the Jupiter collateral mint; pass --jupiter-minimum-out.",
    );
  }
}

async function fetchTradeConfig(
  program: ProgramClient,
  feeOwner: PublicKey,
): Promise<TradeConfigLike> {
  const address = tradeConfigPda(feeOwner);
  const account = (await (program.account as any).tradeConfig.fetch(address)) as any;
  const targetMarket = Object.keys(account.targetMarket)[0] as TargetMarket;
  const side = Object.keys(account.side)[0] as PositionSide;
  return {
    feeOwner: account.feeOwner,
    quoteMint: account.quoteMint,
    targetMarket,
    side,
    custody: account.custody,
    collateralCustody: account.collateralCustody,
  };
}

async function loadSharedRemainingAccounts(args: {
  connection: Connection;
  sharingConfig: PublicKey;
  quoteMint: PublicKey;
}) {
  const info = await args.connection.getAccountInfo(args.sharingConfig);
  if (!info) {
    throw new Error(`Sharing config not found: ${args.sharingConfig.toBase58()}`);
  }
  const decoded = decodeSharingConfig(info.data);
  return shareholderRemainingAccounts({
    shareholders: decoded.shareholders,
    quoteMint: args.quoteMint,
  });
}

function logResolved(record: Record<string, PublicKey>, remaining?: { pubkey: PublicKey }[]) {
  console.log(
    JSON.stringify(
      {
        accounts: publicKeyRecord(record),
        remainingAccounts: remaining?.map((meta) => meta.pubkey.toBase58()) ?? [],
      },
      null,
      2,
    ),
  );
}

const cli = new Command();
cli.name("pump-jup").description("Pump creator fees to Jupiter Perps client");

cli
  .command("init-config")
  .requiredOption("--fee-owner <pubkey>")
  .requiredOption("--quote-mint <mint|wsol|usdc>")
  .requiredOption("--market <sol|eth|btc>")
  .requiredOption("--side <long|short>")
  .requiredOption("--max-leverage-bps <bps>")
  .option("--send", "submit instead of simulate", false)
  .action(async (opts) => {
    const p = provider();
    const program = programClient(p);
    const feeOwner = publicKey(opts.feeOwner);
    const ix = await (program.methods as any)
      .initializeTradeConfig(
        tradeConfigParams({
          quoteMint: quoteMint(opts.quoteMint),
          market: parseMarket(opts.market),
          side: parseSide(opts.side),
          maxLeverageBps: opts.maxLeverageBps,
        }),
      )
      .accounts({
        admin: p.wallet.publicKey,
        feeOwner,
        tradeConfig: tradeConfigPda(feeOwner),
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .instruction();
    await simulateOrSend({ provider: p, instructions: [ix], send: opts.send });
  });

cli
  .command("update-config")
  .requiredOption("--fee-owner <pubkey>")
  .requiredOption("--quote-mint <mint|wsol|usdc>")
  .requiredOption("--market <sol|eth|btc>")
  .requiredOption("--side <long|short>")
  .requiredOption("--max-leverage-bps <bps>")
  .option("--paused", "pause config", false)
  .option("--send", "submit instead of simulate", false)
  .action(async (opts) => {
    const p = provider();
    const program = programClient(p);
    const feeOwner = publicKey(opts.feeOwner);
    const ix = await (program.methods as any)
      .updateTradeConfig(
        tradeConfigParams({
          quoteMint: quoteMint(opts.quoteMint),
          market: parseMarket(opts.market),
          side: parseSide(opts.side),
          maxLeverageBps: opts.maxLeverageBps,
        }),
        opts.paused,
      )
      .accounts({
        admin: p.wallet.publicKey,
        tradeConfig: tradeConfigPda(feeOwner),
      })
      .instruction();
    await simulateOrSend({ provider: p, instructions: [ix], send: opts.send });
  });

cli
  .command("resolve-accounts")
  .requiredOption("--mode <single|shared>")
  .requiredOption("--fee-owner <pubkey>")
  .requiredOption("--quote-mint <mint|wsol|usdc>")
  .requiredOption("--market <sol|eth|btc>")
  .requiredOption("--side <long|short>")
  .requiredOption("--counter <u64>")
  .option("--mint <pubkey>", "base mint for shared mode")
  .action(async (opts) => {
    const mode = opts.mode as ClaimMode;
    const feeOwner = publicKey(opts.feeOwner);
    const config = configForMarket({
      feeOwner,
      quoteMint: quoteMint(opts.quoteMint),
      market: parseMarket(opts.market),
      side: parseSide(opts.side),
    });
    const counter = new BN(opts.counter);
    if (mode === "single") {
      logResolved(resolveSingleAccounts({ config, counter }));
      return;
    }
    if (!opts.mint) throw new Error("--mint is required for shared mode");
    logResolved(
      resolveSharedAccounts({
        config,
        mint: publicKey(opts.mint),
        counter,
      }),
    );
  });

cli
  .command("claim-open")
  .requiredOption("--mode <single|shared>")
  .requiredOption("--fee-owner <pubkey>")
  .requiredOption("--leverage-bps <bps>")
  .requiredOption("--quote-price-usd-e6 <amount>")
  .requiredOption("--price-slippage-usd-e6 <amount>")
  .requiredOption("--counter <u64>")
  .option("--jupiter-minimum-out <amount>", "required when Jupiter must swap", "0")
  .option("--min-claim-amount <amount>", "minimum claim amount", "0")
  .option("--max-claim-amount <amount>", "maximum claim amount, 0 disables", "0")
  .option("--collect-amm", "also collect PumpSwap AMM single-recipient fees", false)
  .option("--skip-bonding-curve", "skip bonding-curve single-recipient fees", false)
  .option("--sweep-amm", "sweep PumpSwap fee-sharing fees before distribution", false)
  .option("--mint <pubkey>", "base mint for shared mode")
  .option("--send", "submit instead of simulate", false)
  .action(async (opts) => {
    const p = provider();
    const program = programClient(p);
    const feeOwner = publicKey(opts.feeOwner);
    const config = await fetchTradeConfig(program, feeOwner);
    ensureMinimumOut({ config, jupiterMinimumOut: opts.jupiterMinimumOut });

    const params = claimOpenParams({
      leverageBps: opts.leverageBps,
      quotePriceUsdE6: opts.quotePriceUsdE6,
      priceSlippageUsdE6: opts.priceSlippageUsdE6,
      jupiterMinimumOut: opts.jupiterMinimumOut,
      counter: opts.counter,
      minClaimAmount: opts.minClaimAmount,
      maxClaimAmount: opts.maxClaimAmount,
    });

    const previewSize = positionSizeUsdE6({
      collateralTokenDelta: BigInt(opts.minClaimAmount || "0") || 1n,
      quoteMint: config.quoteMint,
      quotePriceUsdE6: BigInt(opts.quotePriceUsdE6),
      leverageBps: BigInt(opts.leverageBps),
    });
    console.error(`minimum preview sizeUsdDelta=${previewSize.toString()}`);

    const mode = opts.mode as ClaimMode;
    const counter = new BN(opts.counter);
    let ix: TransactionInstruction;
    let remainingAccounts: { pubkey: PublicKey; isSigner: boolean; isWritable: boolean }[] = [];

    if (mode === "single") {
      const accounts = resolveSingleAccounts({ config, counter });
      ix = await (program.methods as any)
        .claimSingleAndOpen(params, {
          collectBondingCurve: !opts.skipBondingCurve,
          collectAmm: opts.collectAmm,
        })
        .accounts(accounts)
        .instruction();
    } else {
      if (!opts.mint) throw new Error("--mint is required for shared mode");
      const accounts = resolveSharedAccounts({
        config,
        mint: publicKey(opts.mint),
        counter,
      });
      remainingAccounts = await loadSharedRemainingAccounts({
        connection: p.connection,
        sharingConfig: accounts.sharingConfig,
        quoteMint: config.quoteMint,
      });
      ix = await (program.methods as any)
        .claimSharedAndOpen(params, {
          sweepAmm: opts.sweepAmm,
          initializeShareholderAtas: true,
        })
        .accounts(accounts)
        .remainingAccounts(remainingAccounts)
        .instruction();
    }

    await simulateOrSend({ provider: p, instructions: [ix], send: opts.send });
  });

cli.parseAsync().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
