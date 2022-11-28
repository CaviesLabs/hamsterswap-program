use crate::*;

#[error_code]
pub enum SwapError {
    // System error
    #[msg("The program was already initialized")]
    AlreadyInitialized,
    #[msg("The mint account was existed")]
    MintAccountExisted,
    // Business errors
    #[msg("Only Platform Admin")]
    OnlyAdministrator,
    #[msg("Only Owner")]
    OnlyOwner,
    #[msg("Only Buyer")]
    OnlyBuyer,
    #[msg("Only Seller")]
    OnlySeller,
    #[msg("Order expired")]
    OrderExpired,
    #[msg("Invalid Offer")]
    InvalidOffer,
    #[msg("Invalid value")]
    InvalidValue,
    #[msg("Invalid value")]
    UnAllowedMintToken,
    #[msg("Proposal cannot be canceled")]
    ProposalCannotBeCanceled,

}
