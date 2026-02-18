use anchor_lang::prelude::*;

use crate::state::VaultConfig;
use crate::errors::VaultError;

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct RemoveFromWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault_config"],
        bump = vault_config.config_bump,
        has_one = admin @ VaultError::Unauthorized,
        constraint = !vault_config.whitelist.is_empty() @ VaultError::UserNotFound,
        realloc = VaultConfig::size_with_entries(vault_config.whitelist.len() - 1),
        realloc::payer = admin,
        realloc::zero = false,
    )]
    pub vault_config: Account<'info, VaultConfig>,

    pub system_program: Program<'info, System>,
}

impl<'info> RemoveFromWhitelist<'info> {
    pub fn remove_from_whitelist(&mut self, user: Pubkey) -> Result<()> {
        let idx = self.vault_config.whitelist.iter()
            .position(|e| e.address == user)
            .ok_or(VaultError::UserNotFound)?;

        self.vault_config.whitelist.remove(idx);

        msg!("Removed {} from whitelist", user);
        Ok(())
    }
}
