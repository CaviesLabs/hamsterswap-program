import * as anchor from "@project-serum/anchor";
import { web3 } from "@project-serum/anchor";
import { Keypair, AddressLookupTableProgram, TransactionInstruction } from "@solana/web3.js";
import {
  createMint,
  createMintToInstruction,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import _ from "lodash";
import { expect } from "chai";
import { V0transactionProvider } from "../client/v0transaction.provider";

describe("transactionv0_and_lookuptable", async () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const deployer = Keypair.generate();
  const otherUser = Keypair.generate();

  const airdropSignature = await provider.connection.requestAirdrop(
    deployer.publicKey,
    web3.LAMPORTS_PER_SOL
  );
  await provider.connection.confirmTransaction({
    signature: airdropSignature,
    blockhash: (await provider.connection.getLatestBlockhash()).blockhash,
    lastValidBlockHeight: (
      await provider.connection.getLatestBlockhash()
    ).lastValidBlockHeight,
  });

  // funding proposal owner
  const airdropForParticipantSignature =
    await provider.connection.requestAirdrop(
      otherUser.publicKey,
      web3.LAMPORTS_PER_SOL
    );
  await provider.connection.confirmTransaction({
    signature: airdropForParticipantSignature,
    blockhash: (await provider.connection.getLatestBlockhash()).blockhash,
    lastValidBlockHeight: (
      await provider.connection.getLatestBlockhash()
    ).lastValidBlockHeight,
  });

  it("[lookup_table] should: should batch mint using lookup table successfully", async () => {
    /**
     * @dev batch mint to 100 accounts at a time
     */
    const mintableToken = await createMint(
      provider.connection, // connection
      deployer, // fee payer
      deployer.publicKey, // mint authority
      deployer.publicKey, // freeze authority (you can use `null` to disable it. when you disable it, you can't turn it on again)
      9 // decimals
    );

    const recipients1 = new Array(10).fill(0).map(() => Keypair.generate());
    const recipients2 = new Array(10).fill(0).map(() => Keypair.generate());
    const recipients3 = new Array(7).fill(0).map(() => Keypair.generate());
    const recipients4 = new Array(1).fill(0).map(() => Keypair.generate());
    const recipients5 = new Array(1).fill(0).map(() => Keypair.generate());
    const additional = [{ address: mintableToken }];

    /**
     * @dev Create associated token account
     */
    const recipients = await Promise.all(
      [
        ...recipients1,
        ...recipients2,
        ...recipients3,
        ...recipients4,
        ...recipients5,
      ].map(async (elm) => {
        return getOrCreateAssociatedTokenAccount(
          provider.connection,
          deployer,
          mintableToken,
          elm.publicKey
        );
      })
    );

    // Note: max can be set as 64 accounts
    const v0TransactionProvider = new V0transactionProvider();

    /**
     * @dev Create lookup table first
     */
    const [lookupTableInst, lookupTableAddress] =
      AddressLookupTableProgram.createLookupTable({
        authority: deployer.publicKey,
        payer: deployer.publicKey,
        recentSlot: await provider.connection.getSlot({
          commitment: "finalized",
        }),
      });

    const atlInstructions: TransactionInstruction[] = [];
    atlInstructions.push(lookupTableInst);

    /**
     * @dev Extend chunks of addresses
     */
    const recipientChunks = _.chunk([...recipients, ...additional], 30);

    await Promise.all(
      recipientChunks.map(async (chunk) => {
        const extendInstruction =
          web3.AddressLookupTableProgram.extendLookupTable({
            payer: deployer.publicKey,
            authority: deployer.publicKey,
            lookupTable: lookupTableAddress,
            addresses: chunk.map((elm) => elm.address),
          });
        atlInstructions.push(extendInstruction);
      })
    );

    await v0TransactionProvider
      .sendAndConfirmV0Transaction(provider, atlInstructions, deployer)
      .catch((e) => console.log("extend error", e));

    /**
     * @dev Fetch lookup table account
     */
    const lookupTableAccount = await provider.connection
      .getAddressLookupTable(lookupTableAddress, { commitment: "finalized" })
      .then((res) => res.value);

    /**
     * @dev Prepare recipients and send batch mint
     */
    const instructions = await Promise.all(
      recipients.map((elm) =>
        createMintToInstruction(
          mintableToken,
          elm.address, // destination
          deployer.publicKey, // authority
          web3.LAMPORTS_PER_SOL * 100
        )
      )
    );

    await v0TransactionProvider
      .sendAndConfirmV0TransactionWithALT(
        provider,
        instructions,
        [lookupTableAccount],
        deployer
      )
      .catch((e) => console.log("batch mint error", e));

    /**
     * @dev Expect balance
     */
    await Promise.all(
      [
        ...recipients1,
        ...recipients2,
        ...recipients3,
        ...recipients4,
        ...recipients5,
      ].map(async (elm) => {
        const account = await getOrCreateAssociatedTokenAccount(
          provider.connection,
          deployer,
          mintableToken,
          elm.publicKey
        );

        expect(Number(account.amount)).to.eq(web3.LAMPORTS_PER_SOL * 100);
      })
    );
  });
});
