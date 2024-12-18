"use strict";
// automatically generated by the FlatBuffers compiler, do not modify
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.ExecutionRequestV1T = exports.ExecutionRequestV1 = void 0;
/* eslint-disable @typescript-eslint/no-unused-vars, @typescript-eslint/no-explicit-any, @typescript-eslint/no-non-null-assertion */
const flatbuffers = __importStar(require("flatbuffers"));
const account_js_1 = require("./account.js");
const input_js_1 = require("./input.js");
class ExecutionRequestV1 {
    constructor() {
        this.bb = null;
        this.bb_pos = 0;
    }
    __init(i, bb) {
        this.bb_pos = i;
        this.bb = bb;
        return this;
    }
    static getRootAsExecutionRequestV1(bb, obj) {
        return (obj || new ExecutionRequestV1()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
    }
    static getSizePrefixedRootAsExecutionRequestV1(bb, obj) {
        bb.setPosition(bb.position() + flatbuffers.SIZE_PREFIX_LENGTH);
        return (obj || new ExecutionRequestV1()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
    }
    tip() {
        const offset = this.bb.__offset(this.bb_pos, 4);
        return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt('0');
    }
    mutate_tip(value) {
        const offset = this.bb.__offset(this.bb_pos, 4);
        if (offset === 0) {
            return false;
        }
        this.bb.writeUint64(this.bb_pos + offset, value);
        return true;
    }
    executionId(optionalEncoding) {
        const offset = this.bb.__offset(this.bb_pos, 6);
        return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
    }
    imageId(optionalEncoding) {
        const offset = this.bb.__offset(this.bb_pos, 8);
        return offset ? this.bb.__string(this.bb_pos + offset, optionalEncoding) : null;
    }
    callbackProgramId(index) {
        const offset = this.bb.__offset(this.bb_pos, 10);
        return offset ? this.bb.readUint8(this.bb.__vector(this.bb_pos + offset) + index) : 0;
    }
    callbackProgramIdLength() {
        const offset = this.bb.__offset(this.bb_pos, 10);
        return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
    }
    callbackProgramIdArray() {
        const offset = this.bb.__offset(this.bb_pos, 10);
        return offset ? new Uint8Array(this.bb.bytes().buffer, this.bb.bytes().byteOffset + this.bb.__vector(this.bb_pos + offset), this.bb.__vector_len(this.bb_pos + offset)) : null;
    }
    callbackInstructionPrefix(index) {
        const offset = this.bb.__offset(this.bb_pos, 12);
        return offset ? this.bb.readUint8(this.bb.__vector(this.bb_pos + offset) + index) : 0;
    }
    callbackInstructionPrefixLength() {
        const offset = this.bb.__offset(this.bb_pos, 12);
        return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
    }
    callbackInstructionPrefixArray() {
        const offset = this.bb.__offset(this.bb_pos, 12);
        return offset ? new Uint8Array(this.bb.bytes().buffer, this.bb.bytes().byteOffset + this.bb.__vector(this.bb_pos + offset), this.bb.__vector_len(this.bb_pos + offset)) : null;
    }
    forwardOutput() {
        const offset = this.bb.__offset(this.bb_pos, 14);
        return offset ? !!this.bb.readInt8(this.bb_pos + offset) : false;
    }
    mutate_forward_output(value) {
        const offset = this.bb.__offset(this.bb_pos, 14);
        if (offset === 0) {
            return false;
        }
        this.bb.writeInt8(this.bb_pos + offset, +value);
        return true;
    }
    verifyInputHash() {
        const offset = this.bb.__offset(this.bb_pos, 16);
        return offset ? !!this.bb.readInt8(this.bb_pos + offset) : true;
    }
    mutate_verify_input_hash(value) {
        const offset = this.bb.__offset(this.bb_pos, 16);
        if (offset === 0) {
            return false;
        }
        this.bb.writeInt8(this.bb_pos + offset, +value);
        return true;
    }
    input(index, obj) {
        const offset = this.bb.__offset(this.bb_pos, 18);
        return offset ? (obj || new input_js_1.Input()).__init(this.bb.__indirect(this.bb.__vector(this.bb_pos + offset) + index * 4), this.bb) : null;
    }
    inputLength() {
        const offset = this.bb.__offset(this.bb_pos, 18);
        return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
    }
    inputDigest(index) {
        const offset = this.bb.__offset(this.bb_pos, 20);
        return offset ? this.bb.readUint8(this.bb.__vector(this.bb_pos + offset) + index) : 0;
    }
    inputDigestLength() {
        const offset = this.bb.__offset(this.bb_pos, 20);
        return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
    }
    inputDigestArray() {
        const offset = this.bb.__offset(this.bb_pos, 20);
        return offset ? new Uint8Array(this.bb.bytes().buffer, this.bb.bytes().byteOffset + this.bb.__vector(this.bb_pos + offset), this.bb.__vector_len(this.bb_pos + offset)) : null;
    }
    maxBlockHeight() {
        const offset = this.bb.__offset(this.bb_pos, 22);
        return offset ? this.bb.readUint64(this.bb_pos + offset) : BigInt('0');
    }
    mutate_max_block_height(value) {
        const offset = this.bb.__offset(this.bb_pos, 22);
        if (offset === 0) {
            return false;
        }
        this.bb.writeUint64(this.bb_pos + offset, value);
        return true;
    }
    callbackExtraAccounts(index, obj) {
        const offset = this.bb.__offset(this.bb_pos, 24);
        return offset ? (obj || new account_js_1.Account()).__init(this.bb.__vector(this.bb_pos + offset) + index * 40, this.bb) : null;
    }
    callbackExtraAccountsLength() {
        const offset = this.bb.__offset(this.bb_pos, 24);
        return offset ? this.bb.__vector_len(this.bb_pos + offset) : 0;
    }
    static startExecutionRequestV1(builder) {
        builder.startObject(11);
    }
    static addTip(builder, tip) {
        builder.addFieldInt64(0, tip, BigInt('0'));
    }
    static addExecutionId(builder, executionIdOffset) {
        builder.addFieldOffset(1, executionIdOffset, 0);
    }
    static addImageId(builder, imageIdOffset) {
        builder.addFieldOffset(2, imageIdOffset, 0);
    }
    static addCallbackProgramId(builder, callbackProgramIdOffset) {
        builder.addFieldOffset(3, callbackProgramIdOffset, 0);
    }
    static createCallbackProgramIdVector(builder, data) {
        builder.startVector(1, data.length, 1);
        for (let i = data.length - 1; i >= 0; i--) {
            builder.addInt8(data[i]);
        }
        return builder.endVector();
    }
    static startCallbackProgramIdVector(builder, numElems) {
        builder.startVector(1, numElems, 1);
    }
    static addCallbackInstructionPrefix(builder, callbackInstructionPrefixOffset) {
        builder.addFieldOffset(4, callbackInstructionPrefixOffset, 0);
    }
    static createCallbackInstructionPrefixVector(builder, data) {
        builder.startVector(1, data.length, 1);
        for (let i = data.length - 1; i >= 0; i--) {
            builder.addInt8(data[i]);
        }
        return builder.endVector();
    }
    static startCallbackInstructionPrefixVector(builder, numElems) {
        builder.startVector(1, numElems, 1);
    }
    static addForwardOutput(builder, forwardOutput) {
        builder.addFieldInt8(5, +forwardOutput, +false);
    }
    static addVerifyInputHash(builder, verifyInputHash) {
        builder.addFieldInt8(6, +verifyInputHash, +true);
    }
    static addInput(builder, inputOffset) {
        builder.addFieldOffset(7, inputOffset, 0);
    }
    static createInputVector(builder, data) {
        builder.startVector(4, data.length, 4);
        for (let i = data.length - 1; i >= 0; i--) {
            builder.addOffset(data[i]);
        }
        return builder.endVector();
    }
    static startInputVector(builder, numElems) {
        builder.startVector(4, numElems, 4);
    }
    static addInputDigest(builder, inputDigestOffset) {
        builder.addFieldOffset(8, inputDigestOffset, 0);
    }
    static createInputDigestVector(builder, data) {
        builder.startVector(1, data.length, 1);
        for (let i = data.length - 1; i >= 0; i--) {
            builder.addInt8(data[i]);
        }
        return builder.endVector();
    }
    static startInputDigestVector(builder, numElems) {
        builder.startVector(1, numElems, 1);
    }
    static addMaxBlockHeight(builder, maxBlockHeight) {
        builder.addFieldInt64(9, maxBlockHeight, BigInt('0'));
    }
    static addCallbackExtraAccounts(builder, callbackExtraAccountsOffset) {
        builder.addFieldOffset(10, callbackExtraAccountsOffset, 0);
    }
    static startCallbackExtraAccountsVector(builder, numElems) {
        builder.startVector(40, numElems, 8);
    }
    static endExecutionRequestV1(builder) {
        const offset = builder.endObject();
        return offset;
    }
    static finishExecutionRequestV1Buffer(builder, offset) {
        builder.finish(offset);
    }
    static finishSizePrefixedExecutionRequestV1Buffer(builder, offset) {
        builder.finish(offset, undefined, true);
    }
    static createExecutionRequestV1(builder, tip, executionIdOffset, imageIdOffset, callbackProgramIdOffset, callbackInstructionPrefixOffset, forwardOutput, verifyInputHash, inputOffset, inputDigestOffset, maxBlockHeight, callbackExtraAccountsOffset) {
        ExecutionRequestV1.startExecutionRequestV1(builder);
        ExecutionRequestV1.addTip(builder, tip);
        ExecutionRequestV1.addExecutionId(builder, executionIdOffset);
        ExecutionRequestV1.addImageId(builder, imageIdOffset);
        ExecutionRequestV1.addCallbackProgramId(builder, callbackProgramIdOffset);
        ExecutionRequestV1.addCallbackInstructionPrefix(builder, callbackInstructionPrefixOffset);
        ExecutionRequestV1.addForwardOutput(builder, forwardOutput);
        ExecutionRequestV1.addVerifyInputHash(builder, verifyInputHash);
        ExecutionRequestV1.addInput(builder, inputOffset);
        ExecutionRequestV1.addInputDigest(builder, inputDigestOffset);
        ExecutionRequestV1.addMaxBlockHeight(builder, maxBlockHeight);
        ExecutionRequestV1.addCallbackExtraAccounts(builder, callbackExtraAccountsOffset);
        return ExecutionRequestV1.endExecutionRequestV1(builder);
    }
    unpack() {
        return new ExecutionRequestV1T(this.tip(), this.executionId(), this.imageId(), this.bb.createScalarList(this.callbackProgramId.bind(this), this.callbackProgramIdLength()), this.bb.createScalarList(this.callbackInstructionPrefix.bind(this), this.callbackInstructionPrefixLength()), this.forwardOutput(), this.verifyInputHash(), this.bb.createObjList(this.input.bind(this), this.inputLength()), this.bb.createScalarList(this.inputDigest.bind(this), this.inputDigestLength()), this.maxBlockHeight(), this.bb.createObjList(this.callbackExtraAccounts.bind(this), this.callbackExtraAccountsLength()));
    }
    unpackTo(_o) {
        _o.tip = this.tip();
        _o.executionId = this.executionId();
        _o.imageId = this.imageId();
        _o.callbackProgramId = this.bb.createScalarList(this.callbackProgramId.bind(this), this.callbackProgramIdLength());
        _o.callbackInstructionPrefix = this.bb.createScalarList(this.callbackInstructionPrefix.bind(this), this.callbackInstructionPrefixLength());
        _o.forwardOutput = this.forwardOutput();
        _o.verifyInputHash = this.verifyInputHash();
        _o.input = this.bb.createObjList(this.input.bind(this), this.inputLength());
        _o.inputDigest = this.bb.createScalarList(this.inputDigest.bind(this), this.inputDigestLength());
        _o.maxBlockHeight = this.maxBlockHeight();
        _o.callbackExtraAccounts = this.bb.createObjList(this.callbackExtraAccounts.bind(this), this.callbackExtraAccountsLength());
    }
}
exports.ExecutionRequestV1 = ExecutionRequestV1;
class ExecutionRequestV1T {
    constructor(tip = BigInt('0'), executionId = null, imageId = null, callbackProgramId = [], callbackInstructionPrefix = [], forwardOutput = false, verifyInputHash = true, input = [], inputDigest = [], maxBlockHeight = BigInt('0'), callbackExtraAccounts = []) {
        this.tip = tip;
        this.executionId = executionId;
        this.imageId = imageId;
        this.callbackProgramId = callbackProgramId;
        this.callbackInstructionPrefix = callbackInstructionPrefix;
        this.forwardOutput = forwardOutput;
        this.verifyInputHash = verifyInputHash;
        this.input = input;
        this.inputDigest = inputDigest;
        this.maxBlockHeight = maxBlockHeight;
        this.callbackExtraAccounts = callbackExtraAccounts;
    }
    pack(builder) {
        const executionId = (this.executionId !== null ? builder.createString(this.executionId) : 0);
        const imageId = (this.imageId !== null ? builder.createString(this.imageId) : 0);
        const callbackProgramId = ExecutionRequestV1.createCallbackProgramIdVector(builder, this.callbackProgramId);
        const callbackInstructionPrefix = ExecutionRequestV1.createCallbackInstructionPrefixVector(builder, this.callbackInstructionPrefix);
        const input = ExecutionRequestV1.createInputVector(builder, builder.createObjectOffsetList(this.input));
        const inputDigest = ExecutionRequestV1.createInputDigestVector(builder, this.inputDigest);
        const callbackExtraAccounts = builder.createStructOffsetList(this.callbackExtraAccounts, ExecutionRequestV1.startCallbackExtraAccountsVector);
        return ExecutionRequestV1.createExecutionRequestV1(builder, this.tip, executionId, imageId, callbackProgramId, callbackInstructionPrefix, this.forwardOutput, this.verifyInputHash, input, inputDigest, this.maxBlockHeight, callbackExtraAccounts);
    }
}
exports.ExecutionRequestV1T = ExecutionRequestV1T;
