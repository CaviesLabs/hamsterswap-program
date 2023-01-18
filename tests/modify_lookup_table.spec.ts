import * as anchor from "@project-serum/anchor";
import {
  AnchorError,
  BN,
  BorshCoder,
  EventParser,
  Program,
} from "@project-serum/anchor";
import { PublicKey, Keypair, SendTransactionError, AddressLookupTableProgram } from "@solana/web3.js";
import { expect } from "chai";

import { Swap } from "../target/types/swap";

describe("modify_lookup_table", async () => {
    // Configure the client to use the local cluster.
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.Swap as Program<Swap>;
    const deployer = provider.wallet as anchor.Wallet;
    const otherUser = Keypair.generate();

    // find the swap account
    const [swapAccount] = await PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode("SEED::SWAP::PLATFORM")],
      program.programId
    );

  it("[modify_lookup_table] should: create lookup table successfully", async () => {
    const slot = await provider.connection.getSlot({
      commitment: "finalized",
    });

    const [,lookupTableAddress] =
      AddressLookupTableProgram.createLookupTable({
        authority: swapAccount,
        payer: deployer.publicKey,
        recentSlot: await provider.connection.getSlot({
          commitment: "finalized",
        }),
      });

    console.log({
      signer: deployer.publicKey.toBase58(),
      swapRegistry: swapAccount.toBase58(),
      lookupTableAddress: lookupTableAddress.toBase58()
    });

    // Initialize first
    const inst = await program.methods
      .modifyAddressLookupTable({
        slot: new BN(slot),
        whitelistedAddresses: [],
        actionType: {createLookupTable: {}},
      })
      .accounts({
        swapRegistry: swapAccount,
        lookupTableAccount: lookupTableAddress,
        signer: deployer.publicKey,
        lookupTableProgram: AddressLookupTableProgram.programId
      })
      .signers([deployer.payer])
      .rpc({ commitment: "confirmed" }).catch(e => console.log(e));

    const state = await program.account.swapPlatformRegistry.fetch(swapAccount);
    console.log(state);
  });

});
