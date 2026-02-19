use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, close_account,
    Mint, TokenAccount, TokenInterface,
    TransferChecked, CloseAccount,
};

use crate::state::Escrow;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct AutoRefund<'info> {
    /// CHECK: We verify this matches escrow.maker after deserialization
    #[account(mut)]
    pub maker: UncheckedAccount<'info>,

    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: Escrow PDA — we manually deserialize after checking if account is empty
    #[account(
        mut,
        seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub escrow: AccountInfo<'info>,

    /// CHECK: Vault ATA of escrow for mint_a — may already be closed
    #[account(mut)]
    pub vault: AccountInfo<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> AutoRefund<'info> {
    pub fn auto_refund_and_close_vault(
        &mut self,
        seed: u64,
        escrow_bump: u8,
    ) -> Result<()> {
        // Graceful no-op if escrow is already closed (taken or manually refunded)
        if self.escrow.data_is_empty() || self.escrow.lamports() == 0 {
            msg!("AutoRefund: escrow already closed, nothing to do.");
            return Ok(());
        }

        // Deserialize and validate the escrow
        let escrow_data = Escrow::try_deserialize(
            &mut self.escrow.data.borrow().as_ref(),
        )?;
        require_keys_eq!(
            escrow_data.maker,
            self.maker.key(),
            crate::EscrowError::InvalidMaker
        );
        require_keys_eq!(escrow_data.mint_a, self.mint_a.key());

        // Graceful no-op if vault is already closed
        if self.vault.data_is_empty() || self.vault.lamports() == 0 {
            msg!("AutoRefund: vault already closed, closing escrow state.");
            close_account_info(&self.escrow, &self.maker.to_account_info())?;
            return Ok(());
        }

        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            self.maker.key.as_ref(),
            &seed.to_le_bytes()[..],
            &[escrow_bump],
        ]];

        // Read vault amount from raw account data (amount is at offset 64 in SPL token account)
        let vault_data = self.vault.try_borrow_data()?;
        let vault_amount = u64::from_le_bytes(
            vault_data[64..72].try_into().unwrap()
        );
        drop(vault_data);

        let cpi_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.maker_ata_a.to_account_info(),
            mint: self.mint_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            &signer_seeds,
        );
        transfer_checked(cpi_ctx, vault_amount, self.mint_a.decimals)?;

        // Close the vault, sending lamports to maker
        let cpi_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            &signer_seeds,
        );
        close_account(cpi_ctx)?;

        // Close the escrow account, returning rent to maker
        close_account_info(&self.escrow, &self.maker.to_account_info())?;

        msg!("AutoRefund: escrow refunded successfully");
        Ok(())
    }
}

/// Manually close an AccountInfo by transferring lamports and zeroing data.
fn close_account_info(account: &AccountInfo, destination: &AccountInfo) -> Result<()> {
    let dest_starting_lamports = destination.lamports();
    **destination.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(account.lamports())
        .unwrap();
    **account.lamports.borrow_mut() = 0;

    let mut data = account.try_borrow_mut_data()?;
    for byte in data.iter_mut() {
        *byte = 0;
    }

    Ok(())
}
