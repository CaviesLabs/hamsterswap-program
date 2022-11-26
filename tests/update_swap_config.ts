import * as anchor from "@project-serum/anchor";
import { BN, BorshCoder, EventParser, Program } from "@project-serum/anchor";
import { PublicKey, Keypair } from "@solana/web3.js";
import { expect } from "chai";

import { Swap } from "../target/types/swap";

describe("update_swap_program", async () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Swap as Program<Swap>;
  const deployer = provider.wallet as anchor.Wallet;
  const otherUser = Keypair.generate();
  const sampleMintToken = Keypair.generate();

  // find the swap account
  const [swapAccount] = await PublicKey.findProgramAddress([
    anchor.utils.bytes.utf8.encode("SEED::SWAP::PLATFORM"),
  ], program.programId);

  it("[update_swap_program] should: deployer update config successfully", async () => {
    // Initialize first
    const tx = await program.methods.updateSwapRegistry({
      maxAllowedItems: new BN(3).toNumber(),
      maxAllowedOptions: new BN(3).toNumber(),
    }).accounts({
      swapRegistry: swapAccount,
      owner: deployer.publicKey
    }).signers([deployer.payer]).rpc({ commitment: "confirmed" });

    const state = await program.account.swapPlatformRegistry.fetch(swapAccount);

    // Expect conditions
    expect(state.owner.equals(deployer.publicKey));
    expect(state.wasInitialized).equals(true);
    expect(state.maxAllowedItems).equals(3);
    expect(state.maxAllowedOptions).equals(3);
    expect(state.allowedMintAccounts.length).equals(0);

    // expect eventLog
    const transaction = await provider.connection.getParsedTransaction(tx, { commitment: "confirmed" });
    const eventParser = new EventParser(program.programId, new BorshCoder(program.idl));
    const [event] = eventParser.parseLogs(transaction.meta.logMessages);

    // Expect emitted logs
    expect(event.data.owner.toString() === deployer.publicKey.toString()).equals(true);
    expect(event.data.maxAllowedOptions === 3).equals(true);
    expect(event.data.maxAllowedItems === 3).equals(true);
  });


  it("[update_swap_program] should: non-owner cannot modify the swap program", async () => {
    try {
      await program.methods.updateSwapRegistry({
        maxAllowedItems: new BN(6).toNumber(),
        maxAllowedOptions: new BN(5).toNumber(),
      }).accounts({
        swapRegistry: swapAccount,
        owner: otherUser.publicKey
      }).signers([otherUser]).rpc({ commitment: "confirmed" });

      throw new Error("Should be failed");
    } catch (e) {
      expect(!!e).to.be.true;
    }
  });

  it("[update_swap_program] should: cannot update invalid values", async () => {
    try {
      await program.methods.updateSwapRegistry({
        maxAllowedItems: new BN(0).toNumber(),
        maxAllowedOptions: new BN(5).toNumber(),
      }).accounts({
        swapRegistry: swapAccount,
        owner: deployer.publicKey
      }).signers([deployer.payer]).rpc({ commitment: "confirmed" });

      throw new Error("Should be failed");
    } catch (e) {
      expect(!!e).to.be.true;
    }
  });
});
