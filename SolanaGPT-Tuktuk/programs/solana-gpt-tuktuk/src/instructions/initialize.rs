use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;

use crate::oracle;
use crate::state::Agent;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        init,
        payer = maker,
        space = Agent::SPACE,
        seeds = [b"agent", maker.key().as_ref()],
        bump,
    )]
    pub agent: Account<'info, Agent>,

    /// CHECK: Oracle context account — created by oracle CPI
    #[account(mut)]
    pub llm_context: AccountInfo<'info>,

    /// CHECK: Oracle counter PDA
    #[account(mut)]
    pub counter: AccountInfo<'info>,

    /// CHECK: Oracle program
    #[account(address = oracle::ORACLE_PROGRAM_ID)]
    pub oracle_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(
        &mut self,
        system_prompt: String,
        query_prompt: String,
        bumps: &InitializeBumps,
    ) -> Result<()> {
        // Store agent state
        self.agent.maker = self.maker.key();
        self.agent.context = self.llm_context.key();
        self.agent.prompt = if query_prompt.len() > Agent::MAX_PROMPT_LEN {
            query_prompt[..Agent::MAX_PROMPT_LEN].to_string()
        } else {
            query_prompt
        };
        self.agent.last_response = String::new();
        self.agent.bump = bumps.agent;

        // CPI to oracle: create_llm_context
        let ix = oracle::create_llm_context_ix(
            self.maker.key(),
            self.llm_context.key(),
            self.counter.key(),
            system_prompt,
        );
        invoke(
            &ix,
            &[
                self.maker.to_account_info(),
                self.counter.to_account_info(),
                self.llm_context.to_account_info(),
                self.system_program.to_account_info(),
                self.oracle_program.to_account_info(),
            ],
        )?;

        // Fund the agent PDA with SOL for future oracle interactions
        let fund_ix = anchor_lang::solana_program::system_instruction::transfer(
            &self.maker.key(),
            &self.agent.key(),
            50_000_000, // 0.05 SOL
        );
        invoke(
            &fund_ix,
            &[
                self.maker.to_account_info(),
                self.agent.to_account_info(),
                self.system_program.to_account_info(),
            ],
        )?;

        msg!("Agent initialized with context: {}", self.llm_context.key());
        Ok(())
    }
}
