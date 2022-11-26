use crate::*;

#[derive(Accounts)]
pub struct CreateTokenVaultContext<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [PLATFORM_SEED],
        bump = swap_registry.bump,
        has_one = owner
    )]
    pub swap_registry: Account<'info, SwapPlatformRegistry>,

    pub mint_token_account: Account<'info, Mint>,

    #[account(init,
        token::mint = mint_token_account,
        token::authority = swap_registry,
        seeds = [TOKEN_ACCOUNT_SEED, mint_token_account.key().as_ref()],
        payer = owner,
        bump
    )]
    pub swap_token_vault_account: Account<'info, TokenAccount>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = spl_token::ID)]
    pub token_program: Program<'info, Token>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> CreateTokenVaultContext<'info> {
    pub fn execute(&mut self, bump: u8) -> Result<()> {
        Ok(())
    }
}