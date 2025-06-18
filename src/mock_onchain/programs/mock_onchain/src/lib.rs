use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token};


declare_id!("57bfie2LvSfQbirTnWKda6waCwyo2WQeq7ms5Q5VtbJC");

#[program]
pub mod mock_onchain {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn swap(ctx: Context<Swap>, amount: u64,
        other_amount_threshold: u64,
        sqrt_price_limit: u128,
        amount_specified_is_input: bool,
        a_to_b: bool,
        remaining_accounts_info: Option<u64>,
    ) -> Result<()> {
        msg!("Swap: {:?}", ctx.program_id);
        msg!("other_amount_threshold: {:?}", other_amount_threshold);
        msg!("sqrt_price_limit: {:?}", sqrt_price_limit);
        msg!("amount_specified_is_input: {:?}", amount_specified_is_input);
        msg!("a_to_b: {:?}", a_to_b);
        msg!("remaining_accounts_info: {:?}", remaining_accounts_info);
        Ok(())
        // instructions::swap(ctx, amount)
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,

    pub token_authority: Signer<'info>,
}
