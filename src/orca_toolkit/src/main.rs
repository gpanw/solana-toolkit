use { 
    solana_client::rpc_client::RpcClient,
    std::{
        str::FromStr,
        path::Path,
    },
    solana_sdk::{
        pubkey::Pubkey,
        transaction::Transaction,
        instruction::{AccountMeta},
        signer::keypair::{read_keypair_file, Keypair},
        signature::Signer,
    },
    solana_program::{
        hash::hashv,
        instruction::Instruction, 
        program_error::ProgramError,
    },
    borsh::{BorshSerialize, BorshDeserialize},
    anchor_lang::{
        AnchorSerialize, 
        AnchorDeserialize,
        Discriminator,
        account,
        system_program::ID
    },
    solana_transaction_status::UiTransactionEncoding,
    spl_associated_token_account::{self, get_associated_token_address},
};

pub const NUM_REWARDS: usize = 3;
pub const TICK_ARRAY_SIZE: usize = 88;

#[derive(BorshDeserialize, Debug)]
pub struct WhirlpoolsConfig {
    pub fee_authority: Pubkey,
    pub collect_protocol_fees_authority: Pubkey,
    pub reward_emissions_super_authority: Pubkey,
    pub default_protocol_fee_rate: u32,
}

#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize, Default, Debug, PartialEq)]
pub struct WhirlpoolRewardInfo {
    /// Reward token mint.
    pub mint: Pubkey,
    /// Reward vault token account.
    pub vault: Pubkey,
    /// Authority account that has permission to initialize the reward and set emissions.
    pub authority: Pubkey,
    /// Q64.64 number that indicates how many tokens per second are earned per unit of liquidity.
    pub emissions_per_second_x64: u128,
    /// Q64.64 number that tracks the total tokens earned per unit of liquidity since the reward
    /// emissions were turned on.
    pub growth_global_x64: u128,
}

#[account]
#[derive(Debug, Default)]
pub struct Whirlpool {
    pub whirlpools_config: Pubkey, // 32
    pub whirlpool_bump: [u8; 1],   // 1

    pub tick_spacing: u16,          // 2
    pub tick_spacing_seed: [u8; 2], // 2

    // Stored as hundredths of a basis point
    // u16::MAX corresponds to ~6.5%
    pub fee_rate: u16, // 2

    // Portion of fee rate taken stored as basis points
    pub protocol_fee_rate: u16, // 2

    // Maximum amount that can be held by Solana account
    pub liquidity: u128, // 16

    // MAX/MIN at Q32.64, but using Q64.64 for rounder bytes
    // Q64.64
    pub sqrt_price: u128,        // 16
    pub tick_current_index: i32, // 4

    pub protocol_fee_owed_a: u64, // 8
    pub protocol_fee_owed_b: u64, // 8

    pub token_mint_a: Pubkey,  // 32
    pub token_vault_a: Pubkey, // 32

    // Q64.64
    pub fee_growth_global_a: u128, // 16

    pub token_mint_b: Pubkey,  // 32
    pub token_vault_b: Pubkey, // 32

    // Q64.64
    pub fee_growth_global_b: u128, // 16

    pub reward_last_updated_timestamp: u64, // 8

