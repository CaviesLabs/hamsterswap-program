use crate::*;
use std::borrow::Borrow;

// ================ Swap Platform Config ================ //
// Here we define the account state that holds the administration info.
#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Copy, Debug, PartialEq)]
pub struct MintInfo {
    // Whether the mint token is active or not.
    pub is_enabled: bool,
    pub mint_account: Pubkey,
    pub token_account: Pubkey,
    pub bump: u8
}

#[account]
#[derive(Default)]
pub struct SwapPlatformRegistry {
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
    pub allowed_mint_accounts: Vec<MintInfo>,
}

// Define handler
impl SwapPlatformRegistry {
    // handle data integrity after initialization
    pub fn handle_post_initialized(&mut self) -> Result<()> {
        if self.was_initialized == false {
            self.was_initialized = true;
            return Ok(())
        }

        msg!("ERROR::PLATFORM::ALREADY_INITIALIZED");
        return Err(SwapError::AlreadyInitialized.into());
    }

    // Check whether the mint account was previously added or not.
    pub fn is_mint_account_existed(&self, mint_account: Pubkey) -> bool {
        return self.allowed_mint_accounts.iter()
            .map(|allowed_mint_account| allowed_mint_account.mint_account)
            .filter(|&mint_account_key| mint_account_key == mint_account.key().clone())
            .count() >= 1;
    }

    // Check whether the mint account was enabled or not
    pub fn is_mint_account_enabled(&self, mint_account: Pubkey) -> bool {
        return self.allowed_mint_accounts.iter()
            .filter(|&mint_info|
                    mint_info.mint_account == mint_account.key().clone()
                    && mint_info.is_enabled == true
            )
            .count() >= 1;
    }

    // Get mint info
    pub fn get_mint_info(&self, mint_account: Pubkey) -> &MintInfo {
        return self.allowed_mint_accounts.iter()
            .find(|&mint_account_key| mint_account_key.mint_account == mint_account.key().clone())
            .unwrap()
            .borrow();
    }
}

// ================ Swap Item Interface ================ //
// Here we define the swap option type
#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Copy, Debug, PartialEq)]
pub enum SwapItemType {
    // Define the onchain swap item
    #[default]
    NFT,

    // Define the offchain swap item
    Currency,
}

// Here we define the swap option type
#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Copy, Debug, PartialEq)]
pub enum SwapItemStatus {
    // Define the onchain swap item
    #[default]
    Created,

    // Define the item was deposited
    Deposited,

    // Define the item was redeemed
    Redeemed,

    // Define the item was withdrawn
    Withdrawn
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, Debug, PartialEq)]
pub struct SwapItem {
    // Define id of the swap item.
    pub id: String,

    // Define the mint account
    pub mint_account: Pubkey,

    // Define the amount of deposited token
    pub amount: u64,

    // Define swap item status
    pub status: SwapItemStatus,

    // Define the item type
    pub item_type: SwapItemType,

    // Define the owner of the item
    pub owner: Pubkey,
}

