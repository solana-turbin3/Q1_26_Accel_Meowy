use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::Discriminator;

use crate::oracle::{self, OracleAccountMeta};
use crate::state::Agent;

#[derive(Accounts)]
pub struct AskGpt<'info> {
    #[account(
        mut,
        seeds = [b"agent", agent.maker.as_ref()],
        bump = agent.bump,
    )]
    pub agent: Account<'info, Agent>,

    /// CHECK: Interaction PDA on oracle program
    #[account(mut)]
    pub interaction: AccountInfo<'info>,

    /// CHECK: Oracle context account
    #[account(address = agent.context)]
    pub context_account: AccountInfo<'info>,

    /// CHECK: Oracle program
    #[account(address = oracle::ORACLE_PROGRAM_ID)]
    pub oracle_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> AskGpt<'info> {
    pub fn ask_gpt(&self) -> Result<()> {
        // Get our callback_gpt discriminator
        let callback_disc: [u8; 8] = crate::instruction::CallbackGpt::DISCRIMINATOR
            .try_into()
            .expect("discriminator must be 8 bytes");

        // Build oracle interact_with_llm instruction
        let ix = oracle::interact_with_llm_ix(
            self.agent.key(),
            self.interaction.key(),
            self.context_account.key(),
            self.agent.prompt.clone(),
            crate::ID,
            callback_disc,
            Some(vec![OracleAccountMeta {
                pubkey: self.agent.key(),
                is_signer: false,
                is_writable: true,
            }]),
        );

        // Sign as Agent PDA
        let maker_key = self.agent.maker;
        let signer_seeds: &[&[u8]] = &[
            b"agent",
            maker_key.as_ref(),
            &[self.agent.bump],
        ];

        invoke_signed(
            &ix,
            &[
                self.agent.to_account_info(),
                self.interaction.to_account_info(),
                self.context_account.to_account_info(),
                self.system_program.to_account_info(),
                self.oracle_program.to_account_info(),
            ],
            &[signer_seeds],
        )?;

        msg!("Asked GPT: {}", self.agent.prompt);
        Ok(())
    }
}
