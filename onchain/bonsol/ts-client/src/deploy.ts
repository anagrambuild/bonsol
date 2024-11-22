import { AccountRole, Address, IInstruction, ReadonlyAccount, WritableAccount, WritableSignerAccount, address, getAddressDecoder, getAddressEncoder } from "@solana/web3.js";
import { ChannelInstruction, ChannelInstructionIxType, DeployV1 } from "bonsol-schemas";
import { ProgramInputType } from "bonsol-schemas";
import * as flatbuffers from "flatbuffers"
import { BONSOL_PROGRAM_ID, deploymentAddress } from ".";



export async function Deploy(params: {
  imageUrl: string,
  imageId: string,
  imageSize: bigint,
  programName: string,
  deployer: Address,
  payer?: Address,
  inputs: ProgramInputType[],
}): Promise<IInstruction<
  Address,
  [
    WritableSignerAccount,
    WritableSignerAccount,
    WritableAccount,
    ReadonlyAccount
  ]
>> {
  const {
    imageUrl,
    imageId,
    programName,
    deployer,
    payer,
    inputs,
    imageSize
  } = params;
  //eunsure valid url
  if (!URL.canParse(imageUrl)) {
    throw new Error("Invalid URL")
  }
  const builder = new flatbuffers.Builder(0);
  const imgId = builder.createString(imageId)
  const imgName = builder.createString(programName)
  const imgURL = builder.createString(imageUrl)
  const ownerBytes = getAddressEncoder().encode(deployer);
  //@ts-ignore
  const ownerBuf = DeployV1.createOwnerVector(builder, ownerBytes);

  let iv = DeployV1.createInputsVector(builder, inputs);
  DeployV1.startDeployV1(builder);
  DeployV1.addUrl(builder, imgURL)
  DeployV1.addSize(builder, imageSize);
  DeployV1.addImageId(builder, imgId);
  DeployV1.addProgramName(builder, imgName);
  DeployV1.addInputs(builder, iv);
  DeployV1.addOwner(builder, ownerBuf);
  const deploy = DeployV1.endDeployV1(builder);
  builder.finish(deploy);
  const deployData = builder.asUint8Array();
  const erv = ChannelInstruction.createDeployV1Vector(builder, deployData);
  ChannelInstruction.startChannelInstruction(builder);
  ChannelInstruction.addDeployV1(builder, erv);
  ChannelInstruction.addIxType(builder, ChannelInstructionIxType.DeployV1);
  const ci = ChannelInstruction.endChannelInstruction(builder);
  builder.finish(ci);
  const buf = builder.asUint8Array();
  return {
    accounts: [
      {
        address: deployer,
        role: AccountRole.WRITABLE_SIGNER,
      },
      {
        address: payer || deployer,
        role: AccountRole.WRITABLE_SIGNER,
      },
      {
        address: await deploymentAddress(imageId),
        role: AccountRole.WRITABLE,
      },
      {
        address: address("11111111111111111111111111111111"),
        role: AccountRole.READONLY,
      }
    ],
    programAddress: address(BONSOL_PROGRAM_ID),
    data: buf,
  };
}
