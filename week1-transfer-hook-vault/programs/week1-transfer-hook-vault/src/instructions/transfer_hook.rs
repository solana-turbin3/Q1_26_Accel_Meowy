use std::cell::RefMut;

use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::spl_token_2022::{
        extension::{
            transfer_hook::TransferHookAccount,
            BaseStateWithExtensionsMut,
            PodStateWithExtensionsMut,
        },
        pod::PodAccount,
    },
    token_interface::{Mint, TokenAccount},
};

use crate::state::VaultConfig;
use crate::errors::VaultError;

#[derive(Accounts)]
pub struct TransferHook<'info> {
    #[account(
        token::mint = mint,
        token::authority = owner,
    )]
    pub source_token: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        token::mint = mint,
    )]
    pub destination_token: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: source token account owner, can be SystemAccount or PDA
    pub owner: UncheckedAccount<'info>,

    /// CHECK: ExtraAccountMetaList Account
    #[account(
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump,
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    #[account(
        seeds = [b"vault_config"],
        bump = vault_config.config_bump,
    )]
    pub vault_config: Account<'info, VaultConfig>,
}

impl<'info> TransferHook<'info> {
    pub fn transfer_hook(&mut self, amount: u64) -> Result<()> {
        self.check_is_transferring()?;

        // If owner is the vault_config PDA, this is a program-controlled withdrawal — allow it
        if self.owner.key() == self.vault_config.key() {
            msg!("Transfer allowed: program-controlled transfer");
            return Ok(());
        }

        // Check if the owner is in the whitelist
        let owner_key = self.owner.key();
        let entry = self.vault_config.whitelist.iter()
            .find(|e| e.address == owner_key)
            .ok_or(VaultError::NotWhitelisted)?;

        // Check amount limit
        require!(
            amount <= entry.amount,
            VaultError::AmountExceedsLimit
        );

        msg!("Transfer allowed: {} is whitelisted (limit: {}, amount: {})",
            owner_key, entry.amount, amount);

        Ok(())
    }

    fn check_is_transferring(&mut self) -> Result<()> {
        let source_token_info = self.source_token.to_account_info();
        let mut account_data_ref: RefMut<&mut [u8]> =
            source_token_info.try_borrow_mut_data()?;

        let mut account =
            PodStateWithExtensionsMut::<PodAccount>::unpack(*account_data_ref)?;
        let account_extension =
            account.get_extension_mut::<TransferHookAccount>()?;

        if !bool::from(account_extension.transferring) {
            return Err(VaultError::NotTransferring.into());
        }

        Ok(())
    }
}
