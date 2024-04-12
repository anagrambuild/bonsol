import { Address, address, getAddressEncoder, getProgramDerivedAddress } from "@solana/web3.js";
import {keccak_256} from "@noble/hashes/sha3";
export * from "./execute";
export * from "./deploy";

export const BONSOL_PROGRAM_ID = "BoNSrwTtTM4PRkbbPvehk1XzHC65cKfdNSod9FyTejRn";

export async function executionAddress(
  requester: Address,
  executionId: string
): Promise<Address> {
  const addressEncoder = getAddressEncoder();
  const pd = await getProgramDerivedAddress({
    programAddress: address(BONSOL_PROGRAM_ID),
    seeds: [
      Buffer.from("execution", "utf-8"),
      addressEncoder.encode(requester),
      Buffer.from(executionId, "utf-8")
    ]
  });
  return pd[0];
}

export async function deploymentAddress(
  imageId: string
): Promise<Address> {
  const imageIdHash = keccak_256(Buffer.from(imageId, "utf-8"));
  const pd = await getProgramDerivedAddress({
    programAddress: address(BONSOL_PROGRAM_ID),
    seeds: [
      Buffer.from("deployment", "utf-8"),
      imageIdHash
    ]
  });
  return pd[0];
}

export async function executionClaimAddress(
  requester: Address,
  executionId: string,
  claimer: Address
): Promise<Address> {
  const addressEncoder = getAddressEncoder();
  const pd = await getProgramDerivedAddress({
    programAddress: address(BONSOL_PROGRAM_ID),
    seeds: [
      Buffer.from("execution-claim", "utf-8"),
      addressEncoder.encode(requester),
      Buffer.from(executionId, "utf-8"),
      addressEncoder.encode(claimer)
    ]
  });
  return pd[0];
}