#![allow(unexpected_cfgs)]

#[cfg(test)]
mod tests {
    use litesvm::LiteSVM;
    use sha2::{Digest, Sha256};
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_sdk_ids::system_program;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
    use spl_token_2022::{
        extension::StateWithExtensions,
        state::Account as TokenAccount,
    };
    use std::path::PathBuf;

    // Program ID from declare_id! in the program: "75rznRBCfaY7do322oxyeEpcDf73xskqx8D7rTkYE66c"
    fn program_id() -> Pubkey {
        let bytes = bs58::decode("75rznRBCfaY7do322oxyeEpcDf73xskqx8D7rTkYE66c")
            .into_vec()
            .unwrap();
        Pubkey::new_from_array(bytes.try_into().unwrap())
    }

    // Token-2022 program ID
    fn token_2022_program_id() -> Pubkey {
        spl_token_2022::ID
    }

    /// Compute the Anchor discriminator for an instruction: sha256("global:<name>")[..8]
    fn anchor_discriminator(name: &str) -> [u8; 8] {
        let mut hasher = Sha256::new();
        hasher.update(format!("global:{}", name).as_bytes());
        let result = hasher.finalize();
        let mut disc = [0u8; 8];
        disc.copy_from_slice(&result[..8]);
        disc
    }

    /// Compute the Anchor account discriminator: sha256("account:<name>")[..8]
    fn _account_discriminator(name: &str) -> [u8; 8] {
        let mut hasher = Sha256::new();
        hasher.update(format!("account:{}", name).as_bytes());
        let result = hasher.finalize();
        let mut disc = [0u8; 8];
        disc.copy_from_slice(&result[..8]);
        disc
    }

