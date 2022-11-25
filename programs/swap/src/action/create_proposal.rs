use crate::*;

// Define params
#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Debug, PartialEq)]
pub struct CreateProposalParams {
    // define max item can be traded.
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
#[instruction(params: CreateProposalParams)]
pub struct CreateProposalContext<'info> {
    // We define the fee payer
    #[account(mut)]
    pub proposal_owner: Signer<'info>,

    #[account(
        init,
        seeds = [PROPOSAL_SEED, PROGRAM_ID, params.id.as_bytes().as_ref()],
        payer = proposal_owner,
        space = 10240,
        bump
    )]
    pub swap_proposal: Account<'info, SwapProposal>,

    #[account(
        seeds = [PLATFORM_SEED, PROGRAM_ID],
        bump = swap_config.bump,
    )]
    pub swap_config: Account<'info, SwapPlatformConfig>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl<'info> CreateProposalContext<'info> {
    pub fn execute(&mut self, params: CreateProposalParams, bump: u8) -> Result<()> {
        // validate input
        if self.swap_config.max_allowed_options < params.swap_options.len() as u8 {
            return Err(SwapError::InvalidValue.into());
        }
        if self.swap_config.max_allowed_items < params.offered_items.len() as u8 {
            return Err(SwapError::InvalidValue.into());
        }

        // set data
        let swap_proposal = &mut self.swap_proposal;
        swap_proposal.owner = *self.proposal_owner.key;
        swap_proposal.bump = bump;
        swap_proposal.swap_options = params.swap_options;
        swap_proposal.offered_items = params.offered_items;
        swap_proposal.expired_at = params.expired_at;

        // Now to validate data state
        swap_proposal.handle_post_initialized().unwrap();

        //
        // // now we need to transfer appropriate assets.
        // transfer(
        //     CpiContext::new(
        //         self.token_program.to_account_info(),
        //         Transfer {
        //             from: self.sender_stake_token_account.to_account_info(),
        //             to: self.stake_token_account.to_account_info(),
        //             authority: self.sender.to_account_info(),
        //         },
        //     ),
        //     incoming_amount,
        // )?;


        Ok(())
    }
}