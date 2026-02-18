use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, Transfer};

use crate::state::VaultConfig;
use crate::errors::VaultError;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    #[account(
        seeds = [b"vault_config"],
        bump = vault_config.config_bump,
        has_one = mint,
    )]
    pub vault_config: Account<'info, VaultConfig>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = depositor,
        token::token_program = token_program,
    )]
    pub depositor_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault", vault_config.key().as_ref()],
        bump = vault_config.vault_bump,
        token::mint = mint,
        token::authority = vault_config,
        token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        // Validate whitelist internally (avoids reentrancy from transfer hook CPI)
        let depositor_key = self.depositor.key();
        let entry = self.vault_config.whitelist.iter()
            .find(|e| e.address == depositor_key)
            .ok_or(VaultError::NotWhitelisted)?;

        require!(
            amount <= entry.amount,
            VaultError::AmountExceedsLimit
        );

        // Use plain transfer (not transfer_checked) to avoid reentrancy
        // since our program is also the transfer hook program
        let cpi_accounts = Transfer {
            from: self.depositor_token_account.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.depositor.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            cpi_accounts,
        );

        token_interface::transfer(cpi_ctx, amount)?;

        msg!("Deposited {} tokens into vault", amount);
        Ok(())
    }
}
