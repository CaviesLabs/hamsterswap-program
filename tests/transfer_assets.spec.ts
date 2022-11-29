import * as anchor from "@project-serum/anchor";
import { BN, Program, web3 } from "@project-serum/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  createMint,
  mintTo,
  getOrCreateAssociatedTokenAccount,
  getAccount
} from "@solana/spl-token";
import { expect } from "chai";

import { Swap } from "../target/types/swap";

describe("transfer_assets", async () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Swap as Program<Swap>;
  const deployer = provider.wallet as anchor.Wallet;

  // find the swap account
  const [swapRegistry] = await PublicKey.findProgramAddress([
    anchor.utils.bytes.utf8.encode("SEED::SWAP::PLATFORM")
  ], program.programId);

  let mintNormalPublicKey;
  let swapTokenVault;
  let proposalOwner;
  let proposalId;
  let swapProposal;
  let participant;
  let offeredItems;
  let swapOptions;
  let proposalOwnerTokenAccount;
  let participantTokenAccount;
  let expiredAt;
  let swapTokenVaultBump;

  // Construct accounts for proposal creation
  proposalOwner = Keypair.generate();
  participant = Keypair.generate();

  before(async () => {
    // funding proposal owner
    const airdropSignature = await provider.connection.requestAirdrop(proposalOwner.publicKey, web3.LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction({
      signature: airdropSignature,
      blockhash: (await provider.connection.getLatestBlockhash()).blockhash,
      lastValidBlockHeight: ((await provider.connection.getLatestBlockhash())).lastValidBlockHeight
    });

    // funding proposal owner
    const airdropForParticipantSignature = await provider.connection.requestAirdrop(participant.publicKey, web3.LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction({
      signature: airdropForParticipantSignature,
      blockhash: (await provider.connection.getLatestBlockhash()).blockhash,
      lastValidBlockHeight: ((await provider.connection.getLatestBlockhash())).lastValidBlockHeight
    });

    // now we try to create token vault for the mint token
    mintNormalPublicKey = await createMint(
      provider.connection, // conneciton
      deployer.payer, // fee payer
      deployer.publicKey, // mint authority
      deployer.publicKey, // freeze authority (you can use `null` to disable it. when you disable it, you can't turn it on again)
      9 // decimals
    );

    proposalOwnerTokenAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      proposalOwner,
      mintNormalPublicKey,
      proposalOwner.publicKey
    );

    participantTokenAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      participant,
      mintNormalPublicKey,
      participant.publicKey
    );

    await mintTo(
      provider.connection,
      deployer.payer,
      mintNormalPublicKey,
      proposalOwnerTokenAccount.address, // destination
      deployer.publicKey, // authority
      web3.LAMPORTS_PER_SOL * 100
    );
    await mintTo(
      provider.connection,
      deployer.payer,
      mintNormalPublicKey,
      participantTokenAccount.address, // destination
      deployer.publicKey, // authority
      web3.LAMPORTS_PER_SOL * 100
    );

    // Refresh
    proposalOwnerTokenAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      proposalOwner,
      mintNormalPublicKey,
      proposalOwner.publicKey
    );

    participantTokenAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      participant,
      mintNormalPublicKey,
      participant.publicKey
    );

    [swapTokenVault, swapTokenVaultBump] = await PublicKey.findProgramAddress([
      anchor.utils.bytes.utf8.encode("SEED::SWAP::TOKEN_VAULT_SEED"),
      mintNormalPublicKey.toBytes()
    ], program.programId);

    // Now to create the proposal
    proposalId = Keypair.generate().publicKey.toBase58().slice(0, 10);
    [swapProposal] = await PublicKey.findProgramAddress([
      anchor.utils.bytes.utf8.encode("SEED::SWAP::PROPOSAL_SEED"),
      anchor.utils.bytes.utf8.encode(proposalId)
    ], program.programId);

    // now we whitelist the token first
    await program.methods.createTokenVault().accounts({
      mintAccount: mintNormalPublicKey,
      swapRegistry,
      swapTokenVault,
      owner: deployer.publicKey
    }).signers([deployer.payer]).rpc({ commitment: "confirmed" });

    // Construct data to be sent over the RPC.
    expiredAt = new BN(new Date().getTime() + 1000 * 60 * 60 * 24 * 7);
    offeredItems = [{
      id: Keypair.generate().publicKey.toBase58().slice(0, 10),
      mintAccount: mintNormalPublicKey,
      amount: new BN(web3.LAMPORTS_PER_SOL)
    }, {
      id: Keypair.generate().publicKey.toBase58().slice(0, 10),
      mintAccount: mintNormalPublicKey,
      amount: new BN(web3.LAMPORTS_PER_SOL)
    }];
    swapOptions = [{
      id: Keypair.generate().publicKey.toBase58().slice(0, 10),
      askingItems: [{
        id: Keypair.generate().publicKey.toBase58().slice(0, 10),
        mintAccount: mintNormalPublicKey,
        amount: new BN(web3.LAMPORTS_PER_SOL * 4)
      }]
    }, {
      id: Keypair.generate().publicKey.toBase58().slice(0, 10),
      askingItems: [{
        id: Keypair.generate().publicKey.toBase58().slice(0, 10),
        mintAccount: mintNormalPublicKey,
        amount: new BN(web3.LAMPORTS_PER_SOL * 4)
      },{
        id: Keypair.generate().publicKey.toBase58().slice(0, 10),
        mintAccount: mintNormalPublicKey,
        amount: new BN(web3.LAMPORTS_PER_SOL * 4)
      }]
    }];
  });

  it("[deposit_assets] should: proposal owner deposits offered items successfully", async () => {
    expect(Number(proposalOwnerTokenAccount.amount)).equals(web3.LAMPORTS_PER_SOL * 100);

    const depositInstructions = await Promise.all(
      offeredItems.map((item) => {
        const params = {
          proposalId,
          swapItemId: item.id,
          swapTokenVaultBump,
          actionType: { depositing: {} },
          optionId: ""
        };
        // @ts-ignore
        return program.methods.transferAssetsToVault(params).accounts({
          signer: proposalOwner.publicKey,
          signerTokenAccount: proposalOwnerTokenAccount.address,
          swapProposal,
          swapRegistry,
          swapTokenVault,
          mintAccount: mintNormalPublicKey
        }).signers([proposalOwner]).instruction();
      }));

    // now we deposit two times in a row
    await program.methods.createProposal({
      id: proposalId,
      swapOptions,
      offeredItems,
      expiredAt
    })
      .accounts({
        proposalOwner: proposalOwner.publicKey,
        swapRegistry,
        swapProposal: swapProposal
      })
      .signers([proposalOwner])
      .postInstructions(depositInstructions)
      .rpc({ commitment: "confirmed" });

    // now verify the state
    const state = await program.account.swapProposal.fetch(swapProposal);

    // @ts-ignore
    expect(!!state.status.deposited).to.be.true;
    // @ts-ignore
    expect(!!state.offeredItems.find(item => !item.status.deposited)).to.be.false;

    // the proposal owner balance must be debited
    proposalOwnerTokenAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      proposalOwner,
      mintNormalPublicKey,
      proposalOwner.publicKey
    );
    expect(Number(proposalOwnerTokenAccount.amount)).eq(web3.LAMPORTS_PER_SOL * 98);

    // the proposal balance must be credited
    const proposalTokenVaultAccount = await getAccount(
      provider.connection,
      swapTokenVault
    );
    expect(Number(proposalTokenVaultAccount.amount)).eq(web3.LAMPORTS_PER_SOL * 2);
  });

  it("[fulfil_assets] should: participants can cancel proposal anytime when proposal isn't fulfilled", async () => {
    expect(Number(participantTokenAccount.amount)).equals(web3.LAMPORTS_PER_SOL * 100);
    const swapOption = swapOptions[1];

    const fulfillingInstructions = await Promise.all(
      swapOption.askingItems.map((item) => {
        const params = {
          proposalId,
          swapItemId: item.id,
          swapTokenVaultBump,
          actionType: { fulfilling: {} },
          optionId: swapOption.id
        };
        // @ts-ignore
        return program.methods.transferAssetsToVault(params).accounts({
          signer: participant.publicKey,
          signerTokenAccount: participantTokenAccount.address,
          swapProposal,
          swapRegistry,
          swapTokenVault,
          mintAccount: mintNormalPublicKey
        }).signers([participant]).instruction();
      }));

    const transaction = new web3.Transaction();
    transaction.add(...fulfillingInstructions);

    // send transaction
    await provider.sendAndConfirm(transaction, [participant]);

    const state = await program.account.swapProposal.fetch(swapProposal);

    // @ts-ignore
    expect(!!state.status.fulfilled).to.be.true;
    expect(state.fulfilledBy.toBase58()).to.equals(participant.publicKey.toBase58());
    expect(state.fulfilledWithOptionId).to.equals(swapOption.id);

    const submittedSwapOption = state.swapOptions[1];
    // @ts-ignore
    expect(!!submittedSwapOption.askingItems.find(item => !item.status.deposited)).to.be.false;

    // the proposal owner balance must be debited
    participantTokenAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      participant,
      mintNormalPublicKey,
      participant.publicKey
    );
    expect(Number(participantTokenAccount.amount)).eq(web3.LAMPORTS_PER_SOL * 92);

    // the proposal balance must be credited
    const proposalTokenVaultAccount = await getAccount(
      provider.connection,
      swapTokenVault
    );
    expect(Number(proposalTokenVaultAccount.amount)).eq(web3.LAMPORTS_PER_SOL * 10);
  });
});
