use {
    std::path::Path,
    borsh::{BorshSerialize, BorshDeserialize},
    solana_client::rpc_client::RpcClient,
    solana_program::{
        instruction::Instruction, 
        pubkey::Pubkey, 
        program_error::ProgramError,
        hash::hashv,
    },
    solana_sdk::{
        signer::keypair::{read_keypair_file, Keypair},
        signature::Signer,
        transaction::Transaction,
        instruction::{AccountMeta},
    },
    solana_transaction_status::UiTransactionEncoding,
};

fn main() {
    println!("test mock onchain program");
    let rpc_client = RpcClient::new("https://api.devnet.solana.com".to_string());

    let path = "/root/.config/solana/id.json".to_string();
    let my_account: MyAccount = get_public_key(path);

    let payer_pubkey: Pubkey = my_account.public_key;
    let payer = my_account.keypair;

    let mut anchor_tx =create_onchain_transaction(
        &rpc_client,
        &payer_pubkey,
        &payer,
        &Pubkey::from_str_const("57bfie2LvSfQbirTnWKda6waCwyo2WQeq7ms5Q5VtbJC") // on chain program_id
    );

    println!("Solana Logs ------------------------------------");

    let anchor_signature = rpc_client.send_and_confirm_transaction(&anchor_tx).expect("Transaction failed");
    println!("Signature: {:?}", anchor_signature);

    let encoding = UiTransactionEncoding::Json;
    let transaction_details = rpc_client.get_transaction(&anchor_signature, encoding).expect("Error in fetching transaction");
    // println!("Transaction Details: {:?}", transaction_details);
    println!("Solana Logs ------------------------------------");

    let logs = transaction_details.transaction.meta.unwrap().log_messages.unwrap();
    for log in &logs {
        println!("{}", log);
    }

    println!("Logs: {:?}", logs);
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

pub struct MyAccount {
    pub keypair: Keypair,
    pub public_key: Pubkey,
}

pub fn get_public_key(path: String) -> MyAccount {   
    
    let path = Path::new(path.as_str());
    println!("Path: {:?}", path);
    let keypair = read_keypair_file(path).unwrap();

    let secret_key = keypair.secret();
    let public_key = keypair.pubkey();

    MyAccount { keypair: keypair, public_key: public_key }
}

pub fn create_onchain_transaction(
    rpc_client: &RpcClient,
    payer_pubkey: &Pubkey,
    payer: &Keypair,
    anchor_program_id: &Pubkey,
) -> Transaction {
    println!("Swap Transaction");

    let mut data = Vec::new();

    let swap_discriminator = &hashv(&[b"global:swap"]).to_bytes()[..8];
    data.extend_from_slice(swap_discriminator);

    // Add the first 8 bytes (Anchor Discriminator)
    // Serialize the swap arguments and append
    let amount: u64 = 2000000000;
    let other_amount_threshold: u64 = 2000000000;
    // let sqrt_price_limit: u128 = whirlpool.sqrt_price;
    // preparing data for swap instructions
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

    let token_program_id = Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
    let accounts = vec![
        AccountMeta::new_readonly(token_program_id, false),
        AccountMeta::new(*payer_pubkey, true),
    ];
    // end data for swap instructions

    // prepare data for initialize instructions
    // let swap_discriminator = &hashv(&[b"global:initialize"]).to_bytes()[..8];
    data.extend_from_slice(swap_discriminator);

    println!("Swap Discriminator: {:?}", swap_discriminator);  

    // let accounts = vec![];
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
