use crate::*;

// Define params
#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Debug, PartialEq)]
pub struct UpdateSwapPlatformParams {
    // define max item can be traded.
    pub max_allowed_items: u8,

    // define max allowed options can be asked.
    pub max_allowed_options: u8,

    // define whitelisted mint token account
    pub allowed_mint_accounts: Vec<Pubkey>,
}

// Define the context, passed in parameters when trigger from deployer.
#[derive(Accounts)]
pub struct UpdateSwapPlatformContext<'info> {
    // We define the fee payer
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [PLATFORM_SEED, owner.key().as_ref()],
        bump = swap_config.bump,
        has_one = owner
    )]
    pub swap_config: Account<'info, SwapPlatformConfig>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

// implement the handler
impl<'info> UpdateSwapPlatformContext<'info> {
    pub fn execute(&mut self, params: UpdateSwapPlatformParams) -> Result<()> {
        // throw errors first
        if params.allowed_mint_accounts.len() < 1 {
            return Err(SwapError::InvalidValue.into());
        }

        if params.max_allowed_options < 1 {
            return Err(SwapError::InvalidValue.into());
        }

        if params.max_allowed_items < 1 {
            return Err(SwapError::InvalidValue.into());
        }

        // Assigning values
        let swap_config = &mut self.swap_config;
        swap_config.allowed_mint_accounts = params.allowed_mint_accounts.clone();
        swap_config.max_allowed_items = params.max_allowed_items.clone();
        swap_config.max_allowed_options = params.max_allowed_options.clone();

        // emit event
        swap_emit!(
            SwapConfigUpdated {
                owner: self.owner.key().clone(),
                max_allowed_options: params.max_allowed_options.clone(),
                max_allowed_items: params.max_allowed_items.clone(),
                allowed_mint_accounts: params.allowed_mint_accounts.clone(),
            }
        );

        Ok(())
    }
}
