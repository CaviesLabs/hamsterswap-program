use crate::*;

#[error_code]
pub enum SwapError {
    // System error
    #[msg("The program was already initialized")]
    AlreadyInitialized,
    // Business errors
    #[msg("Only Platform Admin")]
    OnlyAdmin,
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
    InvalidValue
}