// Implement the swap item functions
impl SwapItem {
    // Define default value
    fn default() -> SwapItem {
        SwapItem {
            id: String::default(),
            owner: Pubkey::default(),
            mint_account: Pubkey::default(),
            item_type: SwapItemType::NFT,
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

    // Declared that the proposal is deposited
    Deposited,

    // Declare that the proposal is finalized
    Fulfilled,

    // Declare that the proposal is canceled
    Canceled,

    // Declare that the proposal is fully redeemed by both participant and proposal owner
    Redeemed,

    // Declare that the proposal is fully redeemed by both participant and proposal owner
    Withdrawn,
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
    // Id of the proposal
    pub id: String,

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
    pub fn default() -> SwapProposal {
        SwapProposal {
            bump: 0,
            id: String::default(),
            owner: Pubkey::default(),
            fulfilled_by: Pubkey::default(),
            fulfilled_with_option_id: String::default(),
            status: SwapProposalStatus::Created,
            offered_items: vec![],
            swap_options: vec![],
            expired_at: 0,
        }
    }

    // Define whether the proposal is still in time window
    pub fn is_proposal_still_in_time_window(&self) -> bool {
        return self.expired_at > Clock::get().unwrap().unix_timestamp as u64
    }

    // Check whether the proposal owner is the signer.
    pub fn is_proposal_owner(&self, signer: Pubkey) -> bool {
        return self.owner == signer.key().clone();
    }

    // Check whether the proposal owner is the signer.
    pub fn is_fulfilled_participant(&self, signer: Pubkey) -> bool {
        return self.fulfilled_by == signer.key().clone();
    }

    // Define the state that the proposal is still open for participants.
    pub fn is_proposal_open_for_participants(&self) -> bool {
        return self.is_proposal_still_in_time_window()
            && self.status == SwapProposalStatus::Deposited // need to be updated once depositing occurs
            && self.fulfilled_by == Pubkey::default() // need to be updated once depositing occurs
            && self.fulfilled_with_option_id == String::default(); // need to be updated once depositing occurs
    }

    // Define whether the proposal can be canceled for a pubkey.
    pub fn is_proposal_cancelable_for(&self, signer: &Pubkey) -> bool {
        return (!self.is_proposal_redeemable() && !self.is_proposal_withdrawable())
            && (self.owner.key() == signer.key() || self.fulfilled_by.key() == signer.key());
    }

    // Define whether the state is open for depositing.
    pub fn is_proposal_open_for_depositing(&self) -> bool {
        return self.is_proposal_still_in_time_window()
            && self.status == SwapProposalStatus::Created;
    }

    // Define whether the state is open for depositing.
    pub fn is_proposal_open_for_fulfilling(&self, option_id: String, participant: Pubkey) -> bool {
        return self.is_proposal_still_in_time_window()
            && self.status == SwapProposalStatus::Deposited
            && (
            self.fulfilled_with_option_id == option_id
                || self.fulfilled_with_option_id == String::default()
        )
            && (
            self.fulfilled_by == participant.key().clone() ||
                self.fulfilled_by == Pubkey::default().clone()
        );
    }

    // Define the state that the proposal is redeemable (the swap is completed)
    pub fn is_proposal_redeemable(&self) -> bool {
        return !self.is_proposal_open_for_participants()
                && self.status == SwapProposalStatus::Fulfilled; // need to be updated once depositing is completed.
    }

    // Define the state that the proposal is withdrawable (the swap is canceled).
    pub fn is_proposal_withdrawable(&self) -> bool {
        return !self.is_proposal_open_for_participants()
            && self.status == SwapProposalStatus::Canceled; // need to be updated once the proposal owner cancel the proposal.
    }

    // Update redeem status
    pub fn update_redeemed_status(&mut self) -> Result<()> {
        let offered_items = &self.offered_items.clone();
        let fulfilled_option_id = self.fulfilled_with_option_id.clone();

        let fulfilled_option = &self.swap_options
            .clone()
            .into_iter()
            .find(|option| option.id == fulfilled_option_id)
            .unwrap();

        // all offered items must be redeemed
        let all_offered_items_redeemed = offered_items
            .into_iter()
            .filter(|item| item.status != SwapItemStatus::Redeemed)
            .count() == 0;

        // all asking items must be redeemed
        let all_asking_items_redeemed = fulfilled_option.asking_items
            .clone()
            .into_iter()
            .filter(|item| item.status != SwapItemStatus::Redeemed)
            .count() == 0;

        // update final status for the proposal
        if all_offered_items_redeemed && all_asking_items_redeemed {
            self.status = SwapProposalStatus::Redeemed;
        }

        return Ok(());
    }

    // Update redeem status
    pub fn update_withdrawn_status(&mut self) -> Result<()> {
        let offered_items = &self.offered_items.clone();
        let mut all_asking_items_withdrawn = true;
        let fulfilled_option_id = self.fulfilled_with_option_id.clone();

        if fulfilled_option_id != String::default() {
            let fulfilled_option = &self.swap_options
                .clone()
                .into_iter()
                .find(|option| option.id == fulfilled_option_id)
                .unwrap();

            // all asking items must be redeemed
            all_asking_items_withdrawn = fulfilled_option.asking_items.clone()
                .into_iter()
                .filter(|item| item.status != SwapItemStatus::Withdrawn)
                .count() == 0;
        }

        // all offered items must be redeemed
        let all_offered_items_withdrawn = offered_items
            .into_iter()
            .filter(|item| item.status != SwapItemStatus::Withdrawn)
            .count() == 0;

        // update final status for the proposal
        if all_asking_items_withdrawn && all_offered_items_withdrawn {
            self.status = SwapProposalStatus::Withdrawn;
        }

        return Ok(());
    }
}