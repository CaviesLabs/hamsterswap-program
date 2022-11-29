//! Macros
pub use crate::*;

#[macro_export]
macro_rules! token_account_signer {
    ($seed: expr, $mint_account: expr, $bump: expr) => {
        &[&[$seed, &$mint_account, $bump][..]]
    };
}
