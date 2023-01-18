use std::borrow::BorrowMut;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{system_program, sysvar};
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use arrayref::array_ref;
use solana_address_lookup_table_program::*;

pub mod action;
pub mod error;
pub mod event;
pub mod state;
pub mod constants;
pub mod macros;

pub use action::*;
pub use constants::*;
pub use error::*;
pub use state::*;
pub use event::*;
pub use macros::*;

declare_id!("EdeRcNsVGU1s1NXZZo8FhLD8iePxvoUCdbvwVGnj778f");

#[program]
pub mod swap {
    use super::*;

    // Initialize contract once
    pub fn initialize(
        ctx: Context<InitializeSwapPlatformContext>,
        params: InitializeSwapPlatformParams
    ) -> Result<()> {
        // process
        ctx.accounts.execute(
            params,
            *ctx.bumps.get("swap_registry").unwrap(),
        ).unwrap();

        // Program result should be ok.
        Ok(())
    }

    // Deployer can update swap config later
    pub fn update_swap_registry(
        ctx: Context<UpdateSwapPlatformContext>,
        params: UpdateSwapPlatformParams
    ) -> Result<()> {
        // execute with context
        ctx.accounts.execute(params).unwrap();

        // Program result should be ok.
        Ok(())
    }

    // Create proposal, public to anyone
    pub fn create_token_vault(
        ctx: Context<CreateTokenVaultContext>
    ) -> Result<()> {
        ctx.accounts.execute(
            *ctx.bumps.get("swap_token_vault").unwrap(),
        ).unwrap();

        Ok(())
    }

    // Create proposal, public to anyone
    pub fn create_proposal(
        ctx: Context<CreateProposalContext>,
        params: CreateProposalParams
    ) -> Result<()> {
        ctx.accounts.execute(
            params,
            *ctx.bumps.get("swap_proposal").unwrap(),
        ).unwrap();

        Ok(())
    }

    // Create proposal, public to anyone
    pub fn cancel_proposal(
        ctx: Context<CancelProposalContext>,
        params: CancelProposalParams
    ) -> Result<()> {
        ctx.accounts.execute(params).unwrap();
        Ok(())
    }

    // Deposit or fulfilling the proposal
    pub fn transfer_assets_to_vault(
        ctx: Context<TransferAssetsToVaultContext>,
        params: TransferAssetsToVaultParams
    ) -> Result<()> {
        ctx.accounts.execute(params).unwrap();

        Ok(())
    }

    // Withdrawing or redeeming the proposal
    pub fn transfer_assets_from_vault(
        ctx: Context<TransferAssetsFromVaultContext>,
        params: TransferAssetsFromVaultParams
    ) -> Result<()> {
        ctx.accounts.execute(params).unwrap();

        Ok(())
    }

    // modify address lookup table
    pub fn modify_address_lookup_table(
        ctx: Context<ModifyAddressLookupTableContext>,
        params: ModifyAddressLookupTableParams
    ) -> Result<()> {
        ctx.accounts.execute(params).unwrap();

        Ok(())
    }

}