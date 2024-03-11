import * as flatbuffers from 'flatbuffers';
export declare class Inputs {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): Inputs;
    static getRootAsInputs(bb: flatbuffers.ByteBuffer, obj?: Inputs): Inputs;
    static getSizePrefixedRootAsInputs(bb: flatbuffers.ByteBuffer, obj?: Inputs): Inputs;
    publicInputs(index: number): number | null;
    publicInputsLength(): number;
    publicInputsArray(): Uint8Array | null;
    privateInputs(index: number): string;
    privateInputs(index: number, optionalEncoding: flatbuffers.Encoding): string | Uint8Array;
    privateInputsLength(): number;
    static startInputs(builder: flatbuffers.Builder): void;
    static addPublicInputs(builder: flatbuffers.Builder, publicInputsOffset: flatbuffers.Offset): void;
    static createPublicInputsVector(builder: flatbuffers.Builder, data: number[] | Uint8Array): flatbuffers.Offset;
    static startPublicInputsVector(builder: flatbuffers.Builder, numElems: number): void;
    static addPrivateInputs(builder: flatbuffers.Builder, privateInputsOffset: flatbuffers.Offset): void;
    static createPrivateInputsVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startPrivateInputsVector(builder: flatbuffers.Builder, numElems: number): void;
    static endInputs(builder: flatbuffers.Builder): flatbuffers.Offset;
    static createInputs(builder: flatbuffers.Builder, publicInputsOffset: flatbuffers.Offset, privateInputsOffset: flatbuffers.Offset): flatbuffers.Offset;
}
