use std::borrow::Borrow;
use anchor_lang::solana_program;
use crate::*;

// Define params
#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Debug, PartialEq)]
pub struct CreateAddressLookupTableParams {
    // derivation path
    pub slot: u64,
}

// Define the context, passed in parameters when trigger from deployer.
#[derive(Accounts)]
pub struct CreateAddressLookupTableContext<'info> {
    // We define the fee payer
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [LOOKUP_TABLE_SEED, signer.key().as_ref()],
        bump = lookup_table_registry.bump,
    )]
    pub lookup_table_registry: Account<'info, LookupTableRegistry>,

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
impl<'info> CreateAddressLookupTableContext<'info> {
    pub fn execute(&mut self, params: CreateAddressLookupTableParams) -> Result<()> {
        let slot = params.slot;
        self.create_lookup_table(slot).unwrap();

        return Ok(());
    }

    fn create_lookup_table(&mut self, slot: u64) -> Result<()> {
        let lookup_table_registry = self.lookup_table_registry.borrow_mut();

        let (create_ix, table_pk) =
            instruction::create_lookup_table(
                self.signer.key(),
                self.signer.key(),
                slot,
            );

        solana_program::program::invoke(
            &create_ix,
            &[
                self.lookup_table_account.to_account_info(),
                self.signer.to_account_info(),
                self.signer.to_account_info(),
            ]
        ).unwrap();

        lookup_table_registry.lookup_table_addresses.push(table_pk);

        return Ok(());
    }
}
