import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Keypair, PublicKey, SendTransactionError } from "@solana/web3.js";
import { expect } from "chai";

import { Swap } from "../target/types/swap";
import { createMint } from "@solana/spl-token";

describe("create_token_vault", async () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Swap as Program<Swap>;
  const deployer = provider.wallet as anchor.Wallet;
  const otherUser = Keypair.generate();

  // find the swap account
  const [swapRegistry] = await PublicKey.findProgramAddress(
    [anchor.utils.bytes.utf8.encode("SEED::SWAP::PLATFORM")],
    program.programId
  );

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

    [swapTokenVault] = await PublicKey.findProgramAddress(
      [
        anchor.utils.bytes.utf8.encode("SEED::SWAP::TOKEN_VAULT_SEED"),
        mintNormalPublicKey.toBytes(),
      ],
      program.programId
    );
  });

  it("[create_token_vault] should: non-deployer fails to create a token vault", async () => {
    try {
      // create the token vault
      await program.methods
        .createTokenVault()
        .accounts({
          mintAccount: mintNormalPublicKey,
          swapRegistry,
          swapTokenVault,
          signer: otherUser.publicKey,
        })
        .signers([otherUser])
        .rpc({ commitment: "confirmed" });

      throw new Error("Failed");
    } catch (e) {
      expect(e instanceof SendTransactionError).to.be.true;
    }
  });

  it("[create_token_vault] should: deployer creates a token vault successfully", async () => {
    // create the token vault
    await program.methods
      .createTokenVault()
      .accounts({
        mintAccount: mintNormalPublicKey,
        swapRegistry,
        swapTokenVault,
        signer: deployer.publicKey,
      })
      .signers([deployer.payer])
      .rpc({ commitment: "confirmed" });

    // check the state
    const state = await program.account.swapPlatformRegistry.fetch(
      swapRegistry
    );
    // @ts-ignore
    expect(state.allowedMintAccounts.length).equals(1);
    expect(state.allowedMintAccounts[0].isEnabled).equals(true);
    expect(
      state.allowedMintAccounts[0].mintAccount.equals(mintNormalPublicKey)
    ).equals(true);
    expect(!!state.allowedMintAccounts[0].tokenAccount).to.be.true;
  });

  it("[create_token_vault] should: deployer fails to create a token vault for an added mint account", async () => {
    try {
      // create the token vault
      await program.methods
        .createTokenVault()
        .accounts({
          mintAccount: mintNormalPublicKey,
          swapRegistry,
          swapTokenVault,
          signer: deployer.publicKey,
        })
        .signers([deployer.payer])
        .rpc({ commitment: "confirmed" });

      throw new Error("Failed");
    } catch (e) {
      expect(e instanceof SendTransactionError).to.be.true;
    }
  });
});
