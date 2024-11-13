use anchor_lang::prelude::*;

#[error_code]
pub enum PoolError {
    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,
    
    #[msg("Invalid token mint")]
    InvalidTokenMint,
    
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    
    #[msg("Invalid fee amount")]
    InvalidFeeAmount,
    
    #[msg("Unauthorized")]
    Unauthorized,
} 