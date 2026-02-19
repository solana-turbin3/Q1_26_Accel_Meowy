use anchor_lang::prelude::*;

use crate::state::UserAccount;

#[derive(Accounts)]
pub struct CallbackVrfEr<'info> {
    #[account(address = ephemeral_vrf_sdk::consts::VRF_PROGRAM_IDENTITY)]
    pub vrf_program_identity: Signer<'info>,

    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
}

impl<'info> CallbackVrfEr<'info> {
    pub fn callback_vrf_er(&mut self, randomness: [u8; 32]) -> Result<()> {
        let random_value = ephemeral_vrf_sdk::rnd::random_u64(&randomness);
        msg!("VRF callback (ER): random value = {}", random_value);
        self.user_account.data = random_value;
        Ok(())
    }
}
