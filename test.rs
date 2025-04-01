use borsh::BorshSerialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;

use std::str::FromStr;

fn test() {
    #[derive(BorshSerialize)]
    pub enum Instructions {
        Init { id: u64, deposit: u64, receive: u64 },
        Take {},
        Refund {},
    }
    let id: u64 = 2005;
    let init_data = Instructions::Init {
        id,
        deposit: 10_000_000_000,
        receive: 20_000_000_000,
    };
    let mut instr_in_bytes: Vec<u8> = Vec::new();
    init_data.serialize(&mut instr_in_bytes).unwrap();

    let program_id: Pubkey =
        Pubkey::from_str("6YwczHkrhMkuajdvcE6CqcanGzxrZDtwhdiQ6PPU98no").expect("Invalid pubkey");

    let maker = Pubkey::from_str("thrbabBvANwvKdV34GdrFUDXB6YMsksdfmiKj2ZUV3m").unwrap();
    let token_mint_a = Pubkey::from_str("sL6rL3RtoruMpbgXJVvuDFnyBE7TXVpYiLnQNXPduYi").unwrap();
    let token_mint_b = Pubkey::from_str("2o1Now7iEgswjGBexe6sbUbGzMx325fUrbeLVgMedNfU").unwrap();
    let maker_token_account_a =
        Pubkey::from_str("WcqfekmkuLGBTHccGF7Rj3GYoR1ZcBLJ4cq4y4bRTfk").unwrap();

    let escrow = Pubkey::find_program_address(
        &[b"token-escrow", maker.as_ref(), &id.to_be_bytes()],
        &program_id,
    )
    .0;
    let escrow_token_ata = get_associated_token_address(&escrow, &token_mint_a);
    let token_program = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
    let associated_token_program =
        Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap();
    let system_program = Pubkey::from_str("11111111111111111111111111111111").unwrap();

    let accounts: Vec<AccountMeta> = vec![
        AccountMeta::new(maker, true),
        AccountMeta::new(token_mint_a, false),
        AccountMeta::new(token_mint_b, false),
        AccountMeta::new(maker_token_account_a, false),
        AccountMeta::new(escrow, false),
        AccountMeta::new(escrow_token_ata, false),
        AccountMeta::new_readonly(token_program, false),
        AccountMeta::new_readonly(associated_token_program, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let payer: Keypair = Keypair::from_base58_string("");

    sign_and_send_tx(instr_in_bytes, accounts, &payer);

    let taker_key = Pubkey::from_str("DoQ47WTYzvgCNXwVK1Uf3urpXqa8maE7hJFd1xNLYUp2").unwrap();
    let taker_token_account_a = get_associated_token_address(&taker_key, &token_mint_a);
    let taker_token_account_b = get_associated_token_address(&taker_key, &token_mint_b);
    let maker_token_account_b = get_associated_token_address(&maker, &token_mint_b);

    let take_accounts: Vec<AccountMeta> = vec![
        AccountMeta::new(maker, false),
        AccountMeta::new(taker_key, true),
        AccountMeta::new(token_mint_a, false),
        AccountMeta::new(token_mint_b, false),
        AccountMeta::new(escrow, false),
        AccountMeta::new(escrow_token_ata, false),
        AccountMeta::new(taker_token_account_a, false),
        AccountMeta::new(taker_token_account_b, false),
        AccountMeta::new(maker_token_account_b, false),
        AccountMeta::new_readonly(token_program, false),
        AccountMeta::new_readonly(associated_token_program, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let take = Instructions::Take {};
    let mut take_ix_data: Vec<u8> = Vec::new();
    take.serialize(&mut take_ix_data).unwrap();

    let taker = Keypair::from_base58_string("");

    sign_and_send_tx(take_ix_data, take_accounts, taker);
    let refund = Instructions::Refund {};
    let mut refund_ix_data: Vec<u8> = Vec::new();
    refund.serialize(&mut refund_ix_data).unwrap();

    let close_accounts: Vec<AccountMeta> = vec![
        AccountMeta::new(maker, true),
        AccountMeta::new(token_mint_a, false),
        AccountMeta::new(escrow, false),
        AccountMeta::new(escrow_token_ata, false),
        AccountMeta::new(maker_token_account_a, false),
        AccountMeta::new_readonly(token_program, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    sign_and_send_tx(refund_ix_data, close_accounts, &payer);
}

fn sign_and_send_tx(instruction_data: Vec<u8>, accounts: Vec<AccountMeta>, payer: &Keypair) {
    let rpc_client: RpcClient = RpcClient::new("https://api.devnet.solana.com");

    let program_id: Pubkey =
        Pubkey::from_str("6YwczHkrhMkuajdvcE6CqcanGzxrZDtwhdiQ6PPU98no").expect("Invalid pubkey");

    let recent_blockhash = rpc_client
        .get_latest_blockhash()
        .expect("Failed to get blockhash");

    let ix = Instruction::new_with_bytes(program_id, &instruction_data, accounts);

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    let signature = rpc_client
        .send_and_confirm_transaction(&tx)
        .expect("Transaction failed");
    println!("Transaction successful! Signature: {}", signature);
}

fn main() {
    test();
}
