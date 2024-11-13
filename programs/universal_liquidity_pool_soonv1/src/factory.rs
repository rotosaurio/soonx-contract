use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token};

use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
#[instruction(pool_bump: u8)]
pub struct CreatePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<LiquidityPool>(),
        seeds = [b"pool".as_ref(), token_mint.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, LiquidityPool>,
    
    pub token_mint: Account<'info, token::Mint>,
    
    #[account(
        mut,
        seeds = [b"factory".as_ref()],
        bump
    )]
    pub factory: Account<'info, PoolFactory>,
    
    #[account(
        init,
        payer = authority,
        mint::decimals = 9,
        mint::authority = pool,
    )]
    pub lp_token_mint: Account<'info, token::Mint>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitializeFactory<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<PoolFactory>(),
        seeds = [b"factory".as_ref()],
        bump
    )]
    pub factory: Account<'info, PoolFactory>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn initialize_factory(ctx: Context<InitializeFactory>) -> Result<()> {
    let factory = &mut ctx.accounts.factory;
    
    factory.authority = ctx.accounts.authority.key();
    factory.pools_count = 0;
    factory.creation_fee = 1_000_000_000; // 1 SOL como fee inicial
    factory.protocol_fee_rate = 30; // 0.3%
    
    Ok(())
}

pub fn create_pool(ctx: Context<CreatePool>, pool_bump: u8) -> Result<()> {
    // Verificar que el creador tenga suficientes SOL para el fee
    let creation_fee = ctx.accounts.factory.creation_fee;
    require!(
        ctx.accounts.authority.lamports() >= creation_fee,
        PoolError::InsufficientLiquidity
    );
    
    // Transferir el fee de creación
    let transfer_ix = anchor_lang::system_program::Transfer {
        from: ctx.accounts.authority.to_account_info(),
        to: ctx.accounts.factory.to_account_info(),
    };

    anchor_lang::system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            transfer_ix,
        ),
        creation_fee,
    )?;
    
    // Inicializar el pool
    let pool = &mut ctx.accounts.pool;
    pool.factory = ctx.accounts.factory.key();
    pool.token_mint = ctx.accounts.token_mint.key();
    pool.sol_reserve = 0;
    pool.token_reserve = 0;
    pool.lp_token_mint = ctx.accounts.lp_token_mint.key();
    pool.last_price = 0;
    pool.bump = pool_bump;
    
    // Incrementar el contador de pools
    let factory = &mut ctx.accounts.factory;
    factory.pools_count = factory.pools_count.checked_add(1).unwrap();
    
    msg!("Pool creado exitosamente");
    Ok(())
} 