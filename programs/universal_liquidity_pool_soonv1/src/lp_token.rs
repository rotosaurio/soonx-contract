use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Mint, TokenAccount, MintTo, Burn};

use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct MintLPTokens<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool".as_ref(), pool.token_mint.as_ref()],
        bump = pool.bump
    )]
    pub pool: Account<'info, LiquidityPool>,

    #[account(
        mut,
        mint::authority = pool,
    )]
    pub lp_token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub user_lp_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BurnLPTokens<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool".as_ref(), pool.token_mint.as_ref()],
        bump = pool.bump
    )]
    pub pool: Account<'info, LiquidityPool>,

    #[account(
        mut,
        mint::authority = pool,
    )]
    pub lp_token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub user_lp_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn mint_tokens(ctx: Context<MintLPTokens>, amount: u64) -> Result<()> {
    // Verificar que el que firma sea el pool
    require!(
        ctx.accounts.pool.key() == ctx.accounts.authority.key(),
        PoolError::Unauthorized
    );

    // Mintear LP tokens
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.lp_token_mint.to_account_info(),
                to: ctx.accounts.user_lp_token_account.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            &[&[
                b"pool".as_ref(),
                ctx.accounts.pool.token_mint.as_ref(),
                &[ctx.accounts.pool.bump],
            ]],
        ),
        amount,
    )?;

    msg!("Minteados {} LP tokens", amount);
    Ok(())
}

pub fn burn_tokens(ctx: Context<BurnLPTokens>, amount: u64) -> Result<()> {
    // Verificar balance suficiente
    require!(
        ctx.accounts.user_lp_token_account.amount >= amount,
        PoolError::InsufficientLiquidity
    );

    // Quemar LP tokens
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.lp_token_mint.to_account_info(),
                from: ctx.accounts.user_lp_token_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        amount,
    )?;

    msg!("Quemados {} LP tokens", amount);
    Ok(())
} 