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

export function decodeSharingConfig(data: Buffer): DecodedSharingConfig {
  let offset = 8;
  const bump = data.readUInt8(offset);
  offset += 1;
  const version = data.readUInt8(offset);
  offset += 1;
  const status = data.readUInt8(offset);
  offset += 1;
  const mint = new PublicKey(data.subarray(offset, offset + 32));
  offset += 32;
  const admin = new PublicKey(data.subarray(offset, offset + 32));
  offset += 32;
  const adminRevoked = data.readUInt8(offset) !== 0;
  offset += 1;
  const shareholderCount = data.readUInt32LE(offset);
  offset += 4;

  const shareholders: Shareholder[] = [];
  for (let i = 0; i < shareholderCount; i += 1) {
    const address = new PublicKey(data.subarray(offset, offset + 32));
    offset += 32;
    const shareBps = data.readUInt16LE(offset);
    offset += 2;
    shareholders.push({ address, shareBps });
  }

  return { bump, version, status, mint, admin, adminRevoked, shareholders };
}

