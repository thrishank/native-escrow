use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::{
    instruction::{close_account, transfer},
    state::Account,
};

use crate::{error::Error, state::Offer};

pub fn take(program_id: &Pubkey, accounts: &[AccountInfo<'_>]) -> ProgramResult {
    let [
    maker,
    taker,
    token_mint_a,
    token_mint_b,
    escrow,
    escrow_token_account_a,
    taker_token_account_a,
    taker_token_account_b,
    maker_token_account_b,
    token_program,
    associated_token_program, // associated token program
    system_program] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys)
    };
    if !taker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // token checks
    let escrow_token_ata_a = &get_associated_token_address(escrow.key, token_mint_a.key);
    if escrow_token_ata_a != escrow_token_account_a.key {
        return Err(Error::InvalidTokenATA.into());
    }
    let taker_token_ata_b = &get_associated_token_address(taker.key, token_mint_b.key);
    if taker_token_ata_b != taker_token_account_b.key {
        return Err(Error::InvalidTokenATA.into());
    }
    let taker_token_ata_a = &get_associated_token_address(taker.key, token_mint_a.key);
    if taker_token_ata_a != taker_token_account_a.key {
        return Err(Error::InvalidTokenATA.into());
    }
    let maker_token_ata_b = &get_associated_token_address(maker.key, token_mint_b.key);
    if maker_token_ata_b != maker_token_account_b.key {
        return Err(Error::InvalidTokenATA.into());
    }

    let offer = Offer::try_from_slice(&escrow.data.borrow())?;

    let seeds = &[
        b"token-escrow",
        offer.maker.as_ref(),
        &offer.id.to_be_bytes(),
        &[offer.bump],
    ];

    let escrow_pda = Pubkey::create_program_address(seeds, program_id)?;
    if escrow_pda != *escrow.key {
        return Err(Error::InvalidProgramAddress.into());
    }

    // create token_account_b for maker if it does not exist
    if maker_token_account_b.lamports() == 0 {
        invoke(
            &create_associated_token_account(
                taker.key,
                maker.key,
                token_mint_b.key,
                token_program.key,
            ),
            &[
                token_mint_b.clone(),
                maker_token_account_b.clone(),
                maker.clone(),
                taker.clone(),
                system_program.clone(),
                token_program.clone(),
                associated_token_program.clone(),
            ],
        )?;
    }

    // transfer funds from taker to maker
    invoke(
        &transfer(
            token_program.key,
            taker_token_account_b.key,
            maker_token_account_b.key,
            taker.key,
            &[taker.key],
            offer.amount,
        )?,
        &[
            token_program.clone(),
            taker_token_account_b.clone(),
            maker_token_account_b.clone(),
            taker.clone(),
        ],
    )?;

    // create taker_token_account_a for taker if it does not exist
    if taker_token_account_a.lamports() == 0 {
        invoke(
            &create_associated_token_account(
                taker.key,
                taker.key,
                token_mint_a.key,
                token_program.key,
            ),
            &[
                token_mint_a.clone(),
                taker_token_account_a.clone(),
                taker.clone(),
                taker.clone(),
                system_program.clone(),
                token_program.clone(),
                associated_token_program.clone(),
            ],
        )?;
    }

    let escrow_token_amount = Account::unpack(&escrow_token_account_a.data.borrow())?.amount;

    // transfer funds from escrow to taker
    invoke_signed(
        &transfer(
            token_program.key,
            escrow_token_account_a.key,
            taker_token_account_a.key,
            escrow.key,
            &[escrow.key],
            escrow_token_amount,
        )?,
        &[
            token_program.clone(),
            escrow_token_account_a.clone(),
            taker_token_account_a.clone(),
            escrow.clone(),
        ],
        &[seeds],
    )?;

    // close escrow token account and escrow and refund it to taker
    invoke_signed(
        &close_account(
            token_program.key,
            escrow_token_account_a.key,
            taker.key,
            escrow.key,
            &[escrow.key],
        )?,
        &[
            escrow_token_account_a.clone(),
            taker.clone(),
            escrow.clone(),
        ],
        &[seeds],
    )?;

    let lamports = escrow.lamports();
    **escrow.lamports.borrow_mut() -= lamports;
    **maker.lamports.borrow_mut() += lamports;

    escrow.realloc(0, true)?;
    escrow.assign(system_program.key);

    Ok(())
}
