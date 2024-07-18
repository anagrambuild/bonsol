import { Address } from "@solana/web3.js";
export * from "./execute";
export * from "./deploy";
export declare const BONSOL_PROGRAM_ID = "BoNsHRcyLLNdtnoDf8hiCNZpyehMC4FDMxs6NTxFi3ew";
export declare function executionAddress(requester: Address, executionId: string): Promise<Address>;
export declare function deploymentAddress(imageId: string): Promise<Address>;
export declare function executionClaimAddress(requester: Address, executionId: string, claimer: Address): Promise<Address>;
