#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;

mod errors;
mod instructions;
mod state;

use instructions::*;

use spl_discriminator::SplDiscriminate;
use spl_transfer_hook_interface::instruction::ExecuteInstruction;
use spl_tlv_account_resolution::state::ExtraAccountMetaList;

declare_id!("75rznRBCfaY7do322oxyeEpcDf73xskqx8D7rTkYE66c");

#[program]
pub mod week1_transfer_hook_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)
    }

    pub fn add_to_whitelist(ctx: Context<AddToWhitelist>, user: Pubkey, amount: u64) -> Result<()> {
        ctx.accounts.add_to_whitelist(user, amount)
    }

    pub fn remove_from_whitelist(ctx: Context<RemoveFromWhitelist>, user: Pubkey) -> Result<()> {
        ctx.accounts.remove_from_whitelist(user)
    }

    pub fn mint_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
        ctx.accounts.mint_tokens(amount)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64, _user: Pubkey) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

    pub fn initialize_extra_account_metas(ctx: Context<InitializeExtraAccountMetas>) -> Result<()> {
        msg!("Initializing Extra Account Metas for Transfer Hook...");

        let extra_account_metas = InitializeExtraAccountMetas::extra_account_metas()?;

        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &extra_account_metas,
        )
        .unwrap();

        Ok(())
    }

    #[instruction(discriminator = ExecuteInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        ctx.accounts.transfer_hook(amount)
    }
}
