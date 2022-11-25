use anchor_lang::prelude::*;
use anchor_lang::solana_program::{system_program};

use anchor_spl::token::*;

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
    use super::*;

    // Initialize contract once
    pub fn initialize(ctx: Context<InitializeSwapPlatformContext>, params: InitializeSwapPlatformParams) -> Result<()> {
        // process
        ctx.accounts.execute(
            params,
            *ctx.bumps.get("swap_config").unwrap(),
        ).unwrap();

        // Program result should be ok.
        Ok(())
    }

    // Deployer can update swap config later
    pub fn update_swap_config(ctx: Context<UpdateSwapPlatformContext>, params: UpdateSwapPlatformParams) -> Result<()> {
        // execute with context
        ctx.accounts.execute(params).unwrap();

        // Program result should be ok.
        Ok(())
    }
}