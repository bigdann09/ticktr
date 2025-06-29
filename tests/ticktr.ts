import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Ticktr } from "../target/types/ticktr";

describe("ticktr", () => {
  // Configure the client to use the local cluster.
  let provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ticktr as Program<Ticktr>;

  const manager = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("manager")],
    program.programId
  )[0];

  it("setup manager account", async () => {
    // Add your test here.
    const tx = await program.methods.setupManager()
    .accountsPartial({
      signer: provider.wallet.publicKey,
      payer: provider.wallet.publicKey,
      manager,
      systemProgram: anchor.web3.SystemProgram.programId
    })
    .rpc();

    console.log(tx);
  });
});
