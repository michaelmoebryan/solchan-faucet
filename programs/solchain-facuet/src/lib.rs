use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod solchain_faucet {
    use super::*;

    pub fn initialize_faucet(ctx: Context<InitializeFaucet>) -> Result<()> {
        let faucet = &mut ctx.accounts.faucet;
        faucet.last_request_time = 0;
        faucet.admin = *ctx.accounts.admin.key;
        Ok(())
    }

    pub fn request_funds(ctx: Context<RequestFunds>) -> Result<()> {
        let faucet = &mut ctx.accounts.faucet;
        let user = &mut ctx.accounts.user;

        let clock = Clock::get()?;
        if clock.unix_timestamp - faucet.last_request_time < 30 {
            return Err(ErrorCode::RequestTooSoon.into());
        }

        faucet.last_request_time = clock.unix_timestamp;

        let lamports = 10_000_000_000; // 10 SOL in lamports

        **faucet.to_account_info().try_borrow_mut_lamports()? -= lamports;
        **user.to_account_info().try_borrow_mut_lamports()? += lamports;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeFaucet<'info> {
    #[account(init, payer = admin, space = 8 + 8 + 32)]
    pub faucet: Account<'info, Faucet>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RequestFunds<'info> {
    #[account(mut, has_one = admin)]
    pub faucet: Account<'info, Faucet>,
    /// CHECK: This is not dangerous as were just giving away funds
    #[account(mut)]
    pub user: AccountInfo<'info>,
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Faucet {
    pub last_request_time: i64,
    pub admin: Pubkey,
}

#[error_code]
pub enum ErrorCode {
    #[msg("You need to wait 30 seconds before requesting funds again.")]
    RequestTooSoon,
}
