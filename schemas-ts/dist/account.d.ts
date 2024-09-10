import * as flatbuffers from 'flatbuffers';
export declare class Account {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): Account;
    writable(): boolean;
    mutate_writable(value: boolean): boolean;
    pubkey(index: number): number | null;
    static sizeOf(): number;
    static createAccount(builder: flatbuffers.Builder, writable: boolean, pubkey: number[] | null): flatbuffers.Offset;
}
