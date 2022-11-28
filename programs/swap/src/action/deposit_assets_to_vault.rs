use crate::*;

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Debug, PartialEq)]
pub struct DepositAssetsToVaultParams {
    pub swap_token_vault_bump: u8,
    pub swap_item_id: String
}

#[derive(Accounts)]
#[instruction(params: DepositAssetsToVaultParams)]
pub struct DepositAssetsContext<'info> {
    #[account(mut)]
    pub participant: Signer<'info>,

    pub mint_account: Account<'info, Mint>,

    #[account(mut)]
    /// CHECK: the participant token account can be verified later
    pub participant_token_account: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [PLATFORM_SEED],
        bump = swap_registry.bump,
    )]
    pub swap_registry: Account<'info, SwapPlatformRegistry>,

    #[account(
        mut,
        seeds = [TOKEN_ACCOUNT_SEED, mint_account.key().as_ref()],
        bump = params.swap_token_vault_bump
    )]
    pub swap_token_vault: Account<'info, TokenAccount>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = spl_token::ID)]
    pub token_program: Program<'info, Token>,
}

impl<'info> DepositAssetsContext<'info> {
    pub fn execute(&mut self, params: DepositAssetsToVaultParams) -> Result<()> {
        //
        // // transfer token first
        // token::transfer(
        //     CpiContext::new(
        //         self.token_program.to_account_info(),
        //         Transfer {
        //             from: self.sender_token_account.to_account_info(),
        //             to: self.stake_token_account.to_account_info(),
        //             authority: self.sender.to_account_info(),
        //         },
        //     ),
        //     amount,
        // );

        Ok(())
    }
}