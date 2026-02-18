use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, MintTo};

use crate::state::VaultConfig;
use crate::errors::VaultError;

#[derive(Accounts)]
pub struct MintTokens<'info> {
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
        token::mint = mint,
        token::token_program = token_program,
    )]
    pub destination: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> MintTokens<'info> {
    pub fn mint_tokens(&mut self, amount: u64) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"vault_config",
            &[self.vault_config.config_bump],
        ]];

        let cpi_accounts = MintTo {
            mint: self.mint.to_account_info(),
            to: self.destination.to_account_info(),
            authority: self.vault_config.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );

        token_interface::mint_to(cpi_ctx, amount)?;

        msg!("Minted {} tokens to {}", amount, self.destination.key());
        Ok(())
    }
}
