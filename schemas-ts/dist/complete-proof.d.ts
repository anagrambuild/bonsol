import * as flatbuffers from 'flatbuffers';
export declare class CompleteProof {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): CompleteProof;
    seal(index: number): number | null;
    input(index: number): number | null;
    static sizeOf(): number;
    static createCompleteProof(builder: flatbuffers.Builder, seal: number[] | null, input: number[] | null): flatbuffers.Offset;
}