    pub reward_infos: [WhirlpoolRewardInfo; NUM_REWARDS], // 384
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct SwapArgs {
    pub amount: u64,
    pub other_amount_threshold: u64,
    pub sqrt_price_limit: u128,
    pub amount_specified_is_input: bool,
    pub a_to_b: bool,
    pub remaining_accounts_info: Option<Vec<Pubkey>>,
}

pub fn create_swap_transaction(
    rpc_client: &RpcClient,
    whirlpool_pubkey: &Pubkey,
    payer_pubkey: &Pubkey,
    payer: &Keypair,
    whirlpool: &Whirlpool,
    anchor_program_id: &Pubkey,
) -> Transaction { 
    // Transaction
    println!("Swap Transaction");
    let mut data = Vec::new();

    let swap_discriminator = &hashv(&[b"global:swap_v2"]).to_bytes()[..8];
    data.extend_from_slice(swap_discriminator);

    let amount: u64 = 1000000000;                 
    let other_amount_threshold: u64 = 0;
    let sqrt_price_limit: u128 = 0;
    let amount_specified_is_input = true;
    let a_to_b = true;
    let remaining_accounts_info = None;
    let swap_args = SwapArgs {
        amount,
        other_amount_threshold,
        sqrt_price_limit,
        amount_specified_is_input,
        a_to_b,
        remaining_accounts_info,
    };
    let swap_args_vec = borsh::to_vec(&swap_args).unwrap();
    swap_args.serialize(&mut data).unwrap();

    println!("______________trick logic____________________");
    let tick_index = whirlpool.tick_current_index;
    let tick_spacing =  whirlpool.tick_spacing;
    let tick_spacing_i32 = tick_spacing as i32;
    let tick_array_size_i32 = TICK_ARRAY_SIZE as i32;
    let real_index = tick_index
        .div_euclid(tick_spacing_i32)
        .div_euclid(tick_array_size_i32);
    let tick_array_start_index = real_index * tick_spacing_i32 * tick_array_size_i32;
    println!("Tick Index: {:?}", tick_index);
    println!("Tick Spacing: {:?}", tick_spacing);
    println!("Real Index: {:?}", real_index);
    println!("Tick Array Start Index: {:?}", tick_array_start_index);

    let offset = whirlpool.tick_spacing as i32 * TICK_ARRAY_SIZE as i32;

    let tick_array_indexes = [
        tick_array_start_index,
        tick_array_start_index + offset,
        tick_array_start_index + offset * 2,
        tick_array_start_index - offset,
        tick_array_start_index - offset * 2,
    ];

    let mut tick_address = get_tick_array_address(anchor_program_id, whirlpool_pubkey, tick_array_indexes[0]);
    println!("Trick Address: {:?}", tick_address);
    let tick_address_0 = tick_address.unwrap().0;
    tick_address = get_tick_array_address(anchor_program_id, whirlpool_pubkey, tick_array_indexes[1]);
    println!("Trick Address: {:?}", tick_address);
    let tick_address_1 = tick_address.unwrap().0;
    tick_address = get_tick_array_address(anchor_program_id, whirlpool_pubkey, tick_array_indexes[2]);
    println!("Trick Address: {:?}", tick_address);
    let tick_address_2 = tick_address.unwrap().0;

    let seeds = &[b"oracle", whirlpool_pubkey.as_ref()];
    let oracle = Pubkey::try_find_program_address(seeds, &anchor_program_id).ok_or(ProgramError::InvalidSeeds);
    println!("Oracle: {:?}", oracle);

    let mut oracle_pubkey = Pubkey::default();

    match oracle {
        Ok((pubkey, _bump)) => {
            println!("Oracle Pubkey: {}", pubkey);
            oracle_pubkey = pubkey;
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }

    let token_program_id = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
    let memo_program_id = Pubkey::from_str("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr").unwrap();

    let token_mint_a = whirlpool.token_mint_a;
    let payer_token_a_account = get_associated_token_address(&payer_pubkey, &token_mint_a);
    println!("Associated Token Account: {:?}", payer_token_a_account);
    let token_vault_a = whirlpool.token_vault_a;

    let token_mint_b = whirlpool.token_mint_b;
    let payer_token_b_account = get_associated_token_address(&payer_pubkey, &token_mint_b);
    println!("Associated Token Account: {:?}", payer_token_b_account);
    let token_vault_b = whirlpool.token_vault_b;

    // let tick_array_0 = Pubkey::from_str(tick_address_0);
    // let tick_array_1 = Pubkey::from_str(tick_address_1);
    // let tick_array_2 = Pubkey::from_str(tick_address_2);

    let accounts = vec![
        AccountMeta::new_readonly(token_program_id, false),  
        AccountMeta::new_readonly(token_program_id, false), 
        AccountMeta::new_readonly(memo_program_id, false),   
        AccountMeta::new_readonly(*payer_pubkey, true),   // Payer account (signer) 
        AccountMeta::new(*whirlpool_pubkey, false),
        AccountMeta::new_readonly(token_mint_a, false), // Token mint a
        AccountMeta::new_readonly(token_mint_b, false), // Token mint b
        AccountMeta::new(payer_token_a_account, false),
        AccountMeta::new(token_vault_a, false),
        AccountMeta::new(payer_token_b_account, false),
        AccountMeta::new(token_vault_b, false),
        AccountMeta::new(tick_address_0, false),
        AccountMeta::new(tick_address_1, false),
        AccountMeta::new(tick_address_2 , false),
        AccountMeta::new(oracle_pubkey, false), // System program
        // remaining account
        // AccountMeta::new(tick_array_0, false),
    ];

    let anchor_instruction = Instruction {
        program_id: *anchor_program_id,
        accounts: accounts,
        data: data,
    };
    println!("Anchor Instruction: {:?}", anchor_instruction);
    let recent_blockhash = rpc_client.get_latest_blockhash().expect("Error in blockhash");


    Transaction::new_signed_with_payer(
        &[anchor_instruction], 
        Some(&payer_pubkey), 
        &[payer], 
        recent_blockhash,
    )

}

pub fn get_tick_array_address(
    program_id: &Pubkey,
    whirlpool: &Pubkey,
    start_tick_index: i32,
) -> Result<(Pubkey, u8), ProgramError> {
    let WHIRLPOOL_ID: &Pubkey = program_id;
    let start_tick_index_str = start_tick_index.to_string();
    let seeds = &[
        b"tick_array",
        whirlpool.as_ref(),
        start_tick_index_str.as_bytes(),
    ];

    Pubkey::try_find_program_address(seeds, &WHIRLPOOL_ID).ok_or(ProgramError::InvalidSeeds)
}

fn test() {
    let data: Vec<u8> = [63, 149, 209, 12, 225, 128, 99, 9, 19, 228, 65, 248, 57, 19, 202, 104, 176, 99, 79, 176, 37, 253, 234, 168, 135, 55, 232, 65, 16, 209, 37, 94, 53, 123, 51, 119, 221, 238, 28, 205, 254, 16, 0, 16, 0, 64, 6, 20, 5, 41, 203, 234, 108, 217, 119, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 19, 43, 169, 240, 251, 44, 38, 96, 0, 0, 0, 0, 0, 0, 0, 0, 125, 179, 255, 255, 194, 52, 200, 8, 0, 0, 0, 0, 74, 66, 15, 1, 0, 0, 0, 0, 6, 155, 136, 87, 254, 171, 129, 132, 251, 104, 127, 99, 70, 24, 192, 53, 218, 196, 57, 220, 26, 235, 59, 85, 152, 160, 240, 0, 0, 0, 0, 1, 29, 119, 163, 197, 29, 226, 97, 144, 48, 75, 0, 9, 18, 28, 99, 233, 255, 187, 134, 255, 165, 87, 50, 192, 65, 231, 94, 193, 98, 96, 122, 149, 246, 192, 39, 166, 249, 203, 21, 187, 0, 0, 0, 0, 0, 0, 0, 0, 121, 120, 183, 20, 69, 60, 211, 232, 122, 235, 31, 192, 155, 240, 103, 249, 108, 210, 212, 214, 155, 87, 19, 149, 170, 155, 241, 134, 175, 249, 218, 63, 69, 39, 148, 199, 158, 4, 169, 92, 9, 30, 79, 233, 59, 146, 187, 60, 207, 179, 47, 156, 54, 56, 219, 227, 129, 158, 2, 248, 104, 109, 240, 239, 64, 175, 192, 154, 126, 240, 174, 43, 0, 0, 0, 0, 0, 0, 0, 0, 246, 179, 96, 104, 0, 0, 0, 0, 121, 120, 183, 20, 69, 60, 211, 232, 122, 235, 31, 192, 155, 240, 103, 249, 108, 210, 212, 214, 155, 87, 19, 149, 170, 155, 241, 134, 175, 249, 218, 63, 63, 212, 24, 15, 50, 85, 7, 7, 231, 235, 169, 8, 144, 240, 112, 252, 230, 233, 91, 229, 11, 215, 148, 159, 203, 197, 132, 62, 34, 12, 237, 117, 189, 29, 49, 175, 23, 222, 255, 60, 38, 132, 129, 96, 10, 202, 254, 75, 20, 9, 140, 15, 225, 65, 183, 244, 161, 205, 248, 73, 52, 100, 68, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 97, 19, 250, 227, 216, 202, 64, 228, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 189, 29, 49, 175, 23, 222, 255, 60, 38, 132, 129, 96, 10, 202, 254, 75, 20, 9, 140, 15, 225, 65, 183, 244, 161, 205, 248, 73, 52, 100, 68, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 189, 29, 49, 175, 23, 222, 255, 60, 38, 132, 129, 96, 10, 202, 254, 75, 20, 9, 140, 15, 225, 65, 183, 244, 161, 205, 248, 73, 52, 100, 68, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].into();
    let relevant_data = &data[8..];
    let decoded = Whirlpool::try_from_slice(&relevant_data)
        .expect("Failed to deserialize Anchor account");

    println!("{:?}", decoded);

    let disc = Whirlpool::DISCRIMINATOR;
    println!("Whirlpool discriminator: {:?}", disc);
}

fn main() {
    println!("Hello, world!");
    // test();
    let rpc_client = RpcClient::new("http://127.0.0.1:8899".to_string());
    // let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());

    let program_id = Pubkey::from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc").unwrap();

    let whirlpool_config_id = Pubkey::from_str(
        "2LecshUwdy9xi7meFgHtFJQNSKk4KdTrcpvaB56dP2NQ",
      );

    let whirlpool_pubkey = Pubkey::from_str(
        "C9U2Ksk6KKWvLEeo5yUQ7Xu46X7NzeBJtd9PBfuXaUSM",
    ).unwrap();

    println!("Whirlpool: {:?}", whirlpool_pubkey);

    let whirlpool = rpc_client
    .get_account(&whirlpool_pubkey)
    .expect("Failed to fetch account data");

    let whirlpool_data = whirlpool.data;
    let relevant_data = &whirlpool_data[8..];

    let pool_data: Whirlpool = Whirlpool::try_from_slice(&relevant_data).expect("Failed to parse account data");

    println!("Pool: {:?}", pool_data);

    let path = Path::new("/root/.config/solana/id.json");
    let wallet_keypair = read_keypair_file(path).unwrap();
    let wallet_public_key = wallet_keypair.pubkey();

    let mut anchor_tx =create_swap_transaction(
        &rpc_client,
        &whirlpool_pubkey,
        &wallet_public_key,
        &wallet_keypair,
        &pool_data,
        &program_id
    );

    println!("Solana Logs ------------------------------------");

    // let anchor_signature = rpc_client.send_and_confirm_transaction(&anchor_tx).expect("Transaction failed");
    // println!("Signature: {:?}", anchor_signature);

    // let encoding = UiTransactionEncoding::Json;
    // let transaction_details = rpc_client.get_transaction(&anchor_signature, encoding).expect("Error in fetching transaction");
    // println!("Solana Logs ------------------------------------");

    // let logs = transaction_details.transaction.meta.unwrap().log_messages;
    // // for log in &logs {
    // //     println!("{}", log);
    // // }

    // println!("Logs: {:?}", logs);

    test();
}
