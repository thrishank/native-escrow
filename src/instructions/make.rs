use borsh::BorshSerialize;
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::rent::Rent;
use solana_program::system_instruction;
use solana_program::sysvar::Sysvar;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token::instruction::transfer;
use spl_token::state::Account;

use crate::error::Error;
use crate::state::Offer;

pub fn make(
    program_id: &Pubkey,
    accounts: &[AccountInfo<'_>],
    id: u64,
    deposit: u64,
    recieve: u64,
) -> ProgramResult {
    let [
        maker,                    // offer account info
        token_mint_a,             // token_mint a
        token_mint_b,             // token mint b
        maker_token_account_a,    // maker token account a
        escrow,                   // vault
        escrow_token_account_a,   // escrow token account a
        token_program,            // token program
        associated_token_program, // associated token program
        system_program,           // system program
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let maker_token_ata = &get_associated_token_address(maker.key, token_mint_a.key);
    if maker_token_ata != maker_token_account_a.key {
        return Err(Error::InvalidTokenATA.into());
    }

    let escrow_token_ata = &get_associated_token_address(escrow.key, token_mint_a.key);
    if escrow_token_ata != escrow_token_account_a.key {
        return Err(Error::InvalidTokenATA.into());
    }

    let (escrow_pda, bump) = Pubkey::find_program_address(
        &[b"token-escrow", maker.key.as_ref(), &id.to_be_bytes()],
        program_id,
    );

    if escrow_pda != *escrow.key {
        return Err(Error::InvalidProgramAddress.into());
    }

    let offer = Offer {
        id,
        maker: *maker.key,
        mint_a: *token_mint_a.key,
        mint_b: *token_mint_b.key,
        amount: recieve,
        bump,
    };

    let size = borsh::to_vec::<Offer>(&offer)?.len();
    let lamports = (Rent::get()?).minimum_balance(size);

    // create escrow pda
    invoke_signed(
        &system_instruction::create_account(
            maker.key,
            escrow.key,
            lamports,
            size as u64,
            program_id,
        ),
        &[maker.clone(), escrow.clone(), system_program.clone()],
        &[&[
            b"token-escrow",
            maker.key.as_ref(),
            &id.to_be_bytes(),
            &[bump],
        ]],
    )?;

    // create escrow token ata
    invoke(
        &create_associated_token_account(
            maker.key,
            escrow.key,
            token_mint_a.key,
            token_program.key,
        ),
        &[
            token_mint_a.clone(),
            escrow_token_account_a.clone(),
            escrow.clone(),
            maker.clone(),
            system_program.clone(),
            token_program.clone(),
            associated_token_program.clone(),
        ],
    )?;

    // transfer tokens to escrow
    invoke(
        &transfer(
            token_program.key,
            maker_token_account_a.key,
            escrow_token_account_a.key,
            maker.key,
            &[maker.key],
            deposit,
        )?,
        &[
            token_program.clone(),
            maker_token_account_a.clone(),
            escrow_token_account_a.clone(),
            maker.clone(),
        ],
    )?;

    let escrow_token_amount = Account::unpack(&escrow_token_account_a.data.borrow())?.amount;

    assert_eq!(escrow_token_amount, deposit);

    offer.serialize(&mut *escrow.data.borrow_mut())?;

    Ok(())
}
