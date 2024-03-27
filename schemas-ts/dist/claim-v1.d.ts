import * as flatbuffers from 'flatbuffers';
export declare class ClaimV1 {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): ClaimV1;
    static getRootAsClaimV1(bb: flatbuffers.ByteBuffer, obj?: ClaimV1): ClaimV1;
    static getSizePrefixedRootAsClaimV1(bb: flatbuffers.ByteBuffer, obj?: ClaimV1): ClaimV1;
    executionId(): string | null;
    executionId(optionalEncoding: flatbuffers.Encoding): string | Uint8Array | null;
    blockCommitment(): bigint;
    mutate_block_commitment(value: bigint): boolean;
    static startClaimV1(builder: flatbuffers.Builder): void;
    static addExecutionId(builder: flatbuffers.Builder, executionIdOffset: flatbuffers.Offset): void;
    static addBlockCommitment(builder: flatbuffers.Builder, blockCommitment: bigint): void;
    static endClaimV1(builder: flatbuffers.Builder): flatbuffers.Offset;
    static finishClaimV1Buffer(builder: flatbuffers.Builder, offset: flatbuffers.Offset): void;
    static finishSizePrefixedClaimV1Buffer(builder: flatbuffers.Builder, offset: flatbuffers.Offset): void;
    static createClaimV1(builder: flatbuffers.Builder, executionIdOffset: flatbuffers.Offset, blockCommitment: bigint): flatbuffers.Offset;
}
