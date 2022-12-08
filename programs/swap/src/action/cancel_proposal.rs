use crate::*;

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Debug, PartialEq)]
pub struct CancelProposalParams {
    id: String
}

// Define the context, passed in parameters when trigger from deployer.
#[derive(Accounts)]
#[instruction(params: CancelProposalParams)]
pub struct CancelProposalContext<'info> {
    // We define the fee payer
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [PROPOSAL_SEED, params.id.as_bytes().as_ref()],
        bump = swap_proposal.bump,
    )]
    pub swap_proposal: Account<'info, SwapProposal>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl<'info> CancelProposalContext<'info> {
    pub fn execute(&mut self, params: CancelProposalParams) -> Result<()> {
        if self.swap_proposal.is_proposal_cancelable_for(&self.signer.key) {
            self.swap_proposal.status = SwapProposalStatus::Canceled;

            // emit event
            swap_emit!(
               ProposalCanceled {
                    actor: self.swap_proposal.owner.key().clone(),
                    status: SwapProposalStatus::Canceled,
                    id: self.swap_proposal.id.clone(),
                    proposal_key: self.swap_proposal.key().clone(),
                }
            );

            return Ok(());
        }

        // ok
        return Err(SwapError::ProposalCannotBeCanceled.into());
    }

}