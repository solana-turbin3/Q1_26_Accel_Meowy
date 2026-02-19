use anchor_lang::{prelude::*, InstructionData, ToAccountMetas};
use anchor_lang::solana_program::instruction::Instruction;
use anchor_spl::{
    associated_token,
    token_interface::TokenInterface,
};
use tuktuk_program::{
    TransactionSourceV0, compile_transaction,
    tuktuk::{
        cpi::{accounts::QueueTaskV0, queue_task_v0},
        program::Tuktuk,
        types::TriggerV0,
    },
    types::QueueTaskArgsV0,
};

use crate::state::Escrow;

#[derive(Accounts)]
#[instruction(seed: u64, task_id: u16, expiry_timestamp: i64)]
pub struct ScheduleRefund<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
        has_one = maker,
    )]
    pub escrow: Account<'info, Escrow>,

    pub token_program: Interface<'info, TokenInterface>,

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

impl<'info> ScheduleRefund<'info> {
    pub fn schedule_refund(
        &mut self,
        seed: u64,
        task_id: u16,
        expiry_timestamp: i64,
        bumps: &ScheduleRefundBumps,
    ) -> Result<()> {
        let maker_key = self.maker.key();
        let mint_a = self.escrow.mint_a;
        let token_program_key = self.token_program.key();

        // Derive all accounts that auto_refund will need
        let (escrow_pda, _) = Pubkey::find_program_address(
            &[b"escrow", maker_key.as_ref(), &seed.to_le_bytes()],
            &crate::ID,
        );

        let maker_ata_a = associated_token::get_associated_token_address_with_program_id(
            &maker_key,
            &mint_a,
            &token_program_key,
        );

        let vault = associated_token::get_associated_token_address_with_program_id(
            &escrow_pda,
            &mint_a,
            &token_program_key,
        );

        // Build the auto_refund instruction for TukTuk to execute later
        let auto_refund_ix = Instruction {
            program_id: crate::ID,
            accounts: crate::accounts::AutoRefund {
                maker: maker_key,
                mint_a,
                maker_ata_a,
                escrow: escrow_pda,
                vault,
                token_program: token_program_key,
                system_program: anchor_lang::system_program::ID,
            }
            .to_account_metas(None),
            data: crate::instruction::AutoRefund { seed }.data(),
        };

        // Compile to TukTuk's transaction format
        let (compiled_tx, _) = compile_transaction(vec![auto_refund_ix], vec![])
            .map_err(|_| error!(crate::EscrowError::CompileTransactionFailed))?;

        // CPI into TukTuk to register the task
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
                trigger: TriggerV0::Timestamp(expiry_timestamp),
                transaction: TransactionSourceV0::CompiledV0(compiled_tx),
                crank_reward: Some(1_000_001),
                free_tasks: 1,
                id: task_id,
                description: format!("auto-refund-{}", seed),
            },
        )?;

        msg!(
            "Scheduled auto-refund for escrow at timestamp {}",
            expiry_timestamp
        );

        Ok(())
    }
}
