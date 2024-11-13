use anchor_lang::prelude::*;

#[account]
pub struct PoolFactory {
    pub authority: Pubkey,
    pub pools_count: u64,
    pub creation_fee: u64,
    pub protocol_fee_rate: u64,
}

#[account]
pub struct LiquidityPool {
    pub factory: Pubkey,
    pub token_mint: Pubkey,
    pub sol_reserve: u64,
    pub token_reserve: u64,
    pub lp_token_mint: Pubkey,
    pub last_price: u64,
    pub bump: u8,
}

#[account]
pub struct FeeVault {
    pub authority: Pubkey,
    pub accumulated_fees: u64,
    pub last_collection_timestamp: i64,
} 