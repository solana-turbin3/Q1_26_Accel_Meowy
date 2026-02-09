use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};

#[derive(Accounts)]
pub struct TokenFactory<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        mint::decimals = 9,
        mint::authority = user,
        mint::token_program = token_program,
        extensions::transfer_hook::authority = user,
        extensions::transfer_hook::program_id = transfer_hook_program,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    /// CHECK: The transfer hook program ID (this program itself)
    pub transfer_hook_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> TokenFactory<'info> {
    pub fn init_mint(&mut self, _bumps: &TokenFactoryBumps) -> Result<()> {
        Ok(())
    }
}
