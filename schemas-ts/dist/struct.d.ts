import * as flatbuffers from 'flatbuffers';
export declare class Struct {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): Struct;
    static getRootAsStruct(bb: flatbuffers.ByteBuffer, obj?: Struct): Struct;
    static getSizePrefixedRootAsStruct(bb: flatbuffers.ByteBuffer, obj?: Struct): Struct;
    static startStruct(builder: flatbuffers.Builder): void;
    static endStruct(builder: flatbuffers.Builder): flatbuffers.Offset;
    static createStruct(builder: flatbuffers.Builder): flatbuffers.Offset;
}
