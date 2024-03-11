import * as flatbuffers from 'flatbuffers';
import { StatusTypes } from './status-types.js';
export declare class StatusV1 {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): StatusV1;
    static getRootAsStatusV1(bb: flatbuffers.ByteBuffer, obj?: StatusV1): StatusV1;
    static getSizePrefixedRootAsStatusV1(bb: flatbuffers.ByteBuffer, obj?: StatusV1): StatusV1;
    status(): StatusTypes;
    mutate_status(value: StatusTypes): boolean;
    proof(index: number): number | null;
    proofLength(): number;
    proofArray(): Uint8Array | null;
    inputs(index: number): number | null;
    inputsLength(): number;
    inputsArray(): Uint8Array | null;
    static startStatusV1(builder: flatbuffers.Builder): void;
    static addStatus(builder: flatbuffers.Builder, status: StatusTypes): void;
    static addProof(builder: flatbuffers.Builder, proofOffset: flatbuffers.Offset): void;
    static createProofVector(builder: flatbuffers.Builder, data: number[] | Uint8Array): flatbuffers.Offset;
    static startProofVector(builder: flatbuffers.Builder, numElems: number): void;
    static addInputs(builder: flatbuffers.Builder, inputsOffset: flatbuffers.Offset): void;
    static createInputsVector(builder: flatbuffers.Builder, data: number[] | Uint8Array): flatbuffers.Offset;
    static startInputsVector(builder: flatbuffers.Builder, numElems: number): void;
    static endStatusV1(builder: flatbuffers.Builder): flatbuffers.Offset;
    static finishStatusV1Buffer(builder: flatbuffers.Builder, offset: flatbuffers.Offset): void;
    static finishSizePrefixedStatusV1Buffer(builder: flatbuffers.Builder, offset: flatbuffers.Offset): void;
    static createStatusV1(builder: flatbuffers.Builder, status: StatusTypes, proofOffset: flatbuffers.Offset, inputsOffset: flatbuffers.Offset): flatbuffers.Offset;
}
