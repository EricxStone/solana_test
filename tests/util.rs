use {
    assert_matches::*,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        system_instruction::{create_account, SystemInstruction},
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
    }
};
use spl_escrow::{
    instruction::{
        EscrowInstruction, EscrowInstruction::*, initialize,
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

type Error = Box<dyn std::error::Error>;
type CommmandResult = Result<Option<Transaction>, Error>;


pub fn create_token(
    token: &Keypair,
    owner: &Keypair,
    fee_payer: &Keypair, 
    rpc_client: &RpcClient, 
    decimals: u8,
) -> CommmandResult {
    println!("Creating token {}", token.pubkey());

    let minimum_balance_for_rent_exemption = rpc_client
        .get_minimum_balance_for_rent_exemption(Mint::LEN)?;

    let mut transaction = Transaction::new_with_payer(
        &[
            solana_sdk::system_instruction::create_account(
                &fee_payer.pubkey(),
                &token.pubkey(),
                minimum_balance_for_rent_exemption,
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            initialize_mint(
                &spl_token::id(),
                &token.pubkey(),
                &owner.pubkey(),
                None,
                decimals,
            )?,
        ],
        Some(&fee_payer.pubkey()),
    );

    let (recent_blockhash, fee_calculator) = rpc_client.get_recent_blockhash()?;
    check_fee_payer_balance(
        &fee_payer.pubkey(), rpc_client,
        minimum_balance_for_rent_exemption
            + fee_calculator.calculate_fee(&transaction.message()),
    )?;
    transaction.sign(
        &[fee_payer, token],
        recent_blockhash,
    );

    Ok(Some(transaction))
}

pub fn create_escrow_account(
    fee_payer: &Keypair,
    escrow_account: &Keypair,
    owner: &Pubkey,
    rpc_client: &RpcClient, 
) -> CommmandResult {
    println!("Creating escrow account {}", escrow_account.pubkey());

    let minimum_balance_for_rent_exemption = rpc_client
        .get_minimum_balance_for_rent_exemption(Escrow::LEN)?;

    let mut transaction = Transaction::new_with_payer(
        &[
            create_account(
                &fee_payer.pubkey(), 
                &escrow_account.pubkey(),
                minimum_balance_for_rent_exemption,
                Escrow::LEN as u64,
                &owner
            ),
        ],
        Some(&fee_payer.pubkey()),
    );
    let (recent_blockhash, fee_calculator) = rpc_client.get_recent_blockhash()?;
    check_fee_payer_balance(
        &fee_payer.pubkey(), rpc_client,
        minimum_balance_for_rent_exemption
            + fee_calculator.calculate_fee(&transaction.message()),
    )?;
    transaction.sign(
        &[fee_payer, escrow_account],
        recent_blockhash,
    );

    Ok(Some(transaction))
}

pub fn create_token_account(
    token: &Pubkey, 
    fee_payer: &Keypair, 
    account: &Keypair, 
    owner: &Keypair, 
    rpc_client: &RpcClient,
) -> CommmandResult {
    println!("Creating account {}", account.pubkey());

    let minimum_balance_for_rent_exemption = rpc_client
        .get_minimum_balance_for_rent_exemption(Account::LEN)?;

    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &fee_payer.pubkey(),
                &account.pubkey(),
                minimum_balance_for_rent_exemption,
                Account::LEN as u64,
                &spl_token::id(),
            ),
            initialize_account(
                &spl_token::id(),
                &account.pubkey(),
                &token,
                &owner.pubkey(),
            )?,
        ],
        Some(&fee_payer.pubkey()),
    );

    let (recent_blockhash, fee_calculator) = rpc_client.get_recent_blockhash()?;
    check_fee_payer_balance(
        &fee_payer.pubkey(), rpc_client,
        minimum_balance_for_rent_exemption
            + fee_calculator.calculate_fee(&transaction.message()),
    )?;
    transaction.sign(
        &[fee_payer, account],
        recent_blockhash,
    );
    Ok(Some(transaction))
}

pub fn mint_token(
    token: &Pubkey,
    owner: &Keypair,
    payer: &Keypair,
    ui_amount: f64,
    recipient: &Pubkey,
    rpc_client: &RpcClient,
) -> CommmandResult {
    println!(
        "Minting {} tokens\n  Token: {}\n  Recipient: {}",
        ui_amount, token, recipient
    );

    let commitment_config = CommitmentConfig::processed();
    let recipient_token_balance = rpc_client
        .get_token_account_balance_with_commitment(&recipient, commitment_config)?
        .value;
    let amount = spl_token::ui_amount_to_amount(ui_amount, recipient_token_balance.decimals);

    let mut transaction = Transaction::new_with_payer(
        &[mint_to(
            &spl_token::id(),
            &token,
            &recipient,
            &payer.pubkey(),
            &[],
            amount,
        )?],
        Some(&payer.pubkey()),
    );

    let (recent_blockhash, fee_calculator) = rpc_client.get_recent_blockhash()?;
    check_fee_payer_balance(
        &payer.pubkey(), rpc_client,
        fee_calculator.calculate_fee(&transaction.message()),
    )?;
    transaction.sign(&[payer, owner], recent_blockhash);
    Ok(Some(transaction))
}

pub fn initialize_bridge(
    program_id: &Pubkey,
    initializer: &Keypair, 
    initializer_send_token_account: &Pubkey, 
    initializer_receive_token_account: &Pubkey, 
    escrow_account: &Pubkey,
    amount: u64,
    rpc_client: &RpcClient,
) -> CommmandResult {
    let ix = initialize(
        &program_id, 
        &initializer.pubkey(),
        &initializer_send_token_account, 
        &initializer_receive_token_account, 
        &escrow_account,
        amount
    )?;
    let mut transaction = Transaction::new_with_payer(&[ix], Some(&initializer.pubkey()));
    let (recent_blockhash, fee_calculator) = rpc_client.get_recent_blockhash()?;
    check_fee_payer_balance(
        &initializer.pubkey(),
        rpc_client,
        fee_calculator.calculate_fee(&transaction.message()),
    )?;
    transaction.sign(&[initializer], recent_blockhash);
    Ok(Some(transaction))
}

fn check_fee_payer_balance(fee_payer: &Pubkey, rpc_client: &RpcClient, required_balance: u64) -> Result<(), Error> {
    let balance = rpc_client.get_balance(&fee_payer)?;
    if balance < required_balance {
        Err(format!(
            "Fee payer, {}, has insufficient balance: {} required, {} available",
            fee_payer,
            lamports_to_sol(required_balance),
            lamports_to_sol(balance)
        )
            .into())
    } else {
        Ok(())
    }
}