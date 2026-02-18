use anchor_lang::prelude::*;

use crate::state::{VaultConfig, WhitelistEntry};
use crate::errors::VaultError;

#[derive(Accounts)]
#[instruction(user: Pubkey, amount: u64)]
pub struct AddToWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault_config"],
        bump = vault_config.config_bump,
        has_one = admin @ VaultError::Unauthorized,
        realloc = VaultConfig::size_with_entries(vault_config.whitelist.len() + 1),
        realloc::payer = admin,
        realloc::zero = false,
    )]
    pub vault_config: Account<'info, VaultConfig>,

    pub system_program: Program<'info, System>,
}

impl<'info> AddToWhitelist<'info> {
    pub fn add_to_whitelist(&mut self, user: Pubkey, amount: u64) -> Result<()> {
        require!(
            !self.vault_config.whitelist.iter().any(|e| e.address == user),
            VaultError::AlreadyWhitelisted
        );

        self.vault_config.whitelist.push(WhitelistEntry {
            address: user,
            amount,
        });

        msg!("Added {} to whitelist with limit {}", user, amount);
        Ok(())
    }
}
