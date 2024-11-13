use anchor_lang::prelude::*;

use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct CollectFees<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"factory".as_ref()],
        bump,
    )]
    pub factory: Account<'info, PoolFactory>,

    #[account(
        mut,
        seeds = [b"fee_vault".as_ref()],
        bump,
    )]
    pub fee_vault: Account<'info, FeeVault>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateFees<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"factory".as_ref()],
        bump,
        constraint = factory.authority == authority.key() @ PoolError::Unauthorized
    )]
    pub factory: Account<'info, PoolFactory>,
}

pub fn collect_fees(ctx: Context<CollectFees>) -> Result<()> {
    let fee_vault = &mut ctx.accounts.fee_vault;
    let current_time = Clock::get()?.unix_timestamp;

    // Verificar que hayan pasado al menos 24 horas desde la última colección
    require!(
        current_time - fee_vault.last_collection_timestamp >= 86400,
        PoolError::Unauthorized
    );

    // Verificar que quien firma sea la autoridad del fee vault
    require!(
        fee_vault.authority == ctx.accounts.authority.key(),
        PoolError::Unauthorized
    );

    // Obtener el balance actual del vault y calcular la comisión
    let vault_balance = fee_vault.accumulated_fees;
    require!(vault_balance > 0, PoolError::InsufficientLiquidity);

    let fee_amount = calculate_fee_amount(vault_balance, ctx.accounts.factory.protocol_fee_rate)?;

    // Transferir los fees acumulados a la autoridad
    **fee_vault.to_account_info().try_borrow_mut_lamports()? = fee_vault
        .to_account_info()
        .lamports()
        .checked_sub(fee_amount)
        .unwrap();

    **ctx.accounts.authority.to_account_info().try_borrow_mut_lamports()? = ctx
        .accounts.authority
        .to_account_info()
        .lamports()
        .checked_add(fee_amount)
        .unwrap();

    // Actualizar el estado del vault
    fee_vault.accumulated_fees = vault_balance.checked_sub(fee_amount).unwrap();
    fee_vault.last_collection_timestamp = current_time;

    msg!("Fees colectados: {} lamports", fee_amount);
    Ok(())
}

pub fn update_protocol_fee(ctx: Context<UpdateFees>, new_fee_rate: u64) -> Result<()> {
    require!(new_fee_rate <= 1000, PoolError::InvalidFeeAmount); // Max 10%

    let factory = &mut ctx.accounts.factory;
    factory.protocol_fee_rate = new_fee_rate;

    msg!("Tasa de fee actualizada a: {}%", new_fee_rate as f64 / 100.0);
    Ok(())
}

pub(crate) fn calculate_fee_amount(amount: u64, fee_rate: u64) -> Result<u64> {
    Ok((amount as u128 * fee_rate as u128 / 10000) as u64)
} 