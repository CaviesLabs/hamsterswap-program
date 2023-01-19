use crate::*;

// Define the context, passed in parameters when trigger from deployer.
#[derive(Accounts)]
pub struct InitializeAddressLookupTableContext<'info> {
    // We define the fee payer
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        seeds = [LOOKUP_TABLE_SEED, signer.key().as_ref()],
        payer = signer,
        space = 10240,
        bump
    )]
    pub lookup_table_registry: Account<'info, LookupTableRegistry>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

// implement the handler
impl<'info> InitializeAddressLookupTableContext<'info> {
    pub fn execute(&mut self, bump: u8) -> Result<()> {
        self.lookup_table_registry.bump = bump;
        self.lookup_table_registry.owner = self.signer.key();

        return Ok(())
    }
}
