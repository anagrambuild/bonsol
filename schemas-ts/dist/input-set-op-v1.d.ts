import * as flatbuffers from 'flatbuffers';
import { Input, InputT } from './input.js';
import { InputSetOp } from './input-set-op.js';
export declare class InputSetOpV1 implements flatbuffers.IUnpackableObject<InputSetOpV1T> {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): InputSetOpV1;
    static getRootAsInputSetOpV1(bb: flatbuffers.ByteBuffer, obj?: InputSetOpV1): InputSetOpV1;
    static getSizePrefixedRootAsInputSetOpV1(bb: flatbuffers.ByteBuffer, obj?: InputSetOpV1): InputSetOpV1;
    id(): string | null;
    id(optionalEncoding: flatbuffers.Encoding): string | Uint8Array | null;
    op(): InputSetOp;
    mutate_op(value: InputSetOp): boolean;
    inputs(index: number, obj?: Input): Input | null;
    inputsLength(): number;
    static startInputSetOpV1(builder: flatbuffers.Builder): void;
    static addId(builder: flatbuffers.Builder, idOffset: flatbuffers.Offset): void;
    static addOp(builder: flatbuffers.Builder, op: InputSetOp): void;
    static addInputs(builder: flatbuffers.Builder, inputsOffset: flatbuffers.Offset): void;
    static createInputsVector(builder: flatbuffers.Builder, data: flatbuffers.Offset[]): flatbuffers.Offset;
    static startInputsVector(builder: flatbuffers.Builder, numElems: number): void;
    static endInputSetOpV1(builder: flatbuffers.Builder): flatbuffers.Offset;
    static finishInputSetOpV1Buffer(builder: flatbuffers.Builder, offset: flatbuffers.Offset): void;
    static finishSizePrefixedInputSetOpV1Buffer(builder: flatbuffers.Builder, offset: flatbuffers.Offset): void;
    static createInputSetOpV1(builder: flatbuffers.Builder, idOffset: flatbuffers.Offset, op: InputSetOp, inputsOffset: flatbuffers.Offset): flatbuffers.Offset;
    unpack(): InputSetOpV1T;
    unpackTo(_o: InputSetOpV1T): void;
}
export declare class InputSetOpV1T implements flatbuffers.IGeneratedObject {
    id: string | Uint8Array | null;
    op: InputSetOp;
    inputs: (InputT)[];
    constructor(id?: string | Uint8Array | null, op?: InputSetOp, inputs?: (InputT)[]);
    pack(builder: flatbuffers.Builder): flatbuffers.Offset;
}
