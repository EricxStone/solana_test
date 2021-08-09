#![cfg(feature = "test-bpf")]
use std::{println as info, println as warn};

use {
    assert_matches::*,
    solana_program::{
        instruction::{AccountMeta, Instruction}
    },
    solana_sdk::{
        commitment_config::CommitmentConfig,
        account::AccountSharedData,
        native_token::*,
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair, Signer},
        system_instruction,
        transaction::Transaction,
    },
    solana_validator::test_validator::*,
    solana_client::{
        rpc_client::RpcClient, rpc_config::RpcSendTransactionConfig, rpc_request::TokenAccountsFilter,
    },
    solana_account_decoder::{
        parse_token::token_amount_to_ui_amount,
    },
};
use spl_escrow::{
    instruction::{
        EscrowInstruction, EscrowInstruction::*,
    },
    state::*,
};
use spl_token::{
    self,
    instruction::*,
    native_mint,
    state::{Account, Mint},
};
use solana_sdk::program_pack::Pack;

use crate::util::{
    create_token, create_token_account, mint_token, initialize_bridge, create_escrow_account
};

mod util;

type Error = Box<dyn std::error::Error>;
type CommmandResult = Result<Option<Transaction>, Error>;

#[test]
fn test_escrow_initalization() -> Result<(), Error> {
    let program_id = Pubkey::new_unique();
    let alice = Keypair::new();
    let alice_account_data = 
        AccountSharedData::new(10000000000000, 0, &solana_program::system_program::id());

    let (test_validator, payer) = TestValidatorGenesis::default()
        .add_program("spl_escrow", program_id)
        .add_account(alice.pubkey(), alice_account_data)
        .start();
    let (rpc_client, recent_blockhash, _fee_calculator) = test_validator.rpc_client();

    let commitment_config = CommitmentConfig::processed();

    let escrow_account = Keypair::new();
    let create_escrow_account_transaction = create_escrow_account(
        &payer,
        &escrow_account,
        &program_id, 
        &rpc_client,
    )?;
    if let Some(transaction) = create_escrow_account_transaction{
        rpc_client.send_and_confirm_transaction_with_spinner_and_commitment(
            &transaction,
            commitment_config,
        )?;
    }

    let token_x = Keypair::new();
    let token_y = Keypair::new();
    let alice_token_x_account = Keypair::new();
    let bob = Keypair::new();
    let alice_token_y_account = Keypair::new();
    
    let create_token_x_transaction = create_token(&token_x, &payer, &payer, &rpc_client, 8)?;
    if let Some(transaction) = create_token_x_transaction {
        rpc_client.send_and_confirm_transaction_with_spinner_and_commitment(
            &transaction,
            commitment_config,
        )?;
    }

    let create_token_y_transaction = create_token(&token_y, &payer, &payer, &rpc_client, 8)?;
    if let Some(transaction) = create_token_y_transaction {
        rpc_client.send_and_confirm_transaction_with_spinner_and_commitment(
            &transaction,
            commitment_config,
        )?;
    }

    let create_token_x_account_transaction = create_token_account(&token_x.pubkey(), &payer, &alice_token_x_account, &alice, &rpc_client)?;
    if let Some(transaction) = create_token_x_account_transaction {
        rpc_client.send_and_confirm_transaction_with_spinner_and_commitment(
            &transaction,
            commitment_config,
        )?;
    }

    let create_token_y_account_transaction = create_token_account(&token_y.pubkey(), &payer, &alice_token_y_account, &alice, &rpc_client)?;
    if let Some(transaction) = create_token_y_account_transaction {
        rpc_client.send_and_confirm_transaction_with_spinner_and_commitment(
            &transaction,
            commitment_config,
        )?;
    }

    let mint_token_transaction = mint_token(&token_x.pubkey(), &payer, &payer, 100000.0, &alice_token_x_account.pubkey(), &rpc_client)?;
    if let Some(transaction) = mint_token_transaction {
        rpc_client.send_and_confirm_transaction_with_spinner_and_commitment(
            &transaction,
            commitment_config,
        )?;
    }

    let alice_token_account_balance = rpc_client.get_token_account_balance_with_commitment(&alice_token_x_account.pubkey(), commitment_config)?;
    assert_eq!(alice_token_account_balance.value.ui_amount_string, "100000");

    let initialize_bridge_transaction = initialize_bridge(&program_id, &alice, &alice_token_x_account.pubkey(), &alice_token_y_account.pubkey(), &escrow_account.pubkey(), 500, &rpc_client)?;
    if let Some(transaction) = initialize_bridge_transaction {
        rpc_client.send_and_confirm_transaction_with_spinner_and_commitment(
            &transaction,
            commitment_config,
        )?;
    }

    let result = rpc_client.get_account(&escrow_account.pubkey())?;
    let mut escrow_info = Escrow::unpack_unchecked(&result.data)?;
    // Check whether the escrow is initialized
    assert_eq!(escrow_info.is_initialized, true);

    Ok(())
}