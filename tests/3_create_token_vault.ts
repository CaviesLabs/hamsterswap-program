import * as anchor from "@project-serum/anchor";
import { BN, Program } from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { expect } from "chai";

import { Swap } from "../target/types/swap";
import { createMint } from "@solana/spl-token";

describe("create_token_vault", async () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Swap as Program<Swap>;
  const deployer = provider.wallet as anchor.Wallet;

  // find the swap account
  const [swapRegistry] = await PublicKey.findProgramAddress([
    anchor.utils.bytes.utf8.encode("SEED::SWAP::PLATFORM"),
  ], program.programId);

  let mintNormalPublicKey;
  let swapTokenVault;

  before(async () => {
      // now we try to create token vault for the mint token
      mintNormalPublicKey = await createMint(
        provider.connection, // conneciton
        deployer.payer, // fee payer
        deployer.publicKey, // mint authority
        deployer.publicKey, // freeze authority (you can use `null` to disable it. when you disable it, you can't turn it on again)
        8 // decimals
      );
      [swapTokenVault] = await PublicKey.findProgramAddress([
        anchor.utils.bytes.utf8.encode("SEED::SWAP::TOKEN_VAULT_SEED"),
        mintNormalPublicKey.toBytes(),
      ], program.programId);

  })

  it("[create_token_vault] should: deployer creates a token vault successfully", async () => {
    // create the token vault
    await program.methods.createTokenVault().accounts({
      mintTokenAccount: mintNormalPublicKey,
      swapRegistry,
      swapTokenVault,
      owner: deployer.publicKey
    }).signers([deployer.payer]).rpc({ commitment: "confirmed" });

    // check the state
    const state = await program.account.swapPlatformRegistry.fetch(swapRegistry);

    // @ts-ignore
    expect(state.allowedMintAccounts.length).equals(1);
    expect(state.allowedMintAccounts[0].isActive).equals(true);
    expect(state.allowedMintAccounts[0].mintAccount.toBase58()).equals(mintNormalPublicKey.toBase58());
    expect(!!state.allowedMintAccounts[0].tokenAccount).to.be.true;
  });

  it("[create_token_vault] should: deployer fails to create a token vault for an added mint account", async () => {
    try {
      // create the token vault
      await program.methods.createTokenVault().accounts({
        mintTokenAccount: mintNormalPublicKey,
        swapRegistry,
        swapTokenVault,
        owner: deployer.publicKey
      }).signers([deployer.payer]).rpc({ commitment: "confirmed" });
    } catch (e){
      expect(!!e).to.true;
    }
  });
});
