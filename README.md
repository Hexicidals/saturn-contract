# Pump Fees To Jupiter Perps

Anchor program and TypeScript client for claiming Pump.fun creator fees and creating a Jupiter Perps increase-position market request in the same transaction.

The Jupiter Perps leg creates a request account. Jupiter keepers execute the actual position later, so the position fill is not atomic with the Pump fee claim.

## Build

```bash
npm install
anchor build
npm run typecheck
npm run test:ts
cargo test -p pump-fees-to-jupiter-perps
npm run test:anchor
```

`npm run test:anchor` runs the Anchor local-validator suite. The wrapper keeps this
repo reproducible with the local Anchor/Solana toolchain by copying the generated
SBF artifact into `target/deploy` before invoking `anchor test --skip-build`.

## Program Flow

- `initialize_trade_config` creates one `TradeConfig` PDA per fee owner.
- `update_trade_config` changes quote mint, market, side, custody accounts, max leverage, or pause state.
- `claim_single_and_open` claims single-recipient Pump bonding-curve fees and optional PumpSwap AMM fees, then creates a Jupiter Perps increase request.
- `claim_shared_and_open` optionally sweeps PumpSwap shared fees into the Pump creator vault, distributes the sharing config, then uses only the signer’s received share for the Jupiter request.

Supported quote mints are WSOL and USDC. Supported Jupiter Perps markets are SOL, ETH, and BTC.

## CLI

The CLI loads `target/idl/pump_fees_to_jupiter_perps.json`, so run `anchor build` first.

Resolve accounts without touching chain state:

```bash
npm run cli -- resolve-accounts \
  --mode single \
  --fee-owner <WALLET> \
  --quote-mint usdc \
  --market sol \
  --side short \
  --counter 1
```

Initialize config, simulated by default:

```bash
npm run cli -- init-config \
  --fee-owner <WALLET> \
  --quote-mint usdc \
  --market sol \
  --side short \
  --max-leverage-bps 50000
```

Build and simulate a claim/open request:

```bash
npm run cli -- claim-open \
  --mode single \
  --fee-owner <WALLET> \
  --leverage-bps 30000 \
  --quote-price-usd-e6 1000000 \
  --price-slippage-usd-e6 200000000 \
  --counter 1 \
  --min-claim-amount 1
```

Use `--send` only after the program is deployed on the target cluster and the wallet/config/accounts are funded and initialized. Mainnet simulation against `api.mainnet-beta.solana.com` also requires the custom program to already exist on mainnet.

## Notes

- Single-recipient Pump creator vaults are creator/quote scoped, so a single claim can sweep all claimable fees for that creator and quote.
- Fee-sharing mode reads the Pump Fees sharing config and forwards shareholders as remaining accounts in the exact order required by Pump.
- If the quote mint differs from the Jupiter collateral mint, pass `--jupiter-minimum-out` from an external quote.
- The repo intentionally does not submit live mainnet transactions by default.