    // PDA helpers
    fn vault_config_pda() -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"vault_config"], &program_id())
    }

    fn vault_pda(vault_config: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"vault", vault_config.as_ref()],
            &program_id(),
        )
    }

    fn extra_account_meta_list_pda(mint: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"extra-account-metas", mint.as_ref()],
            &program_id(),
        )
    }

    fn get_associated_token_address_2022(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
        spl_associated_token_account::get_associated_token_address_with_program_id(
            owner,
            mint,
            &token_2022_program_id(),
        )
    }

    fn setup() -> (LiteSVM, Keypair) {
        let mut program = LiteSVM::new();
        let payer = Keypair::new();

        program
            .airdrop(&payer.pubkey(), 100 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to payer");

        // Load the compiled program
        let so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../target/deploy/week1_transfer_hook_vault.so");

        let program_data = std::fs::read(&so_path)
            .unwrap_or_else(|_| panic!("Failed to read program SO file at {:?}. Run `anchor build` first.", so_path));

        program.add_program(program_id(), &program_data);

        (program, payer)
    }

    /// Build the initialize instruction
    fn build_initialize_ix(
        admin: &Pubkey,
        vault_config: &Pubkey,
        mint: &Pubkey,
        vault: &Pubkey,
    ) -> Instruction {
        let disc = anchor_discriminator("initialize");

        Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(*admin, true),           // admin (signer, writable)
                AccountMeta::new(*vault_config, false),    // vault_config (PDA, writable)
                AccountMeta::new(*mint, true),             // mint (signer, writable)
                AccountMeta::new(*vault, false),           // vault (PDA, writable)
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(token_2022_program_id(), false),
            ],
            data: disc.to_vec(),
        }
    }

    /// Build the initialize_extra_account_metas instruction
    fn build_init_extra_account_metas_ix(
        payer: &Pubkey,
        extra_account_meta_list: &Pubkey,
        mint: &Pubkey,
    ) -> Instruction {
        let disc = anchor_discriminator("initialize_extra_account_metas");

        Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(*payer, true),
                AccountMeta::new(*extra_account_meta_list, false),
                AccountMeta::new_readonly(*mint, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: disc.to_vec(),
        }
    }

    /// Build the add_to_whitelist instruction
    fn build_add_to_whitelist_ix(
        admin: &Pubkey,
        vault_config: &Pubkey,
        user: &Pubkey,
        amount: u64,
    ) -> Instruction {
        let disc = anchor_discriminator("add_to_whitelist");

        let mut data = disc.to_vec();
        data.extend_from_slice(user.as_ref()); // user: Pubkey (32 bytes)
        data.extend_from_slice(&amount.to_le_bytes()); // amount: u64 (8 bytes)

        Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(*admin, true),
                AccountMeta::new(*vault_config, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data,
        }
    }

    /// Build the remove_from_whitelist instruction
    fn build_remove_from_whitelist_ix(
        admin: &Pubkey,
        vault_config: &Pubkey,
        user: &Pubkey,
    ) -> Instruction {
        let disc = anchor_discriminator("remove_from_whitelist");

        let mut data = disc.to_vec();
        data.extend_from_slice(user.as_ref());

        Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(*admin, true),
                AccountMeta::new(*vault_config, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data,
        }
    }

    /// Build the mint_tokens instruction
    fn build_mint_tokens_ix(
        admin: &Pubkey,
        vault_config: &Pubkey,
        mint: &Pubkey,
        destination: &Pubkey,
        amount: u64,
    ) -> Instruction {
        let disc = anchor_discriminator("mint_tokens");

        let mut data = disc.to_vec();
        data.extend_from_slice(&amount.to_le_bytes());

        Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(*admin, true),
                AccountMeta::new_readonly(*vault_config, false),
                AccountMeta::new(*mint, false),
                AccountMeta::new(*destination, false),
                AccountMeta::new_readonly(token_2022_program_id(), false),
            ],
            data,
        }
    }

    /// Build the deposit instruction (uses burn+mint internally, validates whitelist in-program)
    fn build_deposit_ix(
        depositor: &Pubkey,
        vault_config: &Pubkey,
        mint: &Pubkey,
        depositor_token_account: &Pubkey,
        vault: &Pubkey,
        amount: u64,
    ) -> Instruction {
        let disc = anchor_discriminator("deposit");

        let mut data = disc.to_vec();
        data.extend_from_slice(&amount.to_le_bytes());

        Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(*depositor, true),
                AccountMeta::new_readonly(*vault_config, false),
                AccountMeta::new(*mint, false),              // mut: burn/mint modify supply
                AccountMeta::new(*depositor_token_account, false),
                AccountMeta::new(*vault, false),
                AccountMeta::new_readonly(token_2022_program_id(), false),
            ],
            data,
        }
    }

    /// Build the withdraw instruction (uses burn+mint internally)
    fn build_withdraw_ix(
        admin: &Pubkey,
        vault_config: &Pubkey,
        mint: &Pubkey,
        vault: &Pubkey,
        user_token_account: &Pubkey,
        amount: u64,
        user: &Pubkey,
    ) -> Instruction {
        let disc = anchor_discriminator("withdraw");

        let mut data = disc.to_vec();
        data.extend_from_slice(&amount.to_le_bytes());
        data.extend_from_slice(user.as_ref()); // _user: Pubkey

        Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(*admin, true),
                AccountMeta::new_readonly(*vault_config, false),
                AccountMeta::new(*mint, false),              // mut: burn/mint modify supply
                AccountMeta::new(*vault, false),
                AccountMeta::new(*user_token_account, false),
                AccountMeta::new_readonly(token_2022_program_id(), false),
            ],
            data,
        }
    }

    /// Helper: create an associated token account for Token-2022
    fn create_ata_ix(payer: &Pubkey, owner: &Pubkey, mint: &Pubkey) -> Instruction {
        spl_associated_token_account::instruction::create_associated_token_account(
            payer,
            owner,
            mint,
            &token_2022_program_id(),
        )
    }

    /// Execute a list of instructions as a transaction
    fn send_tx(
        program: &mut LiteSVM,
        ixs: &[Instruction],
        payer: &Keypair,
        extra_signers: &[&Keypair],
    ) -> Result<litesvm::types::TransactionMetadata, litesvm::types::FailedTransactionMetadata> {
        let message = Message::new(ixs, Some(&payer.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let mut all_signers: Vec<&Keypair> = vec![payer];
        all_signers.extend_from_slice(extra_signers);

        let signers_refs: Vec<&dyn Signer> = all_signers.iter().map(|k| *k as &dyn Signer).collect();

        let transaction = Transaction::new(&signers_refs, message, recent_blockhash);
        program.send_transaction(transaction)
    }

    /// Helper: get token balance from a Token-2022 account
    fn get_token_balance(program: &LiteSVM, token_account: &Pubkey) -> u64 {
        let account = program.get_account(token_account).expect("Token account not found");
        let state = StateWithExtensions::<TokenAccount>::unpack(&account.data)
            .expect("Failed to unpack token account");
        state.base.amount
    }

    /// Full setup: initialize, init extra account metas
    fn full_setup() -> (LiteSVM, Keypair, Pubkey, Pubkey, Pubkey) {
        let (mut program, admin) = setup();
        let mint_kp = Keypair::new();
        let mint = mint_kp.pubkey();

        let (vault_config, _) = vault_config_pda();
        let (vault, _) = vault_pda(&vault_config);
        let (extra_meta, _) = extra_account_meta_list_pda(&mint);

        // Initialize the vault
        let init_ix = build_initialize_ix(
            &admin.pubkey(),
            &vault_config,
            &mint,
            &vault,
        );

        send_tx(&mut program, &[init_ix], &admin, &[&mint_kp])
            .expect("Initialize failed");

        // Initialize extra account metas
        let init_meta_ix = build_init_extra_account_metas_ix(
            &admin.pubkey(),
            &extra_meta,
            &mint,
        );

        send_tx(&mut program, &[init_meta_ix], &admin, &[])
            .expect("Initialize extra account metas failed");

        (program, admin, vault_config, mint, vault)
    }

    // ========== TESTS ==========

    #[test]
    fn test_initialize() {
        let (program, _admin, vault_config, mint, vault) = full_setup();

        // Verify vault_config account exists
        let vc_account = program.get_account(&vault_config);
        assert!(vc_account.is_some(), "VaultConfig account should exist");

        let vc_data = vc_account.unwrap();
        assert_eq!(vc_data.owner, program_id(), "VaultConfig owner should be program");

        // Verify mint exists and has correct properties
        let mint_account = program.get_account(&mint);
        assert!(mint_account.is_some(), "Mint account should exist");

        let mint_data = mint_account.unwrap();
        assert_eq!(
            Pubkey::new_from_array(mint_data.owner.to_bytes()),
            token_2022_program_id(),
            "Mint owner should be Token-2022"
        );

        // Verify vault exists
        let vault_account = program.get_account(&vault);
        assert!(vault_account.is_some(), "Vault token account should exist");

        println!("test_initialize passed!");
        println!("  VaultConfig: {}", vault_config);
        println!("  Mint: {}", mint);
        println!("  Vault: {}", vault);
    }

    #[test]
    fn test_add_to_whitelist() {
        let (mut program, admin, vault_config, _mint, _vault) = full_setup();

        let user = Pubkey::new_unique();
        let amount: u64 = 1_000_000_000; // 1 token with 9 decimals

        let ix = build_add_to_whitelist_ix(
            &admin.pubkey(),
            &vault_config,
            &user,
            amount,
        );

        send_tx(&mut program, &[ix], &admin, &[])
            .expect("Add to whitelist failed");

        // Verify vault_config grew (account data should be larger)
        let vc_account = program.get_account(&vault_config).unwrap();
        // BASE_SIZE = 78, with 1 entry = 78 + 40 = 118
        assert!(vc_account.data.len() >= 118, "VaultConfig should have grown to fit 1 entry");

        println!("test_add_to_whitelist passed!");
    }

    #[test]
    fn test_remove_from_whitelist() {
        let (mut program, admin, vault_config, _mint, _vault) = full_setup();

        let user = Pubkey::new_unique();
        let amount: u64 = 1_000_000_000;

        // First add the user
        let add_ix = build_add_to_whitelist_ix(
            &admin.pubkey(),
            &vault_config,
            &user,
            amount,
        );
        send_tx(&mut program, &[add_ix], &admin, &[])
            .expect("Add to whitelist failed");

        // Then remove the user
        let remove_ix = build_remove_from_whitelist_ix(
            &admin.pubkey(),
            &vault_config,
            &user,
        );
        send_tx(&mut program, &[remove_ix], &admin, &[])
            .expect("Remove from whitelist failed");

        // Verify vault_config shrank back
        let vc_account = program.get_account(&vault_config).unwrap();
        // BASE_SIZE = 78 with 0 entries
        assert!(vc_account.data.len() <= 78, "VaultConfig should have shrunk back");

        println!("test_remove_from_whitelist passed!");
    }

    #[test]
    fn test_deposit() {
        let (mut program, admin, vault_config, mint, vault) = full_setup();

        // Create a depositor
        let depositor_kp = Keypair::new();
        program
            .airdrop(&depositor_kp.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop to depositor failed");

        // Add depositor to whitelist with sufficient limit
        let deposit_amount: u64 = 500_000_000; // 0.5 tokens
        let whitelist_limit: u64 = 1_000_000_000; // 1 token

        let add_wl_ix = build_add_to_whitelist_ix(
            &admin.pubkey(),
            &vault_config,
            &depositor_kp.pubkey(),
            whitelist_limit,
        );
        send_tx(&mut program, &[add_wl_ix], &admin, &[])
            .expect("Add depositor to whitelist failed");

        // Create depositor's ATA
        let depositor_ata = get_associated_token_address_2022(&depositor_kp.pubkey(), &mint);
        let create_ata_ix = create_ata_ix(&admin.pubkey(), &depositor_kp.pubkey(), &mint);
        send_tx(&mut program, &[create_ata_ix], &admin, &[])
            .expect("Create depositor ATA failed");

        // Mint tokens to depositor
        let mint_ix = build_mint_tokens_ix(
            &admin.pubkey(),
            &vault_config,
            &mint,
            &depositor_ata,
            1_000_000_000,
        );
        send_tx(&mut program, &[mint_ix], &admin, &[])
            .expect("Mint tokens failed");

        // Verify depositor has tokens
        let balance_before = get_token_balance(&program, &depositor_ata);
        assert_eq!(balance_before, 1_000_000_000, "Depositor should have 1 token");

        let vault_balance_before = get_token_balance(&program, &vault);
        assert_eq!(vault_balance_before, 0, "Vault should start empty");

        // Deposit tokens into vault
        let deposit_ix = build_deposit_ix(
            &depositor_kp.pubkey(),
            &vault_config,
            &mint,
            &depositor_ata,
            &vault,
            deposit_amount,
        );
        send_tx(&mut program, &[deposit_ix], &depositor_kp, &[])
            .expect("Deposit failed");

        // Verify balances after deposit
        let balance_after = get_token_balance(&program, &depositor_ata);
        assert_eq!(balance_after, 500_000_000, "Depositor should have 0.5 tokens left");

        let vault_balance_after = get_token_balance(&program, &vault);
        assert_eq!(vault_balance_after, 500_000_000, "Vault should have 0.5 tokens");

        println!("test_deposit passed!");
        println!("  Depositor balance: {} -> {}", balance_before, balance_after);
        println!("  Vault balance: {} -> {}", vault_balance_before, vault_balance_after);
    }

    #[test]
    fn test_deposit_unauthorized() {
        let (mut program, admin, vault_config, mint, vault) = full_setup();

        // Create a non-whitelisted user
        let user_kp = Keypair::new();
        program
            .airdrop(&user_kp.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        // Create user's ATA and mint tokens
        let user_ata = get_associated_token_address_2022(&user_kp.pubkey(), &mint);
        let create_ata_ix = create_ata_ix(&admin.pubkey(), &user_kp.pubkey(), &mint);
        send_tx(&mut program, &[create_ata_ix], &admin, &[])
            .expect("Create ATA failed");

        let mint_ix = build_mint_tokens_ix(
            &admin.pubkey(),
            &vault_config,
            &mint,
            &user_ata,
            1_000_000_000,
        );
        send_tx(&mut program, &[mint_ix], &admin, &[])
            .expect("Mint tokens failed");

        // Try to deposit without being whitelisted - should fail
        let deposit_ix = build_deposit_ix(
            &user_kp.pubkey(),
            &vault_config,
            &mint,
            &user_ata,
            &vault,
            500_000_000,
        );
        let result = send_tx(&mut program, &[deposit_ix], &user_kp, &[]);
        assert!(result.is_err(), "Deposit should fail for non-whitelisted user");

        println!("test_deposit_unauthorized passed! Non-whitelisted user correctly rejected.");
    }

    #[test]
    fn test_deposit_exceeds_limit() {
        let (mut program, admin, vault_config, mint, vault) = full_setup();

        // Create a depositor
        let depositor_kp = Keypair::new();
        program
            .airdrop(&depositor_kp.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        // Whitelist with a small limit
        let whitelist_limit: u64 = 100_000_000; // 0.1 tokens

        let add_wl_ix = build_add_to_whitelist_ix(
            &admin.pubkey(),
            &vault_config,
            &depositor_kp.pubkey(),
            whitelist_limit,
        );
        send_tx(&mut program, &[add_wl_ix], &admin, &[])
            .expect("Whitelist failed");

        // Create ATA and mint tokens
        let depositor_ata = get_associated_token_address_2022(&depositor_kp.pubkey(), &mint);
        let create_ata_ix = create_ata_ix(&admin.pubkey(), &depositor_kp.pubkey(), &mint);
        send_tx(&mut program, &[create_ata_ix], &admin, &[])
            .expect("Create ATA failed");

        let mint_ix = build_mint_tokens_ix(
            &admin.pubkey(),
            &vault_config,
            &mint,
            &depositor_ata,
            1_000_000_000,
        );
        send_tx(&mut program, &[mint_ix], &admin, &[])
            .expect("Mint failed");

        // Try to deposit more than the whitelist limit
        let deposit_ix = build_deposit_ix(
            &depositor_kp.pubkey(),
            &vault_config,
            &mint,
            &depositor_ata,
            &vault,
            500_000_000, // 0.5 tokens > 0.1 limit
        );
        let result = send_tx(&mut program, &[deposit_ix], &depositor_kp, &[]);
        assert!(result.is_err(), "Deposit should fail when exceeding whitelist limit");

        println!("test_deposit_exceeds_limit passed! Amount limit correctly enforced.");
    }

    #[test]
    fn test_withdraw() {
        let (mut program, admin, vault_config, mint, vault) = full_setup();

        // Create a user
        let user_kp = Keypair::new();
        program
            .airdrop(&user_kp.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        // Whitelist the user
        let add_wl_ix = build_add_to_whitelist_ix(
            &admin.pubkey(),
            &vault_config,
            &user_kp.pubkey(),
            1_000_000_000,
        );
        send_tx(&mut program, &[add_wl_ix], &admin, &[])
            .expect("Whitelist failed");

        // Create user's ATA and mint tokens
        let user_ata = get_associated_token_address_2022(&user_kp.pubkey(), &mint);
        let create_ata_ix = create_ata_ix(&admin.pubkey(), &user_kp.pubkey(), &mint);
        send_tx(&mut program, &[create_ata_ix], &admin, &[])
            .expect("Create ATA failed");

        let mint_ix = build_mint_tokens_ix(
            &admin.pubkey(),
            &vault_config,
            &mint,
            &user_ata,
            2_000_000_000,
        );
        send_tx(&mut program, &[mint_ix], &admin, &[])
            .expect("Mint failed");

        // User deposits into vault
        let deposit_ix = build_deposit_ix(
            &user_kp.pubkey(),
            &vault_config,
            &mint,
            &user_ata,
            &vault,
            1_000_000_000,
        );
        send_tx(&mut program, &[deposit_ix], &user_kp, &[])
            .expect("Deposit failed");

        // Verify vault has tokens
        let vault_balance = get_token_balance(&program, &vault);
        assert_eq!(vault_balance, 1_000_000_000, "Vault should have 1 token");

        // Admin withdraws from vault to user
        let withdraw_ix = build_withdraw_ix(
            &admin.pubkey(),
            &vault_config,
            &mint,
            &vault,
            &user_ata,
            500_000_000,
            &user_kp.pubkey(),
        );
        send_tx(&mut program, &[withdraw_ix], &admin, &[])
            .expect("Withdraw failed");

        // Verify balances
        let vault_after = get_token_balance(&program, &vault);
        assert_eq!(vault_after, 500_000_000, "Vault should have 0.5 tokens");

        let user_after = get_token_balance(&program, &user_ata);
        assert_eq!(user_after, 1_500_000_000, "User should have 1.5 tokens (1 remaining + 0.5 withdrawn)");

        println!("test_withdraw passed!");
        println!("  Vault balance: {} -> {}", vault_balance, vault_after);
        println!("  User balance: {} -> {}", 1_000_000_000u64, user_after);
    }
}
