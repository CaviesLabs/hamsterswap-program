//! Events emitted.
use crate::*;

// Log to Program Log with a prologue so transaction scraper knows following line is valid mango log
#[macro_export]
macro_rules! emit {
    ($e:expr) => {
        msg!("swap-log");
        emit!($e);
    };
}

/// Emitted when a [JoinEvent] is created.
#[event]
pub struct ProposalCreated {
    #[index]
    pub owner: Pubkey,
    pub proposal: Pubkey,
    pub expired_at: i64,
}