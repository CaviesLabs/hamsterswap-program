//! Events emitted.
use crate::*;

// Log to Program Log with a prologue so transaction scraper knows following line is valid mango log
#[macro_export]
macro_rules! swap_emit {
    ($e:expr) => {
        msg!("swap-log");
        emit!($e);
    };
}

/// Emitted when a [SwapConfigUpdated] is created.
#[event]
pub struct SwapConfigUpdated {
    #[index]
    pub actor: Pubkey,
    pub max_allowed_items: u8,
    pub max_allowed_options: u8,
}


/// Emitted when a [VaultCreated] is created.
#[event]
pub struct VaultCreated {
    #[index]
    pub actor: Pubkey,
    #[index]
    pub authority: Pubkey,
    #[index]
    pub mint_account: Pubkey,
    #[index]
    pub associated_account: Pubkey,
}

/// Emitted when a [ProposalCreated] is created.
#[event]
pub struct ProposalCreated {
    #[index]
    pub actor: Pubkey,
    #[index]
    pub proposal_key: Pubkey,
    #[index]
    pub id: String,
    pub expired_at: i64,
}


/// Emitted when a [ProposalDeposited] is created.
#[event]
pub struct ProposalDeposited {
    #[index]
    pub actor: Pubkey,
    #[index]
    pub proposal_key: Pubkey,
    #[index]
    pub id: String,
    pub status: SwapProposalStatus,
}

/// Emitted when a [ProposalFulfilled] is created.
#[event]
pub struct ProposalFulfilled {
    #[index]
    pub actor: Pubkey,
    #[index]
    pub proposal_key: Pubkey,
    #[index]
    pub id: String,
    pub status: SwapProposalStatus,
}

/// Emitted when a [ProposalCanceled] is created.
#[event]
pub struct ProposalCanceled {
    #[index]
    pub actor: Pubkey,
    #[index]
    pub proposal_key: Pubkey,
    #[index]
    pub id: String,
    pub status: SwapProposalStatus,
}

/// Emitted when a [ItemDeposited] is created.
#[event]
pub struct ItemDeposited {
    #[index]
    pub actor: Pubkey,
    #[index]
    pub proposal_key: Pubkey,
    #[index]
    pub id: String,
    pub status: SwapItemStatus,
}

/// Emitted when a [ItemWithdrawn] is created.
#[event]
pub struct ItemWithdrawn {
    #[index]
    pub actor: Pubkey,
    #[index]
    pub proposal_key: Pubkey,
    #[index]
    pub id: String,
    pub status: SwapItemStatus,
}

/// Emitted when a [ItemRedeemed] is created.
#[event]
pub struct ItemRedeemed {
    #[index]
    pub actor: Pubkey,
    #[index]
    pub proposal_key: Pubkey,
    #[index]
    pub id: String,
    pub status: SwapItemStatus,
}

