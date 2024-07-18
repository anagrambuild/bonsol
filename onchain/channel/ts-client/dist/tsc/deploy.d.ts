import { Address, IInstruction, ReadonlyAccount, WritableAccount, WritableSignerAccount } from "@solana/web3.js";
import { ProgramInputType } from "bonsol-schemas";
export declare function Deploy(params: {
    imageUrl: string;
    imageId: string;
    imageSize: bigint;
    programName: string;
    deployer: Address;
    payer?: Address;
    inputs: ProgramInputType[];
}): Promise<IInstruction<Address, [
    WritableSignerAccount,
    WritableSignerAccount,
    WritableAccount,
    ReadonlyAccount
]>>;
