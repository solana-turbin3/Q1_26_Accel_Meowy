#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;

mod instructions;
mod state;

use instructions::*;

use spl_discriminator::SplDiscriminate;
use spl_transfer_hook_interface::instruction::ExecuteInstruction;
use spl_tlv_account_resolution::state::ExtraAccountMetaList;

declare_id!("DhzyDgCmmQzVC4vEcj2zRGUyN8Mt5JynfdGLKkBcRGaX");

#[program]
pub mod whitelist_transfer_hook {
    use super::*;

    pub fn add_to_whitelist(ctx: Context<AddToWhitelist>, _user: Pubkey) -> Result<()> {
        ctx.accounts.add_to_whitelist(&ctx.bumps)
    }

    pub fn remove_from_whitelist(ctx: Context<RemoveFromWhitelist>, _user: Pubkey) -> Result<()> {
        ctx.accounts.remove_from_whitelist()
    }

    pub fn init_mint(ctx: Context<TokenFactory>) -> Result<()> {
        ctx.accounts.init_mint(&ctx.bumps)
    }

    pub fn initialize_transfer_hook(ctx: Context<InitializeExtraAccountMetaList>) -> Result<()> {
        msg!("Initializing Transfer Hook...");

        let extra_account_metas = InitializeExtraAccountMetaList::extra_account_metas()?;

        msg!("Extra Account Metas: {:?}", extra_account_metas);
        msg!("Extra Account Metas Length: {}", extra_account_metas.len());

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
