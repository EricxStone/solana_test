use std::convert::TryInto;
use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
    instruction::{AccountMeta, Instruction},
    msg,
};

use crate::error::EscrowError::InvalidInstruction;

use std::mem::size_of;

pub enum EscrowInstruction {

    /// Starts the trade by creating and populating an escrow account and transferring ownership of the given temp token account to the PDA
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person initializing the escrow
    /// 1. `[writable]` Temporary token account that should be created prior to this instruction and owned by the initializer
    /// 2. `[]` The initializer's token account for the token they will receive should the trade go through
    /// 3. `[writable]` The escrow account, it will hold all necessary info about the trade.
    /// 4. `[]` The rent sysvar
    /// 5. `[]` The token program
    InitEscrow {
        /// The amount party A expects to receive of token Y
        amount: u64
    },

    /// Accepts a trade
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person taking the trade
    /// 1. `[writable]` The taker's token account for the token they send 
    /// 2. `[writable]` The taker's token account for the token they will receive should the trade go through
    /// 3. `[writable]` The PDA's temp token account to get tokens from and eventually close
    /// 4. `[writable]` The initializer's main account to send their rent fees to
    /// 5. `[writable]` The initializer's token account that will receive tokens
    /// 6. `[writable]` The escrow account holding the escrow info
    /// 7. `[]` The token program
    /// 8. `[]` The PDA account
    Exchange {
        /// the amount the taker expects to be paid in the other token, as a u64 because that's the max possible supply of a token
        amount: u64,
    }
}

impl EscrowInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        msg!("Rest: {:#?}", rest);

        Ok(match tag {
            0 => Self::InitEscrow {
                amount: Self::unpack_amount(rest)?,
            },
            1 => Self::Exchange {
                amount: Self::unpack_amount(rest)?
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(amount)
    }

    /// Serializes a EscrowInstruction into a byte buffer.
    pub fn serialize(self: Self, amount: u64) -> Result<Vec<u8>, ProgramError> {
        let mut output = Vec::with_capacity(size_of::<EscrowInstruction>());

        match self {
            Self::InitEscrow{amount:_} => {
                output.resize(size_of::<u64>() + 1, 0);
                output[0] = 0;
            },
            Self::Exchange{amount:_} => {
                output.resize(size_of::<u64>() + 1, 0);
                output[0] = 1;
            }
        }
        #[allow(clippy::cast_ptr_alignment)]
        let value =
            unsafe { &mut *(&mut output[size_of::<u8>()] as *mut u8 as *mut u64) };
        *value = amount;
        Ok(output)
    }
}

// Creates an 'Initialize' instruction.
#[cfg(not(target_arch = "bpf"))]
pub fn initialize(
    program_id: &Pubkey, 
    initializer: &Pubkey, 
    initializer_send_token_account: &Pubkey, 
    initializer_receive_token_account: &Pubkey,
    escrow_account: &Pubkey,
    amount: u64,
) -> Result<Instruction, ProgramError> {
    let data = EscrowInstruction::InitEscrow{amount}.serialize(amount)?;
    let accounts = vec![
        AccountMeta::new(*initializer, true),
        AccountMeta::new(*initializer_send_token_account, false),
        AccountMeta::new_readonly(*initializer_receive_token_account, false),
        AccountMeta::new(*escrow_account, false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}
