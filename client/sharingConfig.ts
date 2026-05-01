import { PublicKey } from "@solana/web3.js";

export type Shareholder = {
  address: PublicKey;
  shareBps: number;
};

export type DecodedSharingConfig = {
  bump: number;
  version: number;
  status: number;
  mint: PublicKey;
  admin: PublicKey;
  adminRevoked: boolean;
  shareholders: Shareholder[];
};

export const SHARING_CONFIG_DISCRIMINATOR = Buffer.from([
  216, 74, 9, 0, 56, 140, 93, 75,
]);

const SHARING_CONFIG_FIXED_LEN = 8 + 1 + 1 + 1 + 32 + 32 + 1 + 4;
const SHAREHOLDER_LEN = 32 + 2;
const MAX_SHAREHOLDERS = 256;

function requireRemaining(data: Buffer, offset: number, size: number, label: string) {
  if (offset + size > data.length) {
    throw new Error(`Sharing config is truncated at ${label}`);
  }
}

export function decodeSharingConfig(data: Buffer): DecodedSharingConfig {
  if (data.length < SHARING_CONFIG_FIXED_LEN) {
    throw new Error("Sharing config account is too small");
  }
  if (!data.subarray(0, 8).equals(SHARING_CONFIG_DISCRIMINATOR)) {
    throw new Error("Sharing config discriminator mismatch");
  }

  let offset = 8;
  requireRemaining(data, offset, 1, "bump");
  const bump = data.readUInt8(offset);
  offset += 1;
  requireRemaining(data, offset, 1, "version");
  const version = data.readUInt8(offset);
  offset += 1;
  requireRemaining(data, offset, 1, "status");
  const status = data.readUInt8(offset);
  offset += 1;
  requireRemaining(data, offset, 32, "mint");
  const mint = new PublicKey(data.subarray(offset, offset + 32));
  offset += 32;
  requireRemaining(data, offset, 32, "admin");
  const admin = new PublicKey(data.subarray(offset, offset + 32));
  offset += 32;
  requireRemaining(data, offset, 1, "admin revoked");
  const adminRevoked = data.readUInt8(offset) !== 0;
  offset += 1;
  requireRemaining(data, offset, 4, "shareholder count");
  const shareholderCount = data.readUInt32LE(offset);
  offset += 4;
  if (shareholderCount > MAX_SHAREHOLDERS) {
    throw new Error(`Sharing config has too many shareholders: ${shareholderCount}`);
  }
  requireRemaining(
    data,
    offset,
    shareholderCount * SHAREHOLDER_LEN,
    "shareholder vector",
  );

  const shareholders: Shareholder[] = [];
  let totalShareBps = 0;
  for (let i = 0; i < shareholderCount; i += 1) {
    const address = new PublicKey(data.subarray(offset, offset + 32));
    offset += 32;
    const shareBps = data.readUInt16LE(offset);
    offset += 2;
    totalShareBps += shareBps;
    shareholders.push({ address, shareBps });
  }
  if (totalShareBps > 10_000) {
    throw new Error(`Sharing config shareholder bps exceeds 10000: ${totalShareBps}`);
  }

  return { bump, version, status, mint, admin, adminRevoked, shareholders };
}
