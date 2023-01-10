import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SeahorseAuction } from "../target/types/seahorse_auction";

describe("seahorse_auction", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.SeahorseAuction as Program<SeahorseAuction>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
