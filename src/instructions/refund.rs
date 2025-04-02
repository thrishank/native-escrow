use borsh::BorshDeserialize;
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address;
use spl_token::instruction::{close_account, transfer};
use spl_token::state::Account;

use crate::error::Error;
use crate::state::Offer;

pub fn refund(program_id: &Pubkey, accounts: &[AccountInfo<'_>]) -> ProgramResult {
    let [maker, token_mint_a, escrow, escrow_token_account_a, maker_token_account_a, token_program, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let escrow_data = Offer::try_from_slice(&escrow.data.borrow())?;
    let seeds = &[
        b"token-escrow",
        maker.key.as_ref(),
        &escrow_data.id.to_be_bytes(),
        &[escrow_data.bump],
    ];
    let escrow_pda = Pubkey::create_program_address(seeds, program_id)?;

    if *escrow.key != escrow_pda {
        return Err(Error::InvalidProgramAddress.into());
    }

    let maker_token_ata = &get_associated_token_address(maker.key, token_mint_a.key);
    if maker_token_ata != maker_token_account_a.key {
        return Err(Error::InvalidTokenATA.into());
    }
    let escrow_token_ata = &get_associated_token_address(escrow.key, token_mint_a.key);
    if escrow_token_ata != escrow_token_account_a.key {
        return Err(Error::InvalidTokenATA.into());
    }

    let escrow_token_amount = Account::unpack(&escrow_token_account_a.data.borrow())?.amount;

    // refund all the funds in the escrow to maket_token_account_a
    invoke_signed(
        &transfer(
            token_program.key,
            escrow_token_account_a.key,
            maker_token_account_a.key,
            escrow.key,
            &[escrow.key],
            escrow_token_amount,
        )?,
        &[
            token_program.clone(),
            escrow_token_account_a.clone(),
            maker_token_account_a.clone(),
            escrow.clone(),
        ],
        &[seeds],
    )?;

    // close the escrow_token_account_a
    invoke_signed(
        &close_account(
            token_program.key,
            escrow_token_account_a.key,
            maker.key,
            escrow.key,
            &[escrow.key],
        )?,
        &[
            escrow_token_account_a.clone(),
            maker.clone(),
            escrow.clone(),
        ],
        &[seeds],
    )?;

    // close the escrow
    let lamports = escrow.lamports();
    **escrow.lamports.borrow_mut() -= lamports;
    **maker.lamports.borrow_mut() += lamports;

    escrow.realloc(0, true)?;
    escrow.assign(system_program.key);

    Ok(())
}
