import { InputType } from "bonsol-schemas";
import { Address, IAccountMeta, IInstruction } from "@solana/web3.js";
export type ProgramInput = {
    inputType: InputType;
    input: Uint8Array | string;
};
export declare function Execute(params: {
    executionId: string;
    imageId: string;
    inputs: ProgramInput[];
    requester: Address;
    payer?: Address;
    callbackProgramId?: Address;
    callBackInstructionPrefix?: Uint8Array;
    expiry?: number | bigint;
    tip?: number;
    verifyInputHash?: boolean;
    forwardOuput?: boolean;
}): Promise<IInstruction<Address, IAccountMeta[]>>;
