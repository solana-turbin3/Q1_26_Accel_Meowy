use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, Burn, MintTo};

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

    #[account(mut)]
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
        // Validate whitelist internally
        let depositor_key = self.depositor.key();
        let entry = self.vault_config.whitelist.iter()
            .find(|e| e.address == depositor_key)
            .ok_or(VaultError::NotWhitelisted)?;

        require!(
            amount <= entry.amount,
            VaultError::AmountExceedsLimit
        );

        // Burn tokens from depositor (avoids transfer_checked reentrancy)
        let burn_accounts = Burn {
            mint: self.mint.to_account_info(),
            from: self.depositor_token_account.to_account_info(),
            authority: self.depositor.to_account_info(),
        };
        let burn_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            burn_accounts,
        );
        token_interface::burn(burn_ctx, amount)?;

        // Mint equivalent tokens to vault (vault_config PDA is mint authority)
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"vault_config",
            &[self.vault_config.config_bump],
        ]];

        let mint_accounts = MintTo {
            mint: self.mint.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.vault_config.to_account_info(),
        };
        let mint_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            mint_accounts,
            signer_seeds,
        );
        token_interface::mint_to(mint_ctx, amount)?;

        msg!("Deposited {} tokens into vault", amount);
        Ok(())
    }
}
