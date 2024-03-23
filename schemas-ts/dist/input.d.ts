import * as flatbuffers from 'flatbuffers';
import { InputType } from './input-type.js';
export declare class Input {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): Input;
    static getRootAsInput(bb: flatbuffers.ByteBuffer, obj?: Input): Input;
    static getSizePrefixedRootAsInput(bb: flatbuffers.ByteBuffer, obj?: Input): Input;
    inputType(): InputType;
    mutate_input_type(value: InputType): boolean;
    data(index: number): number | null;
    dataLength(): number;
    dataArray(): Uint8Array | null;
    static startInput(builder: flatbuffers.Builder): void;
    static addInputType(builder: flatbuffers.Builder, inputType: InputType): void;
    static addData(builder: flatbuffers.Builder, dataOffset: flatbuffers.Offset): void;
    static createDataVector(builder: flatbuffers.Builder, data: number[] | Uint8Array): flatbuffers.Offset;
    static startDataVector(builder: flatbuffers.Builder, numElems: number): void;
    static endInput(builder: flatbuffers.Builder): flatbuffers.Offset;
    static createInput(builder: flatbuffers.Builder, inputType: InputType, dataOffset: flatbuffers.Offset): flatbuffers.Offset;
}
