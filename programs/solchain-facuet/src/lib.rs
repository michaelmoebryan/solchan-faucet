use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod solchan_faucet {
    use super::*;

    pub fn initialize_faucet(ctx: Context<InitializeFaucet>) -> Result<()> {
        let faucet = &mut ctx.accounts.faucet;
        faucet.last_request_time = 0;
        faucet.interval_period = 30; // Default interval period is 30 seconds
        faucet.admin = *ctx.accounts.admin.key;
        faucet.bump = *ctx.bumps.get("faucet").unwrap();

        Ok(())
    }

    pub fn request_funds(ctx: Context<RequestFunds>) -> Result<()> {
        let faucet = &mut ctx.accounts.faucet;
        let clock = Clock::get()?;
        if clock.unix_timestamp - faucet.last_request_time < faucet.interval_period {
            return Err(ErrorCode::RequestTooSoon.into());
        }

        faucet.last_request_time = clock.unix_timestamp;

        let lamports = 10_000_000_000; // 10 SOL in lamports

        **ctx.accounts.faucet.to_account_info().try_borrow_mut_lamports()? -= lamports;
        **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += lamports;

        Ok(())
    }

    pub fn set_interval_period(ctx: Context<SetIntervalPeriod>, interval: i64) -> Result<()> {
        let faucet = &mut ctx.accounts.faucet;
        if ctx.accounts.admin.key() != faucet.admin {
            return Err(ErrorCode::Unauthorized.into());
        }
        faucet.interval_period = interval;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeFaucet<'info> {
    #[account(
        init,
        payer = admin,
        seeds = [b"faucet"],
        bump,
        space = 56,
        has_one = admin)]
    pub faucet: Account<'info, Faucet>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RequestFunds<'info> {
    #[account(
        mut, 
        seeds = [b"faucet"],
        bump = faucet.bump,
        has_one = admin
    )]
    pub faucet: Account<'info, Faucet>,
    /// CHECK: This is not dangerous as we're just giving away funds
    #[account(mut)]
    pub user: AccountInfo<'info>,
    /// CHECK: not dangerous as only used to get the admina acc for sigcheck
    pub admin: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetIntervalPeriod<'info> {
    #[account(
        mut, 
        seeds = [b"faucet"],
        bump = faucet.bump,
        has_one = admin
    )]
    pub faucet: Account<'info, Faucet>,
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Faucet {
    pub last_request_time: i64,
    pub interval_period: i64,
    pub admin: Pubkey,
    pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("You need to wait before requesting funds again.")]
    RequestTooSoon,
    #[msg("Unauthorized access.")]
    Unauthorized,
}
