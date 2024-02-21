import * as flatbuffers from 'flatbuffers';
import { ChannelInstructionData } from './channel-instruction-data.js';
export declare class ChannelInstruction {
    bb: flatbuffers.ByteBuffer | null;
    bb_pos: number;
    __init(i: number, bb: flatbuffers.ByteBuffer): ChannelInstruction;
    static getRootAsChannelInstruction(bb: flatbuffers.ByteBuffer, obj?: ChannelInstruction): ChannelInstruction;
    static getSizePrefixedRootAsChannelInstruction(bb: flatbuffers.ByteBuffer, obj?: ChannelInstruction): ChannelInstruction;
    instructionType(): ChannelInstructionData;
    instruction<T extends flatbuffers.Table>(obj: any): any | null;
    static startChannelInstruction(builder: flatbuffers.Builder): void;
    static addInstructionType(builder: flatbuffers.Builder, instructionType: ChannelInstructionData): void;
    static addInstruction(builder: flatbuffers.Builder, instructionOffset: flatbuffers.Offset): void;
    static endChannelInstruction(builder: flatbuffers.Builder): flatbuffers.Offset;
    static finishChannelInstructionBuffer(builder: flatbuffers.Builder, offset: flatbuffers.Offset): void;
    static finishSizePrefixedChannelInstructionBuffer(builder: flatbuffers.Builder, offset: flatbuffers.Offset): void;
    static createChannelInstruction(builder: flatbuffers.Builder, instructionType: ChannelInstructionData, instructionOffset: flatbuffers.Offset): flatbuffers.Offset;
}
