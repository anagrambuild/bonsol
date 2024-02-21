import * as flatbuffers from 'flatbuffers';
export declare class StatusV1 {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): StatusV1;
    static getRootAsStatusV1(bb: flatbuffers.ByteBuffer, obj?: StatusV1): StatusV1;
    static getSizePrefixedRootAsStatusV1(bb: flatbuffers.ByteBuffer, obj?: StatusV1): StatusV1;
    status(): number;
    mutate_status(value: number): boolean;
    message(index: number): number | null;
    messageLength(): number;
    messageArray(): Uint8Array | null;
    static startStatusV1(builder: flatbuffers.Builder): void;
    static addStatus(builder: flatbuffers.Builder, status: number): void;
    static addMessage(builder: flatbuffers.Builder, messageOffset: flatbuffers.Offset): void;
    static createMessageVector(builder: flatbuffers.Builder, data: number[] | Uint8Array): flatbuffers.Offset;
    static startMessageVector(builder: flatbuffers.Builder, numElems: number): void;
    static endStatusV1(builder: flatbuffers.Builder): flatbuffers.Offset;
    static createStatusV1(builder: flatbuffers.Builder, status: number, messageOffset: flatbuffers.Offset): flatbuffers.Offset;
}
