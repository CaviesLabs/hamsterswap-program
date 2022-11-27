use crate::*;

// Define params
#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Debug, PartialEq)]
pub struct UpdateSwapPlatformParams {
    // define max item can be traded.
    pub max_allowed_items: u8,

    // define max allowed options can be asked.
    pub max_allowed_options: u8,
}

// Define the context, passed in parameters when trigger from deployer.
#[derive(Accounts)]
pub struct UpdateSwapPlatformContext<'info> {
    // We define the fee payer
    #[account(
        mut,
        address = swap_registry.owner @ SwapError::OnlyAdministrator
    )]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [PLATFORM_SEED],
        bump = swap_registry.bump,
        has_one = owner
    )]
    pub swap_registry: Account<'info, SwapPlatformRegistry>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

// implement the handler
impl<'info> UpdateSwapPlatformContext<'info> {
    pub fn execute(&mut self, params: UpdateSwapPlatformParams) -> Result<()> {
        if params.max_allowed_options < 1 {
            return Err(SwapError::InvalidValue.into());
        }

        if params.max_allowed_items < 1 {
            return Err(SwapError::InvalidValue.into());
        }

        // Assigning values
        let swap_registry = &mut self.swap_registry;
        swap_registry.max_allowed_options = params.max_allowed_options.clone();
        swap_registry.max_allowed_items = params.max_allowed_items.clone();

        // emit event
        swap_emit!(
            SwapConfigUpdated {
                owner: self.owner.key().clone(),
                max_allowed_options: params.max_allowed_options.clone(),
                max_allowed_items: params.max_allowed_items.clone(),
            }
        );

        Ok(())
    }
}
