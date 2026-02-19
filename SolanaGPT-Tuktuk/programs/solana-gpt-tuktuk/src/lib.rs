#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;

mod state;
mod instructions;
mod oracle;

use instructions::*;

declare_id!("CpS3rNPN8bB8fW8EuBNQ2p6my2Lbh6ZTpoi9SuhTqKoE");

#[program]
pub mod solana_gpt_tuktuk {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        system_prompt: String,
        query_prompt: String,
    ) -> Result<()> {
        ctx.accounts.initialize(system_prompt, query_prompt, &ctx.bumps)
    }

    pub fn ask_gpt(ctx: Context<AskGpt>) -> Result<()> {
        ctx.accounts.ask_gpt()
    }

    pub fn callback_gpt(ctx: Context<CallbackGpt>, response: String) -> Result<()> {
        ctx.accounts.callback_gpt(response)
    }

    pub fn schedule_ask(
        ctx: Context<ScheduleAsk>,
        task_id: u16,
        trigger_timestamp: i64,
    ) -> Result<()> {
        ctx.accounts.schedule_ask(task_id, trigger_timestamp, &ctx.bumps)
    }
}

#[error_code]
pub enum GptTuktukError {
    #[msg("Invalid oracle identity — callback must come from the GPT oracle")]
    InvalidOracleIdentity,
    #[msg("Failed to compile transaction for TukTuk scheduling")]
    CompileTransactionFailed,
}
