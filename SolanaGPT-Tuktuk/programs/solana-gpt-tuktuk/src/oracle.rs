use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    hash::hash as sha256,
    instruction::{AccountMeta, Instruction},
};

pub const ORACLE_PROGRAM_ID: Pubkey = pubkey!("LLMrieZMpbJFwN52WgmBNMxYojrpRVYXdC1RCweEbab");

pub const IDENTITY_SEED: &[u8] = b"identity";
#[allow(dead_code)]
pub const COUNTER_SEED: &[u8] = b"counter";
#[allow(dead_code)]
pub const CONTEXT_SEED: &[u8] = b"test-context";
pub const INTERACTION_SEED: &[u8] = b"interaction";

/// Compute Anchor instruction discriminator: sha256("global:<name>")[..8]
fn sighash(name: &str) -> [u8; 8] {
    let preimage = format!("global:{}", name);
    let hash = sha256(preimage.as_bytes());
    let mut disc = [0u8; 8];
    disc.copy_from_slice(&hash.to_bytes()[..8]);
    disc
}

/// Build the oracle's `create_llm_context` CPI instruction.
pub fn create_llm_context_ix(
    payer: Pubkey,
    context_account: Pubkey,
    counter: Pubkey,
    text: String,
) -> Instruction {
    let disc = sighash("create_llm_context");
    let mut data = disc.to_vec();
    // Borsh: String = u32 LE length + bytes
    data.extend_from_slice(&(text.len() as u32).to_le_bytes());
    data.extend_from_slice(text.as_bytes());

    Instruction {
        program_id: ORACLE_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(payer, true),
            AccountMeta::new(counter, false),
            AccountMeta::new(context_account, false),
            AccountMeta::new_readonly(anchor_lang::system_program::ID, false),
        ],
        data,
    }
}

/// Build the oracle's `interact_with_llm` CPI instruction.
pub fn interact_with_llm_ix(
    payer: Pubkey,
    interaction: Pubkey,
    context_account: Pubkey,
    text: String,
    callback_program_id: Pubkey,
    callback_discriminator: [u8; 8],
    account_metas: Option<Vec<OracleAccountMeta>>,
) -> Instruction {
    let disc = sighash("interact_with_llm");
    let mut data = disc.to_vec();

    // String
    data.extend_from_slice(&(text.len() as u32).to_le_bytes());
    data.extend_from_slice(text.as_bytes());

    // Pubkey (32 bytes)
    data.extend_from_slice(&callback_program_id.to_bytes());

    // [u8; 8]
    data.extend_from_slice(&callback_discriminator);

    // Option<Vec<AccountMeta>>
    match &account_metas {
        None => data.push(0),
        Some(metas) => {
            data.push(1);
            data.extend_from_slice(&(metas.len() as u32).to_le_bytes());
            for meta in metas {
                data.extend_from_slice(&meta.pubkey.to_bytes());
                data.push(meta.is_signer as u8);
                data.push(meta.is_writable as u8);
            }
        }
    }

    Instruction {
        program_id: ORACLE_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(payer, true),
            AccountMeta::new(interaction, false),
            AccountMeta::new_readonly(context_account, false),
            AccountMeta::new_readonly(anchor_lang::system_program::ID, false),
        ],
        data,
    }
}

pub struct OracleAccountMeta {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}
