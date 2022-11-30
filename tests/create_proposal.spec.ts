import * as anchor from "@project-serum/anchor";
import { BN, BorshCoder, EventParser, Program, web3 } from "@project-serum/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import { createMint, getAccount, getOrCreateAssociatedTokenAccount, mintTo } from "@solana/spl-token";

import { Swap } from "../target/types/swap";

describe("create_proposal", async () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Swap as Program<Swap>;
  const deployer = provider.wallet as anchor.Wallet;

  // find the swap account
  const [swapRegistry, swapRegistryBump] = await PublicKey.findProgramAddress([
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
  let swapTokenVaultBump;
  let proposalOwnerTokenAccount;

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
      mintNormalPublicKey.toBytes()
    ], program.programId);

    // Construct accounts for proposal creation
    proposalOwner = Keypair.generate();
    participant = Keypair.generate();

    // funding proposal owner
    const airdropSignature = await provider.connection.requestAirdrop(proposalOwner.publicKey, web3.LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction({
      signature: airdropSignature,
      blockhash: (await provider.connection.getLatestBlockhash()).blockhash,
      lastValidBlockHeight: ((await provider.connection.getLatestBlockhash())).lastValidBlockHeight
    });

    // Now to create the proposal
    proposalId = Keypair.generate().publicKey.toBase58().slice(0, 10);
    [swapProposal] = await PublicKey.findProgramAddress([
      anchor.utils.bytes.utf8.encode("SEED::SWAP::PROPOSAL_SEED"),
      anchor.utils.bytes.utf8.encode(proposalId)
    ], program.programId);

    proposalOwnerTokenAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      proposalOwner,
      mintNormalPublicKey,
      proposalOwner.publicKey
    );

    await mintTo(
      provider.connection,
      deployer.payer,
      mintNormalPublicKey,
      proposalOwnerTokenAccount.address, // destination
      deployer.publicKey, // authority
      web3.LAMPORTS_PER_SOL * 100
    );

    [swapTokenVault, swapTokenVaultBump] = await PublicKey.findProgramAddress([
      anchor.utils.bytes.utf8.encode("SEED::SWAP::TOKEN_VAULT_SEED"),
      mintNormalPublicKey.toBytes()
    ], program.programId);

    offeredItems = [{
      id: Keypair.generate().publicKey.toBase58().slice(0, 10),
      mintAccount: mintNormalPublicKey,
      amount: new BN(web3.LAMPORTS_PER_SOL)
    },
      {
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
      }, {
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
      }]
    }];
  });

  it("[create_proposal] should: fail to create proposal with un-allowed mint tokens", async () => {
    // Now to create the proposal
    const id = Keypair.generate().publicKey.toBase58().slice(0, 10);
    const [swapProposal] = await PublicKey.findProgramAddress([
      anchor.utils.bytes.utf8.encode("SEED::SWAP::PROPOSAL_SEED"),
      anchor.utils.bytes.utf8.encode(id)
    ], program.programId);

    try {
      // Send RPC request to blockchain.
      await program.methods.createProposal({
        id,
        swapOptions,
        offeredItems,
        expiredAt: new BN(new Date().getTime() + 1000 * 60 * 60 * 24 * 7)
      }).accounts({
        proposalOwner: proposalOwner.publicKey,
        swapRegistry,
        swapProposal: swapProposal
      }).signers([proposalOwner]).rpc({ commitment: "confirmed" });

      throw new Error("Should failed");
    } catch (e) {
      expect(!!e).to.be.true;
    }
  });

  it("[create_proposal] should: everyone can create publicly a proposal", async () => {
    // now we whitelist the token first
    await program.methods.createTokenVault().accounts({
      mintAccount: mintNormalPublicKey,
      swapRegistry,
      swapTokenVault,
      owner: deployer.publicKey
    }).signers([deployer.payer]).rpc({ commitment: "confirmed" });

    // Construct data to be sent over the RPC.
    const expiredAt = new BN(new Date().getTime() + 1000 * 60 * 60 * 24 * 7);


    // Send RPC request to blockchain.
    const tx = await program.methods.createProposal({
      id: proposalId,
      swapOptions,
      offeredItems,
      expiredAt
    }).accounts({
      proposalOwner: proposalOwner.publicKey,
      swapRegistry,
      swapProposal: swapProposal
    }).signers([proposalOwner]).rpc({ commitment: "confirmed" });

    // now we verify the state
    const state = await program.account.swapProposal.fetch(swapProposal);

    expect(state.id).eq(proposalId);
    expect(state.expiredAt.eq(new BN(expiredAt))).to.be.true;
    expect(!!state.bump).to.be.true;
    expect(state.owner.equals(proposalOwner.publicKey)).to.be.true;
    // expect offered items
    // @ts-ignore
    expect(state.offeredItems.length).eq(2);
    expect(state.offeredItems[0].id === offeredItems[0].id).to.be.true;
    expect(state.offeredItems[0].mintAccount.equals(mintNormalPublicKey)).to.be.true;
    expect(state.offeredItems[0].amount.eq(new BN(web3.LAMPORTS_PER_SOL))).to.be.true;
    expect(!!state.offeredItems[0].status.created).to.be.true;

    // @ts-ignore
    expect(state.swapOptions.length).eq(2);
    expect(state.swapOptions[0].id).eq(swapOptions[0].id);
    expect(state.swapOptions[0].askingItems.length).eq(2);

    expect(state.swapOptions[0].askingItems[0].mintAccount.equals(mintNormalPublicKey)).to.be.true;
    expect(state.swapOptions[0].askingItems[0].amount.eq(new BN(web3.LAMPORTS_PER_SOL * 4))).to.be.true;
    expect(!!state.swapOptions[0].askingItems[0].status.created).to.be.true;

    expect(state.swapOptions[0].askingItems[1].mintAccount.equals(mintNormalPublicKey)).to.be.true;
    expect(state.swapOptions[0].askingItems[1].amount.eq(new BN(web3.LAMPORTS_PER_SOL * 4))).to.be.true;
    expect(!!state.swapOptions[0].askingItems[1].status.created).to.be.true;

    expect(state.swapOptions[1].id).eq(swapOptions[1].id);
    expect(state.swapOptions[1].askingItems.length).eq(1);
    expect(state.swapOptions[1].askingItems[0].mintAccount.equals(mintNormalPublicKey)).to.be.true;
    expect(state.swapOptions[1].askingItems[0].amount.eq(new BN(web3.LAMPORTS_PER_SOL * 4))).to.be.true;
    expect(!!state.swapOptions[1].askingItems[0].status.created).to.be.true;

    // expect log
    const transaction = await provider.connection.getParsedTransaction(tx, { commitment: "confirmed" });
    const eventParser = new EventParser(program.programId, new BorshCoder(program.idl));
    const [event] = eventParser.parseLogs(transaction.meta.logMessages);

    // Expect emitted logs
    expect(event.data.actor.toString() === proposalOwner.publicKey.toString()).equals(true);
    expect(event.data.proposalKey.toString() === swapProposal.toString()).equals(true);
    expect(event.data.id).equals(proposalId);
    // @ts-ignore
    expect(event.data.expiredAt.eq(new BN(expiredAt))).to.be.true;
  });

  it("[cancel_proposal] should: participants can cancel proposal anytime when proposal isn't fulfilled", async () => {
    // try depositing some items
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
          swapTokenVault,
          mintAccount: mintNormalPublicKey
        }).signers([proposalOwner]).instruction();
      }));

    const transaction = new web3.Transaction();
    transaction.add(...depositInstructions);

    await provider.sendAndConfirm(transaction, [proposalOwner]);

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

    // now we cancel the program
    const response = await program.methods.cancelProposal({ id: proposalId }).accounts({
      swapProposal,
      signer: proposalOwner.publicKey
    }).signers([proposalOwner]).rpc({ commitment: "confirmed" });
    expect(!!response).to.be.true;
    const state = await program.account.swapProposal.fetch(swapProposal);
    expect(state.id).eq(proposalId);
    // @ts-ignore
    expect(!!state.status.canceled).to.be.true;
  });

  it("[withdraw_assets] should: participant can withdraw assets when proposal is canceled", async () => {
    const fulfillingInstructions = await Promise.all(
      offeredItems.map((item) => {
        const params = {
          proposalId,
          swapItemId: item.id,
          swapRegistryBump,
          swapTokenVaultBump,
          actionType: { withdrawing: {} }
        };

        // @ts-ignore
        return program.methods.transferAssetsFromVault(params).accounts({
          signer: proposalOwner.publicKey,
          signerTokenAccount: proposalOwnerTokenAccount.address,
          swapProposal,
          swapTokenVault,
          swapRegistry,
          mintAccount: mintNormalPublicKey
        }).signers([proposalOwner]).instruction();
      }));

    const transaction = new web3.Transaction();
    transaction.add(...fulfillingInstructions);

    try {
      await provider.sendAndConfirm(transaction, [proposalOwner]);

    } catch (e) {
      console.log(e);
    }

    const state = await program.account.swapProposal.fetch(swapProposal);

    // @ts-ignore
    expect(!!state.offeredItems.find(item => !item.status.withdrawn)).to.be.false;

    // the proposal owner balance must be debited
    proposalOwnerTokenAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      proposalOwner,
      mintNormalPublicKey,
      proposalOwner.publicKey
    );
    expect(Number(proposalOwnerTokenAccount.amount)).eq(web3.LAMPORTS_PER_SOL * 100);

    // the proposal balance must be credited
    const proposalTokenVaultAccount = await getAccount(
      provider.connection,
      swapTokenVault
    );
    expect(Number(proposalTokenVaultAccount.amount)).eq(web3.LAMPORTS_PER_SOL * 0);
  });
});
