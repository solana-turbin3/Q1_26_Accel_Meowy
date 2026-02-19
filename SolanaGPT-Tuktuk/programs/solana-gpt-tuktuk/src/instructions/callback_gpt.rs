use anchor_lang::prelude::*;

use crate::oracle;
use crate::state::Agent;

#[derive(Accounts)]
pub struct CallbackGpt<'info> {
    /// CHECK: Oracle Identity PDA — must be signer, verified below
    pub identity: AccountInfo<'info>,

    #[account(mut)]
    pub agent: Account<'info, Agent>,
}

impl<'info> CallbackGpt<'info> {
    pub fn callback_gpt(&mut self, response: String) -> Result<()> {
        // Verify the caller is the oracle's Identity PDA
        let (expected_identity, _) = Pubkey::find_program_address(
            &[oracle::IDENTITY_SEED],
            &oracle::ORACLE_PROGRAM_ID,
        );
        require_keys_eq!(
            self.identity.key(),
            expected_identity,
            crate::GptTuktukError::InvalidOracleIdentity
        );
        require!(
            self.identity.is_signer,
            crate::GptTuktukError::InvalidOracleIdentity
        );

        // Store response (truncate if too long)
        self.agent.last_response = if response.len() > Agent::MAX_RESPONSE_LEN {
            response[..Agent::MAX_RESPONSE_LEN].to_string()
        } else {
            response.clone()
        };

        msg!("GPT Response: {}", response);
        Ok(())
    }
}
