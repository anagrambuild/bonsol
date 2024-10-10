import { ChannelInstruction, ChannelInstructionIxType, ExecutionRequestV1, Input, InputType } from "bonsol-schemas"
import * as flatbuffers from "flatbuffers"
import { AccountRole, Address, IAccountMeta, IInstruction, ReadonlyAccount, WritableAccount, address } from "@solana/web3.js";
import { BONSOL_PROGRAM_ID, deploymentAddress, executionAddress } from ".";

export type ProgramInput = {
  inputType: InputType,
  input: Uint8Array | string
}
export async function Execute(params: {
  executionId: string,
  imageId: string,
  inputs: ProgramInput[],
  requester: Address,
  payer?: Address,
  callbackProgramId?: Address,
  callBackInstructionPrefix?: Uint8Array,
  expiry?: number | bigint,
  tip?: number,
  verifyInputHash?: boolean,
  forwardOuput?: boolean
}): Promise<IInstruction<
  Address,
  IAccountMeta[]
>> {
  const {
    executionId,
    imageId,
    callbackProgramId,
    callBackInstructionPrefix,
    inputs,
    requester,
    payer
  } = params;
  const builder = new flatbuffers.Builder(0);
  const execId = builder.createString(executionId)
  const imgId = builder.createString(imageId)
  let cb, prf;
  if (callbackProgramId && callBackInstructionPrefix) {
    cb = builder.createString(callbackProgramId)
    prf = builder.createString(callBackInstructionPrefix)
  }
  let extra_accounts = [];
  let accnt_offset = 6;
  let fbb_inputs = [];
  
  for (let i of inputs) {
    if (i.inputType == InputType.InputSet) {
      extra_accounts.push({
        address: address(i.input.toString()), //must be input set account
        role: AccountRole.READONLY
      })
      i.input = Uint8Array.from([accnt_offset]);
      accnt_offset += 1;
    }
    let off = Input.createDataVector(builder, typeof i.input === "string" ? Buffer.from(i.input, "utf-8") : i.input)
    let ioff = Input.createInput(builder, i.inputType, off)
    fbb_inputs.push(ioff);
  }
  let iv = ExecutionRequestV1.createInputVector(builder, fbb_inputs);
  ExecutionRequestV1.startExecutionRequestV1(builder);
  ExecutionRequestV1.addInput(builder, iv);
  ExecutionRequestV1.addExecutionId(builder, execId);
  ExecutionRequestV1.addImageId(builder, imgId);
  ExecutionRequestV1.addMaxBlockHeight(builder, BigInt(params.expiry || 0));
  if (cb) {
    ExecutionRequestV1.addCallbackProgramId(builder, cb);
  }
  if (prf) {
    ExecutionRequestV1.addCallbackInstructionPrefix(builder, prf);
  }
  ExecutionRequestV1.addTip(builder, BigInt(params.tip || 0));
  ExecutionRequestV1.addVerifyInputHash(builder, params.verifyInputHash || false);
  ExecutionRequestV1.addForwardOutput(builder, params.forwardOuput || false);
  const er = ExecutionRequestV1.endExecutionRequestV1(builder);
  builder.finish(er);
  const erbuf = builder.asUint8Array();
  const erv = ChannelInstruction.createExecuteV1Vector(builder, erbuf);
  ChannelInstruction.startChannelInstruction(builder);
  ChannelInstruction.addExecuteV1(builder, erv);
  ChannelInstruction.addIxType(builder, ChannelInstructionIxType.ExecuteV1);
  const ci = ChannelInstruction.endChannelInstruction(builder);
  builder.finish(ci);
  const buf = builder.asUint8Array();

  return {
    accounts: [
      {
        address: requester,
        role: AccountRole.WRITABLE_SIGNER,
      },
      {
        address: payer || requester,
        role: AccountRole.WRITABLE_SIGNER,
      },
      {
        address: await executionAddress(requester, executionId),
        role: AccountRole.WRITABLE,
      },
      {
        address: await deploymentAddress(imageId),
        role: AccountRole.READONLY,
      },
      {
        address: callbackProgramId || address(BONSOL_PROGRAM_ID),
        role: AccountRole.READONLY,
      },
      {
        address: address("11111111111111111111111111111111"),
        role: AccountRole.READONLY,
      },
      ...extra_accounts
    ],
    programAddress: address(BONSOL_PROGRAM_ID),
    data: buf,
  }
}


