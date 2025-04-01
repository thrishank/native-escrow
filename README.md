# Native Escrow

- [Accounts](#accounts)
- [Make Instruction](#make-instruction)
- [Take Instruction](#take-instruction)
- [Refund Instruction](#refund-instruction)
- [Sign and Send Transaction](#sign-and-send-transaction)
- [Cargo.toml for Testing](#cargotoml-for-testing)

---

# Testing

## Accounts

```rust
let program_id: Pubkey = Pubkey::from_str("6YwczHkrhMkuajdvcE6CqcanGzxrZDtwhdiQ6PPU98no")?;

let maker = Pubkey::from_str("")?;
let token_mint_a = Pubkey::from_str("")?;
let token_mint_b = Pubkey::from_str("")?;
let maker_token_account_a = Pubkey::from_str("")?;
let maker_token_account_b = get_associated_token_address(&maker, &token_mint_b);

let taker_key = Pubkey::from_str("")?;
let taker_token_account_a = get_associated_token_address(&taker_key, &token_mint_a);
let taker_token_account_b = get_associated_token_address(&taker_key, &token_mint_b);


let escrow = Pubkey::find_program_address(
    &[b"token-escrow", maker.as_ref(), &id.to_be_bytes()],
    &program_id,
).0;

let escrow_token_ata = get_associated_token_address(&escrow, &token_mint_a);

let token_program = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")?;
let associated_token_program = Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL")?;
let system_program = Pubkey::from_str("11111111111111111111111111111111")?;
```

## Make Instruction

```rust
#[derive(BorshSerialize)]
pub enum Instruction { // order of the instruction matters
  Make {id: u64, deposit: u64, recieve: u64},
  Take {},
  Refund {}
}
// serialize the instruction
let mut instr_in_bytes: Vec<u8> = Vec::new();

let id: u64 = 23424; // random number can be used only once
let init_data = Instructions::Init {
    id,
    deposit: 10_000_000_000,
    receive: 20_000_000_000,
};

init_data.serialize(&mut instr_in_bytes)?;

// accounts must be in the same order
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

let maker_keypair: Keypair = Keypair::from_base58_string("private bs58 string");

sign_and_send_tx(instr_in_bytes, accounts, &maker_keypair);
```

## Take Instruction

```rust
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

let taker = Keypair::from_base58_string("taker private key");

sign_and_send_tx(take_ix_data, take_accounts, &taker);
```

## Refund Instruction

```rust
let refund = Instructions::Refund {};
let mut refund_ix_data: Vec<u8> = Vec::new();
refund.serialize(&mut refund_ix_data).unwrap();

let refund_accounts: Vec<AccountMeta> = vec![
    AccountMeta::new(maker, true),
    AccountMeta::new(token_mint_a, false),
    AccountMeta::new(escrow, false),
    AccountMeta::new(escrow_token_ata, false),
    AccountMeta::new(maker_token_account_a, false),
    AccountMeta::new_readonly(token_program, false),
    AccountMeta::new_readonly(system_program, false),
];

sign_and_send_tx(refund_ix_data, refund_accounts, &maker_keypair);
```

## Sign and send transaction

```rust
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
```

## Cargo.toml for testing

```toml
[package]
name = "rust"
version = "0.1.0"
edition = "2021"

[dependencies]
solana-sdk = "~1.18.11"
solana-client = "~1.18.11"
borsh = "1.5"
spl-associated-token-account = { version = "3.0.4", features = [
  "no-entrypoint",
] }
```
