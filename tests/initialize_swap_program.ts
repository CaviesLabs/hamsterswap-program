import * as anchor from "@project-serum/anchor";
import { BN, Program } from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { expect } from "chai";

import { Swap } from "../target/types/swap";

describe("initialize swap program", async () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Swap as Program<Swap>;
  const deployer = provider.wallet as anchor.Wallet;

  // find the swap account
  const [swapAccount] = await PublicKey.findProgramAddress([
    anchor.utils.bytes.utf8.encode("SEED::SWAP::PLATFORM"),
    deployer.publicKey.toBuffer()
  ], program.programId);

  it("should: deployer should initialize successfully", async () => {
    // Initialize first
    await program.methods.initialize({
      maxAllowedItems: new BN(5).toNumber(),
      maxAllowedOptions: new BN(5).toNumber(),
      allowedMintAccounts: [deployer.publicKey]
    }).accounts({
      swapConfig: swapAccount,
      owner: deployer.publicKey
    }).signers([deployer.payer]).rpc({ commitment: "confirmed" });

    const state = await program.account.swapPlatformConfig.fetch(swapAccount);

    // Expect conditions
    expect(state.owner.equals(deployer.publicKey));
    expect(state.wasInitialized).equals(true);
    expect(state.maxAllowedItems).equals(5);
    expect(state.maxAllowedOptions).equals(5);
    expect(state.allowedMintAccounts.length).equals(1);
    expect(state.allowedMintAccounts.map(elm => elm.toString()).includes(deployer.publicKey.toString())).equals(true);
  });

  it("should: cannot re-initialize", async () => {
    try {
      await program.methods.initialize({
        maxAllowedItems: new BN(6).toNumber(),
        maxAllowedOptions: new BN(5).toNumber(),
        allowedMintAccounts: [deployer.publicKey]
      }).accounts({
        swapConfig: swapAccount,
        owner: deployer.publicKey
      }).signers([deployer.payer]).rpc({ commitment: "confirmed" });

      throw new Error("Should be failed");
    } catch (e) {
      expect(!!e).to.be.true;
    }
  });
});
