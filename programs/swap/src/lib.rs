use anchor_lang::prelude::*;
use anchor_lang::solana_program::{system_program, sysvar};

use anchor_spl::token::*;

use arrayref::array_ref;

pub mod action;
pub mod error;
pub mod event;
pub mod state;
pub mod constants;

pub use action::*;
pub use constants::*;
pub use error::*;
pub use state::*;
pub use event::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod swap {
    use std::borrow::Borrow;
    use super::*;

    // Initialize contract once
    pub fn initialize(ctx: Context<InitializeSwapPlatformContext>, params: InitializeSwapPlatformParams) -> Result<()> {
        // process
        ctx.accounts.execute(
            params,
            *ctx.bumps.get("swap_registry").unwrap(),
        ).unwrap();

        // Program result should be ok.
        Ok(())
    }

    // Deployer can update swap config later
    pub fn update_swap_registry(ctx: Context<UpdateSwapPlatformContext>, params: UpdateSwapPlatformParams) -> Result<()> {
        // execute with context
        ctx.accounts.execute(params).unwrap();

        // Program result should be ok.
        Ok(())
    }

    // Create proposal, public to anyone
    pub fn create_proposal(ctx: Context<CreateProposalContext>, params: CreateProposalParams) -> Result<()> {
        let id = random_number(
            ctx.accounts.recent_slothashes.borrow(),
            params.id.to_string()
        );

        ctx.accounts.execute(
            params,
            id.to_string(),
            *ctx.bumps.get("swap_proposal").unwrap(),
        ).unwrap();

        Ok(())
    }

    // Create proposal, public to anyone
    pub fn create_token_vault(ctx: Context<CreateTokenVaultContext>) -> Result<()> {
        ctx.accounts.execute(
            *ctx.bumps.get("swap_token_vault").unwrap(),
        ).unwrap();

        Ok(())
    }
}