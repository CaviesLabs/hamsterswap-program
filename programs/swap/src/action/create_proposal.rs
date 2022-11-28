use crate::*;

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Debug, PartialEq)]
pub struct SwapItemInfo {
    pub id: String,
    pub mint_account: Pubkey,
    pub amount: u64
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Debug, PartialEq)]
pub struct SwapItemOptionInfo {
    id: String,
    asking_items: Vec<SwapItemInfo>,
}

// Define params
#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Debug, PartialEq)]
pub struct CreateProposalParams {
    // offchain id used as a ref
    pub id: String,

    // define swap options that has been included
    pub swap_options: Vec<SwapItemOptionInfo>,

    // define offered items that has been included
    pub offered_items: Vec<SwapItemInfo>,

    // define expiry date
    pub expired_at: u64,
}

// Define the context, passed in parameters when trigger from deployer.
#[derive(Accounts)]
#[instruction(params: CreateProposalParams)]
pub struct CreateProposalContext<'info> {
    // We define the fee payer
    #[account(mut)]
    pub proposal_owner: Signer<'info>,

    #[account(
        init,
        seeds = [PROPOSAL_SEED, params.id.as_bytes().as_ref()],
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
}

impl<'info> CreateProposalContext<'info> {
    pub fn execute(&mut self, params: CreateProposalParams, bump: u8) -> Result<()> {
        // set data
        let swap_proposal = &mut self.swap_proposal;
        swap_proposal.owner = *self.proposal_owner.key;

        // Compute asking items
        swap_proposal.swap_options = params.swap_options.into_iter().map(|option| {
            let mut swap_option = SwapOption::default();

            swap_option.id = option.id;
            swap_option.asking_items =  option.asking_items.into_iter()
                .map(|item| {
                    let mut swap_item = SwapItem::default();
                    swap_item.amount = item.amount;
                    swap_item.mint_account = item.mint_account;
                    swap_item.id = item.id;

                    return swap_item;
                }).collect();

            return swap_option;
        }).collect();

        // Compute offered items
        swap_proposal.offered_items = params.offered_items.into_iter().map(|item| {
            let mut swap_item = SwapItem::default();
            swap_item.amount = item.amount;
            swap_item.mint_account = item.mint_account;

            return swap_item;
        }).collect();


        swap_proposal.expired_at = params.expired_at;
        swap_proposal.id = params.id;
        swap_proposal.bump = bump;

        // Now to validate data state
        self.handle_post_initialized().unwrap();

        swap_emit!(
          ProposalCreated {
                id: self.swap_proposal.id.to_string(),
                proposal_key: self.swap_proposal.key().clone(),
                expired_at: self.swap_proposal.expired_at as i64,
                owner: self.swap_proposal.owner.clone()
            }
        );

        // ok
        Ok(())
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
            if !self.swap_registry.is_mint_account_enabled(item.mint_account) {
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

        if self.swap_proposal.expired_at <= Clock::get().unwrap().unix_timestamp as u64 {
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