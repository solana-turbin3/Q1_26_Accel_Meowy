#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;

mod state;
mod instructions;
mod tests;

use instructions::*;

declare_id!("FircrADQ2wgGuvpm8qneNCfKM7o5zoHTWnDQxngpTQ3J");

#[program]
pub mod anchor_escrow {
    use super::*;

    pub fn make(ctx: Context<Make>, seed: u64, deposit: u64, receive: u64) -> Result<()> {
        ctx.accounts.init_escrow(seed, receive, &ctx.bumps)?;
        ctx.accounts.deposit(deposit)
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund_and_close_vault()
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.check_time_lock()?;
        ctx.accounts.deposit()?;
        ctx.accounts.withdraw_and_close_vault()
    }

    pub fn auto_refund(ctx: Context<AutoRefund>, seed: u64) -> Result<()> {
        ctx.accounts.auto_refund_and_close_vault(seed, ctx.bumps.escrow)
    }

    pub fn schedule_refund(
        ctx: Context<ScheduleRefund>,
        seed: u64,
        task_id: u16,
        expiry_timestamp: i64,
    ) -> Result<()> {
        ctx.accounts.schedule_refund(seed, task_id, expiry_timestamp, &ctx.bumps)
    }
}

#[error_code]
pub enum EscrowError {
    #[msg("The escrow time lock has not expired. Take can only happen 5 days after make.")]
    TimeLockNotExpired,
    #[msg("Escrow maker does not match the provided maker account.")]
    InvalidMaker,
    #[msg("Failed to compile auto_refund transaction for TukTuk.")]
    CompileTransactionFailed,
}