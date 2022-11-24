use crate::*;

// Define params
#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Debug, PartialEq)]
pub struct InitializeSwapPlatformParams {
    // define max item can be traded.
    pub max_allowed_items: u8,

    // define max allowed options can be asked.
    pub max_allowed_options: u8,

    // define whitelisted mint token account
    pub allowed_mint_account: HashMap<Pubkey, bool>,
}

// Define the context, passed in parameters when trigger from deployer.
#[derive(Accounts)]
pub struct InitializeSwapPlatformContext<'info> {
    // We define the fee payer
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        seeds = [PLATFORM_SEED, owner.key().as_ref()],
        payer = owner,
        space = 10240,
        bump
    )]
    pub swap_config: Account<'info, SwapPlatformConfig>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

// implement the handler
impl<'info> InitializeSwapPlatformContext<'info> {
    pub fn execute(&mut self, params: InitializeSwapPlatformParams, bump: u8) -> Result<()> {
        // Assigning values
        let swap_config = &mut self.swap_config;
        swap_config.bump = bump;
        swap_config.owner = *self.owner.key;
        swap_config.allowed_mint_account = params.allowed_mint_account;
        swap_config.max_allowed_items = params.max_allowed_items;
        swap_config.max_allowed_options = params.max_allowed_options;

        // Handle post initialization
        self.swap_config.handle_post_initialized().unwrap();

        Ok(())
    }
}
