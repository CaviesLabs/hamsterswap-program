use crate::*;

// Define params
#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Debug, PartialEq)]
pub struct CreateProposalParams {
    // offchain id used as a ref
    pub id: String,

    // define swap options that has been included
    pub swap_options: Vec<SwapOption>,

    // define offered items that has been included
    pub offered_items: Vec<SwapItem>,

    // define expiry date
    pub expired_at: u64,
}

// Define the context, passed in parameters when trigger from deployer.
#[derive(Accounts)]
#[instruction(params: CreateProposalParams, seed: String)]
pub struct CreateProposalContext<'info> {
    // We define the fee payer
    #[account(mut)]
    pub proposal_owner: Signer<'info>,

    #[account(
        init,
        seeds = [PROPOSAL_SEED, seed.as_bytes().as_ref()],
        payer = proposal_owner,
        space = 10240,
        bump
    )]
    pub swap_proposal: Account<'info, SwapProposal>,

    #[account(
        seeds = [PLATFORM_SEED],
        bump = swap_registry.bump,
    )]
    pub swap_registry: Account<'info, SwapPlatformRegistry>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = sysvar::slot_hashes::id())]
    /// CHECK: no need to check since this is system variable and non-mutable
    pub recent_slothashes: UncheckedAccount<'info>,
}

impl<'info> CreateProposalContext<'info> {
    pub fn execute(&mut self, params: CreateProposalParams, id: String, bump: u8) -> Result<()> {
        // set data
        let swap_proposal = &mut self.swap_proposal;
        swap_proposal.owner = *self.proposal_owner.key;
        swap_proposal.swap_options = params.swap_options;
        swap_proposal.offered_items = params.offered_items;
        swap_proposal.expired_at = params.expired_at;
        swap_proposal.id = id;
        swap_proposal.bump = bump;

        // Now to validate data state
        self.handle_post_initialized().unwrap();

        // now we need to transfer appropriate assets.
        self.transfer_asset_to_vault().unwrap();

        // ok
        Ok(())
    }

    // TODO: implement this
    fn transfer_asset_to_vault(&mut self) -> Result<()> {
        return Ok(());
    }

    // validate mint accounts
    fn validate_mint_accounts(&self, params: &Vec<SwapItem>) -> Result<()> {
        // Cannot exceed max allowed items
        if self.swap_registry.max_allowed_items < params.len() as u8 {
            return Err(SwapError::InvalidValue.into());
        }

        // Check if user submitted un-allowed mint tokens
        let iterator = params.iter();

        for item in iterator {
            if !self.swap_registry.allowed_mint_accounts.contains(&item.mint_account) {
                return Err(SwapError::UnAllowedMintToken.into());
            }
        }

        return Ok(());
    }

    fn validate_swap_options(&self, params: &Vec<SwapOption>) -> Result<()> {
        // validate input
        if self.swap_registry.max_allowed_options < params.len() as u8 {
            return Err(SwapError::InvalidValue.into());
        }

        let iterator = params.iter();

        // validate if the tokens were allowed
        for item in iterator {
            self.validate_mint_accounts(&item.asking_items).unwrap();
        }

        // ok
        return Ok(());
    }

    fn handle_post_initialized(&mut self) -> Result<()> {
        if self.swap_proposal.id == "".to_string() {
            return Err(SwapError::InvalidValue.into());
        }

        if self.swap_proposal.bump == 0 {
            return Err(SwapError::InvalidValue.into());
        }

        if self.swap_proposal.owner == Pubkey::default() {
            return Err(SwapError::InvalidValue.into());
        }

        if self.swap_proposal.offered_items.len() < 1 {
            return Err(SwapError::InvalidValue.into());
        }

        if self.swap_proposal.swap_options.len() < 1 {
            return Err(SwapError::InvalidValue.into());
        }

        if self.swap_proposal.expired_at < Clock::get().unwrap().unix_timestamp as u64 {
            return Err(SwapError::InvalidValue.into());
        }

        // Check if user want to offer un-allowed mint tokens
        self.validate_mint_accounts(&self.swap_proposal.offered_items).unwrap();

        // Check if user want to ask for un-allowed mint tokens
        self.validate_swap_options(&self.swap_proposal.swap_options).unwrap();

        // ok
        return Ok(());
    }
}