use crate::*;
use std::borrow::{Borrow, BorrowMut};

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Debug, PartialEq)]
pub enum TransferActionType {
    #[default]
    Redeeming,
    Withdrawing
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Debug, PartialEq)]
pub struct TransferAssetsFromVaultParams {
    pub swap_registry_bump: u8,
    pub swap_token_vault_bump: u8,
    pub proposal_id: String,
    pub action_type: TransferActionType,
    pub swap_item_id: String,
}

#[derive(Accounts)]
#[instruction(params: TransferAssetsFromVaultParams)]
pub struct TransferAssetsFromVaultContext<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub mint_account: Account<'info, Mint>,

    #[account(
        seeds = [PLATFORM_SEED],
        bump = swap_registry.bump,
    )]
    pub swap_registry: Account<'info, SwapPlatformRegistry>,

    #[account(mut)]
    /// CHECK: the signer token account can be verified later
    pub signer_token_account: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [PROPOSAL_SEED, params.proposal_id.as_bytes().as_ref()],
        bump = swap_proposal.bump,
    )]
    pub swap_proposal: Account<'info, SwapProposal>,

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

impl<'info> TransferAssetsFromVaultContext<'info> {
    pub fn execute(&mut self, params: TransferAssetsFromVaultParams) -> Result<()> {
        // Check and route for redeeming
        if params.action_type == TransferActionType::Redeeming {
            return self.redeem(params);
        }

        // Check and route for withdrawal
        if params.action_type == TransferActionType::Withdrawing {
            return self.withdraw(params);
        }

        return Err(SwapError::InvalidValue.into());
    }

    fn redeem(&mut self, params: TransferAssetsFromVaultParams) -> Result<()> {
        let current_params = params.clone();
        let swap_proposal = self.swap_proposal.borrow();

        // check whether the proposal is still open for redeeming
        if !swap_proposal.is_proposal_redeemable() {
            return Err(SwapError::RedeemIsNotAvailable.into());
        }

        swap_emit!(
            ItemRedeemed {
                id: params.swap_item_id.clone(),
                proposal_key: swap_proposal.key().clone(),
                status: SwapItemStatus::Redeemed,
                actor: self.signer.key().clone()
            }
        );

        // Check whether the signer is allowed to redeem.
        if swap_proposal.is_proposal_owner(self.signer.key().clone()) {
            return self.transfer_asking_items(current_params, SwapItemStatus::Redeemed);
        }

        if swap_proposal.is_fulfilled_participant(self.signer.key().clone()) {
            return self.transfer_offered_items(current_params, SwapItemStatus::Redeemed);
        }

        return Err(SwapError::InvalidValue.into());
    }

    fn withdraw(&mut self, params: TransferAssetsFromVaultParams) -> Result<()> {
        let current_params = params.clone();
        let swap_proposal = self.swap_proposal.borrow();

        // check whether the proposal is still open for withdrawal
        if !swap_proposal.is_proposal_withdrawable() {
            return Err(SwapError::WithdrawalIsNotAvailable.into());
        }

        // emit
        swap_emit!(
            ItemWithdrawn {
                id: params.swap_item_id.clone(),
                proposal_key: swap_proposal.key().clone(),
                status: SwapItemStatus::Redeemed,
                actor: self.signer.key().clone()
            }
        );

        // Check whether the signer is allowed to withdraw.
        if swap_proposal.is_proposal_owner(self.signer.key().clone()) {
            return self.transfer_offered_items(current_params, SwapItemStatus::Withdrawn);
        }

        if swap_proposal.is_fulfilled_participant(self.signer.key().clone()) {
            return self.transfer_asking_items(current_params, SwapItemStatus::Withdrawn);
        }

        return Err(SwapError::InvalidValue.into());
    }

    fn transfer_asking_items(&mut self, params: TransferAssetsFromVaultParams, desired_item_status: SwapItemStatus) -> Result<()> {
        let current_params = params.clone();
        let swap_proposal = self.swap_proposal.borrow_mut();
        let option_id = swap_proposal.fulfilled_with_option_id.clone();

        // find the option id
        let desired_option = swap_proposal.swap_options
            .iter_mut()
            .find(|x| x.id == option_id.clone())
            .unwrap();

        // find the swap item
        let mut item = desired_option.asking_items
            .iter_mut()
            .find(|x| x.id == current_params.swap_item_id.clone())
            .unwrap();

        if item.status != SwapItemStatus::Deposited {
            return Err(SwapError::TransferTokenFromVaultIsNotAvailable.into());
        }

        // find the bump to sign with the pda
        let bump = &[params.swap_registry_bump][..];
        let signer = token_account_signer!(
            PLATFORM_SEED,
            bump
        );

        // transfer the token
        token::transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.swap_token_vault.to_account_info(),
                    to: self.signer_token_account.to_account_info(),
                    authority: self.swap_registry.to_account_info(),
                },
                signer
            ),
            item.amount,
        ).unwrap();

        // update the item status
        item.status = desired_item_status;

        Ok(())
    }

    fn transfer_offered_items(&mut self, params: TransferAssetsFromVaultParams, status: SwapItemStatus) -> Result<()> {
        let swap_proposal = self.swap_proposal.borrow_mut();

        // find the swap item
        let mut item = swap_proposal.offered_items
            .iter_mut()
            .find(|x| x.id == params.swap_item_id)
            .unwrap();

        // Redeem is not available
        if item.status != SwapItemStatus::Deposited {
            return Err(SwapError::TransferTokenFromVaultIsNotAvailable.into());
        }

        // find the bump to sign with the pda
        let bump = &[params.swap_registry_bump][..];
        let signer = token_account_signer!(
            PLATFORM_SEED,
            bump
        );

        // transfer the token
        token::transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.swap_token_vault.to_account_info(),
                    to: self.signer_token_account.to_account_info(),
                    authority: self.swap_registry.to_account_info(),
                },
                signer
            ),
            item.amount,
        ).unwrap();

        // update the item status
        item.status = status;

        return Ok(());
    }
}