use crate::*;

#[derive(Accounts)]
pub struct CreateTokenVaultContext<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [PLATFORM_SEED],
        bump = swap_registry.bump,
    )]
    pub swap_registry: Account<'info, SwapPlatformRegistry>,

    pub mint_account: Account<'info, Mint>,

    #[account(init,
        token::mint = mint_account,
        token::authority = swap_registry,
        seeds = [TOKEN_ACCOUNT_SEED, mint_account.key().as_ref()],
        payer = signer,
        bump
    )]
    pub swap_token_vault: Account<'info, TokenAccount>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = spl_token::ID)]
    pub token_program: Program<'info, Token>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> CreateTokenVaultContext<'info> {
    pub fn execute(&mut self, bump: u8) -> Result<()> {
        // Avoid adding duplicated value
        if self.swap_registry.is_mint_account_existed(self.mint_account.key().clone()) {
            return Err(SwapError::MintAccountExisted.into());
        }

        // Now we push into the allowed mint tokens array.
        self.swap_registry.allowed_mint_accounts.push(
            MintInfo {
                mint_account: self.mint_account.key().clone(),
                token_account: self.swap_token_vault.key(),
                bump,
                is_enabled: true
            }
        );

        // emit event
        swap_emit!(
            VaultCreated {
                actor: self.signer.key().clone(),
                authority: self.swap_registry.key().clone(),
                associated_account: self.swap_token_vault.key().clone(),
                mint_account: self.mint_account.key().clone()
            }
        );

        Ok(())
    }
}