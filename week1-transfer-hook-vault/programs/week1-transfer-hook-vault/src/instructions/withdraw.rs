use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, Burn, MintTo};

use crate::state::VaultConfig;
use crate::errors::VaultError;

#[derive(Accounts)]
#[instruction(amount: u64, user: Pubkey)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"vault_config"],
        bump = vault_config.config_bump,
        has_one = admin @ VaultError::Unauthorized,
        has_one = mint,
    )]
    pub vault_config: Account<'info, VaultConfig>,

    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"vault", vault_config.key().as_ref()],
        bump = vault_config.vault_bump,
        token::mint = mint,
        token::authority = vault_config,
        token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = mint,
        token::token_program = token_program,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"vault_config",
            &[self.vault_config.config_bump],
        ]];

        // Burn tokens from vault (vault_config PDA is authority)
        let burn_accounts = Burn {
            mint: self.mint.to_account_info(),
            from: self.vault.to_account_info(),
            authority: self.vault_config.to_account_info(),
        };
        let burn_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            burn_accounts,
            signer_seeds,
        );
        token_interface::burn(burn_ctx, amount)?;

        // Mint equivalent tokens to user
        let mint_accounts = MintTo {
            mint: self.mint.to_account_info(),
            to: self.user_token_account.to_account_info(),
            authority: self.vault_config.to_account_info(),
        };
        let mint_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            mint_accounts,
            signer_seeds,
        );
        token_interface::mint_to(mint_ctx, amount)?;

        msg!("Withdrew {} tokens from vault", amount);
        Ok(())
    }
}
