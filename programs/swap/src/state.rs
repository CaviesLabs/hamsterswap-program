use crate::*;

// ================ Swap Platform Config ================ //
// Here we define the account state that holds the administration info.
#[account]
#[derive(Default)]
pub struct SwapPlatformConfig {
    // Define owner
    pub owner: Pubkey,

    // define whether the config was initialized or not, the contract must be only initialized once.
    pub was_initialized: bool,

    // Bump to help define the PDA of swap account
    pub bump: u8,

    // define max item can be traded.
    pub max_allowed_items: u8,

    // define max allowed options can be asked.
    pub max_allowed_options: u8,

    // define whitelisted mint token account
    pub allowed_mint_accounts: Vec<Pubkey>,
}

// Define handler
impl SwapPlatformConfig {
    pub fn handle_post_initialized(&mut self) -> Result<()> {
        if self.was_initialized == false {
            self.was_initialized = true;
            return Ok(())
        }

        msg!("ERROR::PLATFORM::ALREADY_INITIALIZED");
        return Err(SwapError::AlreadyInitialized.into());
    }
}

// ================ Swap Item Interface ================ //
// Here we define the swap option type
#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Copy, Debug, PartialEq)]
pub enum SwapItemType {
    // Define the onchain swap item
    #[default]
    OnChain,

    // Define the offchain swap item
    OffChain,
}

// Here we define the swap option type
#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Copy, Debug, PartialEq)]
pub enum SwapItemStatus {
    // Define the onchain swap item
    #[default]
    Created,

    // Define the onchain swap item
    Deposited,

    // Define the offchain swap item
    Redeemed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Copy, Debug, PartialEq)]
pub struct SwapItem {
    // Define the owner item
    pub owner: Pubkey,

    // Define the mint account
    pub mint_account: Pubkey,

    // Define the associated account of order so that user can deposit to this address.
    pub proposal_token_account: Pubkey,

    // Define the amount of deposited token
    pub amount: u64,

    // Define swap item status
    pub status: SwapItemStatus,

    // Define the item type
    pub item_type: SwapItemType,
}

// Implement the swap item functions
impl SwapItem {
    // Define default value
    fn default() -> SwapItem {
        SwapItem {
            owner: Pubkey::default(),
            item_type: SwapItemType::OnChain,
            mint_account: Pubkey::default(),
            proposal_token_account: Pubkey::default(),
            status: SwapItemStatus::Created,
            amount: 0,
        }
    }

    // Define deposit function
    pub fn handle_post_deposited(&mut self) {
        self.status = SwapItemStatus::Deposited;
    }

    pub fn handle_post_redeemed(&mut self) {
        self.status = SwapItemStatus::Redeemed;
    }
}

// ================ Swap Option Interface ================ //
#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Copy, Debug, PartialEq)]
pub enum SwapProposalStatus {
    // Declare that the proposal is created
    #[default]
    Created,

    // Declare that the proposal is finalized
    Fulfilled,

    // Declare that the proposal is canceled
    Canceled
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Debug, PartialEq)]
pub struct SwapOption {
    // Swap option id
    pub id: String,

    // asking item included in swap option
    pub asking_items: Vec<SwapItem>
}

// Here we define the account state that holds the swap order. SwapOrder will be the PDA.
#[account]
#[derive(Default)]
pub struct SwapProposal {
    // Bump to help define the PDA of swap order.
    pub bump: u8,

    // Define the owner of the proposal
    pub owner: Pubkey,

    // Define the buyer that fulfil this proposal
    pub fulfilled_by: Pubkey,

    // Define the option has been fulfilled for this proposal.
    pub fulfilled_with_option_id: String,

    // Swap items that have been offered.
    pub offered_items: Vec<SwapItem>,

    // Swap options that have been asked.
    pub swap_options: Vec<SwapOption>,

    // Expiry date
    pub expired_at: u64,

    // Define the proposal status
    pub status: SwapProposalStatus,
}

// Implement some domain logic
impl SwapProposal {
    // Define default value
    fn default() -> SwapProposal {
        SwapProposal {
            bump: 0,
            owner: Pubkey::default(),
            fulfilled_by: Pubkey::default(),
            fulfilled_with_option_id: "".to_string(),
            status: SwapProposalStatus::Created,
            offered_items: vec![],
            swap_options: vec![],
            expired_at: 0,
        }
    }

    // validate input
    pub fn handle_post_initialized(&mut self) -> Result<()> {
        if self.bump == 0 {
            return Err(SwapError::InvalidValue.into());
        }

        if self.owner == Pubkey::default() {
            return Err(SwapError::InvalidValue.into());
        }

        if self.offered_items.len() < 1 {
            return Err(SwapError::InvalidValue.into());
        }

        if self.swap_options.len() < 1 {
            return Err(SwapError::InvalidValue.into());
        }

        if self.expired_at < Clock::get().unwrap().unix_timestamp as u64 {
            return Err(SwapError::InvalidValue.into());
        }

        return Ok(());
    }
}