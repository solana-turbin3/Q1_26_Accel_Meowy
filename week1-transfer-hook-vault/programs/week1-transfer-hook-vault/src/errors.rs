use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("User is not whitelisted")]
    NotWhitelisted,

    #[msg("Transfer amount exceeds whitelist limit")]
    AmountExceedsLimit,

    #[msg("User is already whitelisted")]
    AlreadyWhitelisted,

    #[msg("User not found in whitelist")]
    UserNotFound,

    #[msg("Unauthorized: only admin can perform this action")]
    Unauthorized,

    #[msg("The transfer hook was not invoked during a transfer")]
    NotTransferring,
}
