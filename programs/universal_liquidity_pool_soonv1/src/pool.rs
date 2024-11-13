use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Mint, TokenAccount, Transfer, MintTo, Burn};

use crate::state::*;

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool".as_ref(), pool.token_mint.as_ref()],
        bump = pool.bump
    )]
    pub pool: Account<'info, LiquidityPool>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user_lp_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub pool_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        mint::authority = pool,
    )]
    pub lp_token_mint: Account<'info, Mint>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool".as_ref(), pool.token_mint.as_ref()],
        bump = pool.bump
    )]
    pub pool: Account<'info, LiquidityPool>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user_lp_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub pool_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        mint::authority = pool,
    )]
    pub lp_token_mint: Account<'info, Mint>,
    
    pub token_program: Program<'info, Token>,
}

pub fn add_liquidity(ctx: Context<AddLiquidity>, amount_sol: u64, amount_token: u64) -> Result<()> {
    // Calcular la cantidad de LP tokens a mintear
    let lp_amount = if ctx.accounts.pool.sol_reserve == 0 && ctx.accounts.pool.token_reserve == 0 {
        (amount_sol as f64 * amount_token as f64).sqrt() as u64
    } else {
        let sol_ratio = amount_sol as f64 / ctx.accounts.pool.sol_reserve as f64;
        let token_ratio = amount_token as f64 / ctx.accounts.pool.token_reserve as f64;
        let min_ratio = sol_ratio.min(token_ratio);
        (min_ratio * ctx.accounts.lp_token_mint.supply as f64) as u64
    };

    // Transferir SOL al pool
    let transfer_ix = anchor_lang::system_program::Transfer {
        from: ctx.accounts.user.to_account_info(),
        to: ctx.accounts.pool.to_account_info(),
    };

    anchor_lang::system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            transfer_ix,
        ),
        amount_sol,
    )?;

    // Transferir tokens al pool
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.accounts.pool_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount_token,
    )?;

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
        lp_amount,
    )?;

    // Actualizar reservas al final
    let pool = &mut ctx.accounts.pool;
    pool.sol_reserve = pool.sol_reserve.checked_add(amount_sol).unwrap();
    pool.token_reserve = pool.token_reserve.checked_add(amount_token).unwrap();
    
    Ok(())
}

pub fn remove_liquidity(ctx: Context<RemoveLiquidity>, lp_amount: u64) -> Result<()> {
    // Calcular cantidades a devolver
    let total_supply = ctx.accounts.lp_token_mint.supply;
    let sol_amount = (ctx.accounts.pool.sol_reserve as u128 * lp_amount as u128 / total_supply as u128) as u64;
    let token_amount = (ctx.accounts.pool.token_reserve as u128 * lp_amount as u128 / total_supply as u128) as u64;

    // Transferir SOL al usuario
    **ctx.accounts.pool.to_account_info().try_borrow_mut_lamports()? = ctx
        .accounts.pool
        .to_account_info()
        .lamports()
        .checked_sub(sol_amount)
        .unwrap();
    **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? = ctx
        .accounts.user
        .to_account_info()
        .lamports()
        .checked_add(sol_amount)
        .unwrap();

    // Transferir tokens al usuario
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.pool_token_account.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            &[&[
                b"pool".as_ref(),
                ctx.accounts.pool.token_mint.as_ref(),
                &[ctx.accounts.pool.bump],
            ]],
        ),
        token_amount,
    )?;

    // Quemar LP tokens
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.lp_token_mint.to_account_info(),
                from: ctx.accounts.user_lp_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        lp_amount,
    )?;

    // Actualizar reservas al final
    let pool = &mut ctx.accounts.pool;
    pool.sol_reserve = pool.sol_reserve.checked_sub(sol_amount).unwrap();
    pool.token_reserve = pool.token_reserve.checked_sub(token_amount).unwrap();
    
    Ok(())
} 