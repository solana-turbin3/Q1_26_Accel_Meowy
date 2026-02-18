use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct WhitelistEntry {
    pub address: Pubkey,
    pub amount: u64,
}

impl WhitelistEntry {
    pub const SIZE: usize = 32 + 8;
}

#[account]
pub struct VaultConfig {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub vault_bump: u8,
    pub config_bump: u8,
    pub whitelist: Vec<WhitelistEntry>,
}

impl VaultConfig {
    // 8 (discriminator) + 32 (admin) + 32 (mint) + 1 (vault_bump) + 1 (config_bump) + 4 (vec len)
    pub const BASE_SIZE: usize = 8 + 32 + 32 + 1 + 1 + 4;

    pub fn size_with_entries(n: usize) -> usize {
        Self::BASE_SIZE + n * WhitelistEntry::SIZE
    }
}
