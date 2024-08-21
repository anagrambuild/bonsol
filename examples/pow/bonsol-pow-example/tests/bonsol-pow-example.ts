import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { BonsolPowExample } from "../target/types/bonsol_pow_example";

describe("bonsol-pow-example", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.BonsolPowExample as Program<BonsolPowExample>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
