use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::entrypoint;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

entrypoint!(process_instruction);

pub mod error;
pub mod instructions;
pub mod state;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
enum Instructions {
    Make { id: u64, deposit: u64, receive: u64 },
    Take {},
    Refund {},
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let data = Instructions::try_from_slice(instruction_data)?;
    match data {
        Instructions::Make {
            id,
            deposit,
            receive,
        } => {
            instructions::make(program_id, accounts, id, deposit, receive)?;
        }
        Instructions::Take {} => {
            instructions::take(program_id, accounts)?;
        }
        Instructions::Refund {} => {
            instructions::refund(program_id, accounts)?;
        }
    }
    Ok(())
}

solana_security_txt::security_txt! {
    name: "native-token-escrow",
    source_code: "https://github.com/thrishank/native-escrow",
    contacts: "https://github.com/thrishank",
    project_url: "https://github.com/thrishank/native-escrow",
    policy: "",
    preferred_languages: "en"
}
