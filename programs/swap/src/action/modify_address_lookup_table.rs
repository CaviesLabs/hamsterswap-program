use std::borrow::Borrow;
use anchor_lang::solana_program;
use solana_address_lookup_table_program::state::{AddressLookupTable, LookupTableMeta};
use crate::*;


#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Debug, PartialEq)]
pub enum ModifyAddressLookupTableActionType {
    #[default]
    CreateLookupTable,
    ExtendLookupTable
}

// Define params
#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Debug, PartialEq)]
pub struct ModifyAddressLookupTableParams {
    // define max item can be traded.
    pub whitelisted_addresses: Vec<Pubkey>,

    // define the action type
    pub action_type: ModifyAddressLookupTableActionType,

    // derivation path
    pub slot: u64,
}

// Define the context, passed in parameters when trigger from deployer.
#[derive(Accounts)]
pub struct ModifyAddressLookupTableContext<'info> {
    // We define the fee payer
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [PLATFORM_SEED],
        bump = swap_registry.bump,
    )]
    pub swap_registry: Account<'info, SwapPlatformRegistry>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(mut)]
    /// CHECK:` doc comment explaining why no checks through types are necessary.
    pub lookup_table_account: AccountInfo<'info>,

    #[account(address = solana_address_lookup_table_program::ID)]
    /// CHECK:` doc comment explaining why no checks through types are necessary.
    pub lookup_table_program: AccountInfo<'info>,
}

// implement the handler
impl<'info> ModifyAddressLookupTableContext<'info> {
    pub fn execute(&mut self, params: ModifyAddressLookupTableParams) -> Result<()> {
        if params.action_type == ModifyAddressLookupTableActionType::CreateLookupTable {
            return Ok(self.create_lookup_table(params.slot).unwrap());
        }

        return Ok(())
    }

    fn create_lookup_table(&mut self, slot: u64) -> Result<()> {
        let swap_registry = self.swap_registry.borrow_mut();

        let (create_ix, table_pk) =
            instruction::create_lookup_table(
                swap_registry.key(),
                self.signer.key(),
                slot,
            );

        // find the bump to sign with the pda
        let bump = &[swap_registry.bump][..];
        let signer = token_account_signer!(
            PLATFORM_SEED,
            bump
        );

        solana_program::program::invoke_signed(
            &create_ix,
            &[
                self.lookup_table_account.to_account_info(),
                swap_registry.to_account_info(),
                self.signer.to_account_info(),
                self.system_program.to_account_info(),
                self.lookup_table_program.to_account_info(),
            ],
            signer
        ).unwrap();

        swap_registry.address_lookup_table.push(table_pk);

        return Ok(());
    }
}
