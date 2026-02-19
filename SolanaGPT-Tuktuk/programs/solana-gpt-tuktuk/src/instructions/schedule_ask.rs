use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::InstructionData;
use tuktuk_program::{
    TransactionSourceV0, compile_transaction,
    tuktuk::{
        cpi::{accounts::QueueTaskV0, queue_task_v0},
        program::Tuktuk,
        types::TriggerV0,
    },
    types::QueueTaskArgsV0,
};

use crate::oracle;
use crate::state::Agent;

#[derive(Accounts)]
pub struct ScheduleAsk<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        seeds = [b"agent", maker.key().as_ref()],
        bump = agent.bump,
        has_one = maker,
    )]
    pub agent: Account<'info, Agent>,

    /// CHECK: TukTuk task queue (pre-created off-chain)
    #[account(mut)]
    pub task_queue: UncheckedAccount<'info>,

    /// CHECK: TukTuk task queue authority PDA
    pub task_queue_authority: UncheckedAccount<'info>,

    /// CHECK: TukTuk task account (initialized by CPI)
    #[account(mut)]
    pub task: UncheckedAccount<'info>,

    /// CHECK: Our program's PDA that signs TukTuk CPI
    #[account(
        mut,
        seeds = [b"queue_authority"],
        bump,
    )]
    pub queue_authority: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub tuktuk_program: Program<'info, Tuktuk>,
}

impl<'info> ScheduleAsk<'info> {
    pub fn schedule_ask(
        &self,
        task_id: u16,
        trigger_timestamp: i64,
        bumps: &ScheduleAskBumps,
    ) -> Result<()> {
        let agent_pda = self.agent.key();
        let context_pda = self.agent.context;

        // Derive the interaction PDA on oracle program
        let (interaction_pda, _) = Pubkey::find_program_address(
            &[
                oracle::INTERACTION_SEED,
                agent_pda.as_ref(),
                context_pda.as_ref(),
            ],
            &oracle::ORACLE_PROGRAM_ID,
        );

        // Build the ask_gpt instruction for TukTuk to execute later
        let ask_gpt_ix = Instruction {
            program_id: crate::ID,
            accounts: vec![
                anchor_lang::solana_program::instruction::AccountMeta::new(agent_pda, false),
                anchor_lang::solana_program::instruction::AccountMeta::new(interaction_pda, false),
                anchor_lang::solana_program::instruction::AccountMeta::new_readonly(context_pda, false),
                anchor_lang::solana_program::instruction::AccountMeta::new_readonly(oracle::ORACLE_PROGRAM_ID, false),
                anchor_lang::solana_program::instruction::AccountMeta::new_readonly(anchor_lang::system_program::ID, false),
            ],
            data: crate::instruction::AskGpt {}.data(),
        };

        // Compile to TukTuk's transaction format
        let (compiled_tx, _) = compile_transaction(vec![ask_gpt_ix], vec![])
            .map_err(|_| error!(crate::GptTuktukError::CompileTransactionFailed))?;

        // CPI into TukTuk to register the scheduled task
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"queue_authority",
            &[bumps.queue_authority],
        ]];

        let cpi_accounts = QueueTaskV0 {
            payer: self.maker.to_account_info(),
            queue_authority: self.queue_authority.to_account_info(),
            task_queue: self.task_queue.to_account_info(),
            task_queue_authority: self.task_queue_authority.to_account_info(),
            task: self.task.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.tuktuk_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );

        queue_task_v0(
            cpi_ctx,
            QueueTaskArgsV0 {
                trigger: TriggerV0::Timestamp(trigger_timestamp),
                transaction: TransactionSourceV0::CompiledV0(compiled_tx),
                crank_reward: Some(1_000_001),
                free_tasks: 1,
                id: task_id,
                description: format!("ask-gpt-{}", task_id),
            },
        )?;

        msg!(
            "Scheduled GPT query at timestamp {}",
            trigger_timestamp
        );

        Ok(())
    }
}
