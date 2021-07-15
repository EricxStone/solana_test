#![cfg(feature = "test-bpf")]
use std::{println as info, println as warn};

use {
    assert_matches::*,
    solana_program::{
        instruction::{AccountMeta, Instruction}
    },
    solana_sdk::{
        commitment_config::CommitmentConfig,
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
    create_token, create_token_account, mint_token,
};

mod util;

type Error = Box<dyn std::error::Error>;
type CommmandResult = Result<Option<Transaction>, Error>;

#[test]
fn test_escrow_initalization() {
    let program_id = Pubkey::new_unique();

    let (test_validator, payer) = TestValidatorGenesis::default()
        .add_program("spl_escrow", program_id)
        .start();
    let (rpc_client, recent_blockhash, _fee_calculator) = test_validator.rpc_client();

    let token_x = Keypair::new();
    let alice = Keypair::new();
    let alice_token_x_account = Keypair::new();
    let bob = Keypair::new();
    let alice_token_y = Keypair::new();
    
    match create_token(&token_x, &payer, &payer, &rpc_client, 8){
        Ok(transaction) => {
            if let Some(transaction) = transaction {
                rpc_client.send_and_confirm_transaction(&transaction);
            }
        },
        Err(e) =>{
            println!("Error: {}", e);
        }
    }
    match create_token_account(&token_x.pubkey(), &payer, &alice_token_x_account, &alice, &rpc_client){
        Ok(transaction) => {
            if let Some(transaction) = transaction {
                rpc_client.send_and_confirm_transaction(&transaction);
            }
        },
        Err(e) =>{
            println!("Error: {}", e);
        }
    }
    match mint_token(&token_x.pubkey(), &payer, &payer, 100000.0, &alice_token_x_account.pubkey(), &rpc_client){
        Ok(transaction) => {
            if let Some(transaction) = transaction {
                rpc_client.send_and_confirm_transaction(&transaction);
            }
        },
        Err(e) =>{
            println!("Error: {}", e);
        }
    }

    let commitment_config = CommitmentConfig::processed();
    match rpc_client.get_token_account_balance_with_commitment(&alice_token_x_account.pubkey(), commitment_config) {
        Ok(bal) => {
            assert_eq!(bal.value.ui_amount_string, "100000");
        },
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}