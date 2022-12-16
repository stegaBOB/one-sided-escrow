import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { OneSidedEscrow } from "../target/types/one_sided_escrow";

describe("one-sided-escrow", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.OneSidedEscrow as Program<OneSidedEscrow>;

  it("Is initialized!", async () => {
    // Add your test here.
    // const tx = await program.methods.initialize().rpc();
    // console.log("Your transaction signature", tx);
  });
});
