import * as anchor from "@project-serum/anchor";
import {
  BN,
  Program
} from "@project-serum/anchor";
import { PublicKey, AddressLookupTableProgram } from "@solana/web3.js";

import { Swap } from "../target/types/swap";
import { V0transactionProvider } from "../client/v0transaction.provider";
import { expect } from "chai";

describe("modify_lookup_table", async () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Swap as Program<Swap>;
  const deployer = provider.wallet as anchor.Wallet;

  const slot = await provider.connection.getSlot({
    commitment: "finalized"
  });

  const [lookupTableRegistry] = await PublicKey.findProgramAddress(
    [
      anchor.utils.bytes.utf8.encode("SEED::SWAP::LOOKUP_TABLE_SEED"),
      deployer.publicKey.toBytes(),
    ],
    program.programId
  );

  const [,lookupTableAddress] =
    AddressLookupTableProgram.createLookupTable({
      authority: deployer.publicKey,
      payer: deployer.publicKey,
      recentSlot: slot
    });

  it("[modify_lookup_table] should: create lookup table successfully", async () => {
    const createLookupTableRegistryInx = await program.methods
      .initializeAddressLookupTable()
      .accounts({
        lookupTableRegistry: lookupTableRegistry,
        signer: deployer.publicKey
      }).instruction();

    // Initialize first
    const createLookupTableInx = await program.methods
      // @ts-ignore
      .modifyAddressLookupTable({
        slot: new BN(slot),
      })
      .accounts({
        lookupTableRegistry,
        lookupTableAccount: lookupTableAddress,
        signer: deployer.publicKey,
        lookupTableProgram: AddressLookupTableProgram.programId
      }).instruction();

    const extendLookupTableInx = AddressLookupTableProgram.extendLookupTable({
      lookupTable: lookupTableAddress,
      authority: deployer.publicKey,
      payer: deployer.publicKey,
      addresses: [deployer.publicKey],
    });

    const v0TransactionProvider = new V0transactionProvider();
    await v0TransactionProvider.sendAndConfirmV0Transaction(
      provider,
      [createLookupTableRegistryInx, createLookupTableInx, extendLookupTableInx],
      deployer.payer
    ).then(res => res).catch(e => console.log(e));

    const state = await program.account.lookupTableRegistry.fetch(lookupTableRegistry);
    const lookupTableAccount = await provider.connection.getAddressLookupTable(lookupTableAddress).then(res => res.value);

    expect(state.lookupTableAddresses.filter(elm => elm.equals(lookupTableAddress)).length > 0).to.be.true;
    expect(lookupTableAccount.state.addresses.filter(elm => elm.equals(deployer.publicKey)).length > 0).to.be.true;
  });
});
