use anchor_lang::prelude::*;

#[account]
pub struct Agent {
    pub maker: Pubkey,
    pub context: Pubkey,
    pub prompt: String,
    pub last_response: String,
    pub bump: u8,
}

impl Agent {
    pub const MAX_PROMPT_LEN: usize = 256;
    pub const MAX_RESPONSE_LEN: usize = 512;
    // 8 (discriminator) + 32 (maker) + 32 (context) + (4 + 256) (prompt) + (4 + 512) (response) + 1 (bump)
    pub const SPACE: usize = 8 + 32 + 32 + (4 + Self::MAX_PROMPT_LEN) + (4 + Self::MAX_RESPONSE_LEN) + 1;
}
