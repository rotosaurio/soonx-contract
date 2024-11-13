use anchor_lang::prelude::*;

declare_id!("GndtcpvZJN6ybuNZJQsRGbBCAX32cHHYev4aJjbDrzLY");

mod factory;
mod pool;
mod lp_token;
mod fee_handler;
mod state;
mod errors;

use factory::*;
use pool::*;
use lp_token::*;
use fee_handler::*;

#[program]
pub mod universal_liquidity_pool_soonv1 {
    use super::*;

    // Factory Instructions
    pub fn create_pool(ctx: Context<CreatePool>, pool_bump: u8) -> Result<()> {
        factory::create_pool(ctx, pool_bump)
    }

    pub fn initialize_factory(ctx: Context<InitializeFactory>) -> Result<()> {
        factory::initialize_factory(ctx)
    }

    pub fn update_protocol_fee(ctx: Context<UpdateFees>, new_fee_rate: u64) -> Result<()> {
        fee_handler::update_protocol_fee(ctx, new_fee_rate)
    }

    // Pool Instructions
    pub fn add_liquidity(ctx: Context<AddLiquidity>, amount_sol: u64, amount_token: u64) -> Result<()> {
        pool::add_liquidity(ctx, amount_sol, amount_token)
    }

    pub fn remove_liquidity(ctx: Context<RemoveLiquidity>, lp_amount: u64) -> Result<()> {
        pool::remove_liquidity(ctx, lp_amount)
    }

    // LP Token Instructions
    pub fn mint_lp_tokens(ctx: Context<MintLPTokens>, amount: u64) -> Result<()> {
        lp_token::mint_tokens(ctx, amount)
    }

    pub fn burn_lp_tokens(ctx: Context<BurnLPTokens>, amount: u64) -> Result<()> {
        lp_token::burn_tokens(ctx, amount)
    }

    // Fee Handler Instructions
    pub fn collect_fees(ctx: Context<CollectFees>) -> Result<()> {
        fee_handler::collect_fees(ctx)
    }
}

#[derive(Accounts)] 
pub struct Initialize {}
