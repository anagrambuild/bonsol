"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Execute = void 0;
const tslib_1 = require("tslib");
const bonsol_schemas_1 = require("bonsol-schemas");
const flatbuffers = tslib_1.__importStar(require("flatbuffers"));
const web3_js_1 = require("@solana/web3.js");
const _1 = require(".");
async function Execute(params) {
    const { executionId, imageId, callbackProgramId, callBackInstructionPrefix, inputs, requester, payer } = params;
    const builder = new flatbuffers.Builder(0);
    const execId = builder.createString(executionId);
    const imgId = builder.createString(imageId);
    let cb, prf;
    if (callbackProgramId && callBackInstructionPrefix) {
        cb = builder.createString(callbackProgramId);
        prf = builder.createString(callBackInstructionPrefix);
    }
    let extra_accounts = [];
    let accnt_offset = 6;
    let fbb_inputs = [];
    for (let i of inputs) {
        if (i.inputType == bonsol_schemas_1.InputType.InputSet) {
            extra_accounts.push({
                address: (0, web3_js_1.address)(i.input.toString()), //must be input set account
                role: web3_js_1.AccountRole.READONLY
            });
            i.input = Uint8Array.from([accnt_offset]);
            accnt_offset += 1;
        }
        let off = bonsol_schemas_1.Input.createDataVector(builder, typeof i.input === "string" ? Buffer.from(i.input, "utf-8") : i.input);
        let ioff = bonsol_schemas_1.Input.createInput(builder, i.inputType, off);
        fbb_inputs.push(ioff);
    }
    let iv = bonsol_schemas_1.ExecutionRequestV1.createInputVector(builder, fbb_inputs);
    bonsol_schemas_1.ExecutionRequestV1.startExecutionRequestV1(builder);
    bonsol_schemas_1.ExecutionRequestV1.addInput(builder, iv);
    bonsol_schemas_1.ExecutionRequestV1.addExecutionId(builder, execId);
    bonsol_schemas_1.ExecutionRequestV1.addImageId(builder, imgId);
    bonsol_schemas_1.ExecutionRequestV1.addMaxBlockHeight(builder, BigInt(params.expiry || 0));
    if (cb) {
        bonsol_schemas_1.ExecutionRequestV1.addCallbackProgramId(builder, cb);
    }
    if (prf) {
        bonsol_schemas_1.ExecutionRequestV1.addCallbackInstructionPrefix(builder, prf);
    }
    bonsol_schemas_1.ExecutionRequestV1.addTip(builder, BigInt(params.tip || 0));
    bonsol_schemas_1.ExecutionRequestV1.addVerifyInputHash(builder, params.verifyInputHash || false);
    bonsol_schemas_1.ExecutionRequestV1.addForwardOutput(builder, params.forwardOuput || false);
    const er = bonsol_schemas_1.ExecutionRequestV1.endExecutionRequestV1(builder);
    builder.finish(er);
    const erbuf = builder.asUint8Array();
    const erv = bonsol_schemas_1.ChannelInstruction.createExecuteV1Vector(builder, erbuf);
    bonsol_schemas_1.ChannelInstruction.startChannelInstruction(builder);
    bonsol_schemas_1.ChannelInstruction.addExecuteV1(builder, erv);
    bonsol_schemas_1.ChannelInstruction.addIxType(builder, bonsol_schemas_1.ChannelInstructionIxType.ExecuteV1);
    const ci = bonsol_schemas_1.ChannelInstruction.endChannelInstruction(builder);
    builder.finish(ci);
    const buf = builder.asUint8Array();
    return {
        accounts: [
            {
                address: requester,
                role: web3_js_1.AccountRole.WRITABLE_SIGNER,
            },
            {
                address: payer || requester,
                role: web3_js_1.AccountRole.WRITABLE_SIGNER,
            },
            {
                address: await (0, _1.executionAddress)(requester, executionId),
                role: web3_js_1.AccountRole.WRITABLE,
            },
            {
                address: await (0, _1.deploymentAddress)(imageId),
                role: web3_js_1.AccountRole.READONLY,
            },
            {
                address: callbackProgramId || (0, web3_js_1.address)(_1.BONSOL_PROGRAM_ID),
                role: web3_js_1.AccountRole.READONLY,
            },
            {
                address: (0, web3_js_1.address)("11111111111111111111111111111111"),
                role: web3_js_1.AccountRole.READONLY,
            },
        ],
        programAddress: (0, web3_js_1.address)(_1.BONSOL_PROGRAM_ID),
        data: buf,
    };
}
exports.Execute = Execute;
//# sourceMappingURL=execute.js.map