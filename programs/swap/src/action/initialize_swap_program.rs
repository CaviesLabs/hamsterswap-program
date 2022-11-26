use crate::*;

// Define params
#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Debug, PartialEq)]
pub struct InitializeSwapPlatformParams {
    // define max item can be traded.
    pub max_allowed_items: u8,

    // define max allowed options can be asked.
    pub max_allowed_options: u8,
}

// Define the context, passed in parameters when trigger from deployer.
#[derive(Accounts)]
pub struct InitializeSwapPlatformContext<'info> {
    // We define the fee payer
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        seeds = [PLATFORM_SEED],
        payer = owner,
        space = 10240,
        bump
    )]
    pub swap_registry: Account<'info, SwapPlatformRegistry>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

// implement the handler
impl<'info> InitializeSwapPlatformContext<'info> {
    pub fn execute(&mut self, params: InitializeSwapPlatformParams, bump: u8) -> Result<()> {
        // Handle post initialization
        self.swap_registry.handle_post_initialized().unwrap();

        // Assigning values
        let swap_registry = &mut self.swap_registry;
        swap_registry.bump = bump;
        swap_registry.owner = *self.owner.key;
        swap_registry.max_allowed_items = params.max_allowed_items;
        swap_registry.max_allowed_options = params.max_allowed_options;

        Ok(())
    }
}
