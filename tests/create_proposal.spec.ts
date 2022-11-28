import * as anchor from "@project-serum/anchor";
import { BN, BorshCoder, EventParser, Program, web3 } from "@project-serum/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import { createMint } from "@solana/spl-token";

import { Swap } from "../target/types/swap";

describe("create_proposal", async () => {
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
  });

  it("[create_proposal] should: fail to create proposal with un-allowed mint tokens", async () => {
    // Now to create the proposal
    const id = Keypair.generate().publicKey.toBase58().slice(0, 10);
    const [swapProposal] = await PublicKey.findProgramAddress([
      anchor.utils.bytes.utf8.encode("SEED::SWAP::PROPOSAL_SEED"),
      anchor.utils.bytes.utf8.encode(id)
    ], program.programId);

    // Construct data to be sent over the RPC.
    const offeredItems = [{
      mintAccount: mintNormalPublicKey,
      amount: new BN(1)
    }];
    const swapOptions = [{
      id: Keypair.generate().publicKey.toBase58().slice(0, 10),
      asking_items: [{
        mintAccount: mintNormalPublicKey,
        amount: new BN(4)
      }]
    }];

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
    const expiredAt= new BN(new Date().getTime() + 1000 * 60 * 60 * 24 * 7);
    const offeredItems = [{
      mintAccount: mintNormalPublicKey,
      amount: new BN(1)
    }];
    const swapOptions = [{
      id: Keypair.generate().publicKey.toBase58().slice(0, 10),
      askingItems: [{
        mintAccount: mintNormalPublicKey,
        amount: new BN(4)
      }]
    }];

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
    expect(state.offeredItems.length).eq(1);
    expect(state.offeredItems[0].mintAccount.equals(mintNormalPublicKey)).to.be.true;
    expect(state.offeredItems[0].amount.eq(new BN(1))).to.be.true;
    expect(!!state.offeredItems[0].status.created).to.be.true;

     // @ts-ignore
    expect(state.swapOptions.length).eq(1);
    expect(state.swapOptions[0].id).eq(swapOptions[0].id);
    expect(state.swapOptions[0].askingItems.length).eq(1);
    expect(state.swapOptions[0].askingItems[0].mintAccount.equals(mintNormalPublicKey)).to.be.true;
    expect(state.swapOptions[0].askingItems[0].amount.eq(new BN(4))).to.be.true;
    expect(!!state.swapOptions[0].askingItems[0].status.created).to.be.true;

    // expect log
    const transaction = await provider.connection.getParsedTransaction(tx, { commitment: "confirmed" });
    const eventParser = new EventParser(program.programId, new BorshCoder(program.idl));
    const [event] = eventParser.parseLogs(transaction.meta.logMessages);

    // Expect emitted logs
    expect(event.data.owner.toString() === proposalOwner.publicKey.toString()).equals(true);
    expect(event.data.proposalKey.toString() === swapProposal.toString()).equals(true);
    expect(event.data.id).equals(proposalId);
    // @ts-ignore
    expect(event.data.expiredAt.eq(new BN(expiredAt))).to.be.true;
  });

  it('[cancel_proposal] should: participants can cancel proposal anytime when proposal isn\'t fulfilled', async () => {
      const response = await program.methods.cancelProposal({ id: proposalId }).accounts({
        swapProposal,
        signer: proposalOwner.publicKey
      }).signers([proposalOwner]).rpc({commitment: 'confirmed'});
      expect(!!response).to.be.true;
    const state = await program.account.swapProposal.fetch(swapProposal);
    expect(state.id).eq(proposalId);
    // @ts-ignore
    expect(!!state.status.canceled).to.be.true;
  })
});
