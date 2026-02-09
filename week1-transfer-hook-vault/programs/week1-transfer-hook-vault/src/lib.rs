use anchor_lang::prelude::*;

declare_id!("75rznRBCfaY7do322oxyeEpcDf73xskqx8D7rTkYE66c");

#[program]
pub mod week1_transfer_hook_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
