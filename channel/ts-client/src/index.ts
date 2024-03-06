import { ChannelInstruction, ExecutionInputType, ChannelInstructionIxType, ExecutionRequestV1 } from "bonsol-schemas"
import * as flatbuffers from "flatbuffers"
import { AccountRole, Address, IInstruction, ReadonlyAccount, WritableAccount, address, getAddressEncoder, getProgramDerivedAddress } from "@solana/web3.js";
import { unionToChannelInstructionData } from "bonsol-schemas/dist/bonsol/channel-instruction-data";
export const BONSOL_PROGRAM_ID = "BoNSrwTtTM4PRkbbPvehk1XzHC65cKfdNSod9FyTejRn";


export class BonsolProgram {
  static async Execute(params: {
    executionId: string,
    imageId: string,
    inputType: ExecutionInputType,
    input: Uint8Array,
    requester: Address,
    callbackProgramId?: Address,
    callBackInstructionPrefix?: Uint8Array,
    tip?: number
  }): Promise<IInstruction<
    Address,
    [
      WritableAccount,
      WritableAccount,
      ReadonlyAccount,
      ReadonlyAccount
    ]
  >> {
    const {
      executionId,
      imageId,
      callbackProgramId,
      callBackInstructionPrefix,
      inputType,
      input,
      requester
    } = params;
    const builder = new flatbuffers.Builder(0);
    const execId = builder.createString(executionId)
    const imgId = builder.createString(imageId)
    let cb, prf;
    if (callbackProgramId && callBackInstructionPrefix) {
      cb = builder.createString(callbackProgramId)
      prf = builder.createString(callBackInstructionPrefix) 
    }
    ExecutionRequestV1.startInputDataVector(builder, input.length);
    const ind = ExecutionRequestV1.createInputDataVector(builder, input)
    ExecutionRequestV1.startExecutionRequestV1(builder);
    ExecutionRequestV1.addExecutionId(builder, execId);
    ExecutionRequestV1.addImageId(builder, imgId);
    if (cb) {
      ExecutionRequestV1.addCallbackProgramId(builder, cb);
    }
    if (prf) {
      ExecutionRequestV1.addCallbackInstructionPrefix(builder, prf);
    }
    ExecutionRequestV1.addInputType(builder, inputType);
    ExecutionRequestV1.addInputData(builder, ind);
    ExecutionRequestV1.addTip(builder, BigInt(params.tip || 0));
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
          role: AccountRole.WRITABLE,
        },
        {
          address: await executionAddress(requester, executionId),
          role: AccountRole.WRITABLE,
        },
        {
          address: callbackProgramId || address(BONSOL_PROGRAM_ID),
          role: AccountRole.READONLY,
        },
        {
          address: address("11111111111111111111111111111111"),
          role: AccountRole.READONLY,
        }
      ],
      programAddress: address(BONSOL_PROGRAM_ID),
      data: buf,
    }
  }
}

export async function executionAddress(
  requester: Address,
  executionId: string
): Promise<Address> {
  const addressEncoder = getAddressEncoder();
  const pd = await getProgramDerivedAddress({
    programAddress: address(BONSOL_PROGRAM_ID),
    seeds: [
      Buffer.from("execution", "utf-8"),
      addressEncoder.encode(requester),
      Buffer.from(executionId, "utf-8")
    ]
  });
  return pd[0];
}