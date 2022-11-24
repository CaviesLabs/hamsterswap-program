use anchor_lang::prelude::*;
use anchor_lang::solana_program::{system_program, sysvar};

use std::collections::HashMap;

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

    pub fn initialize(ctx: Context<InitializeSwapPlatformContext>, params: InitializeSwapPlatformParams) -> Result<()> {
        // process
        ctx.accounts.execute(
            params,
            *ctx.bumps.get("swap_config").unwrap(),
        ).unwrap();

        Ok(())
    }

    pub fn update_swap_config(ctx: Context<UpdateSwapPlatformContext>, params: UpdateSwapPlatformParams) -> Result<()> {
        ctx.accounts.execute(params).unwrap();

        Ok(())
    }
}