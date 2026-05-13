
use anchor_lang::{
    solana_program::instruction::Instruction,
    system_program::ID as SYSTEM_PROGRAM_ID,
    InstructionData, ToAccountMetas,
};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;

fn setup() -> (LiteSVM, Keypair) {
    let program_id = anchor_vault::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/anchor_vault.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    (svm, payer)
}

fn send_instruction(svm: &mut LiteSVM, payer: &Keypair, instruction: Instruction) {
    let message = Message::new(&[instruction], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&[payer], message, recent_blockhash);
    svm.send_transaction(transaction).unwrap();
}

#[test]
fn test_initialize_deposit_withdraw_close() {
    let (mut svm, payer) = setup();
    let user = payer.pubkey();
    let deposit_amount = 1_000_000;
    let withdraw_amount = 400_000;
    let expected_balance_after_withdraw = deposit_amount - withdraw_amount;

    let (vault_state_pda, _state_bump) =
        Pubkey::find_program_address(&[b"state", user.as_ref()], &anchor_vault::id());

    let (vault_pda, _vault_bump) =
        Pubkey::find_program_address(&[b"vault", vault_state_pda.as_ref()], &anchor_vault::id());

    println!("User wallet address: {}", user);
    println!("Vault state PDA: {}", vault_state_pda);
    println!("Vault PDA: {}", vault_pda);
    println!("Deposit amount: {} lamports", deposit_amount);
    println!("Withdraw amount: {} lamports", withdraw_amount);

    let init_ix = Instruction {
        program_id: anchor_vault::id(),
        accounts: anchor_vault::accounts::Initialize {
            user,
            vault_state: vault_state_pda,
            vault: vault_pda,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: anchor_vault::instruction::Initialize {}.data(),
    };
    send_instruction(&mut svm, &payer, init_ix);

    let _vault_state_account = svm.get_account(&vault_state_pda).unwrap();
    let initial_vault_balance = svm
        .get_account(&vault_pda)
        .map(|account| account.lamports)
        .unwrap_or_default();

    println!("Initialize complete");
    println!("Current vault balance: {} lamports", initial_vault_balance);
    assert_eq!(
        initial_vault_balance, 0,
        "vault should start with zero lamports"
    );

    let deposit_ix = Instruction {
        program_id: anchor_vault::id(),
        accounts: anchor_vault::accounts::Deposit {
            user,
            vault: vault_pda,
            vault_state: vault_state_pda,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: anchor_vault::instruction::Deposit {
            amount: deposit_amount,
        }
        .data(),
    };
    send_instruction(&mut svm, &payer, deposit_ix);

    let vault_balance_after_deposit = svm.get_account(&vault_pda).unwrap().lamports;
    println!("Deposit complete");
    println!("Sent to vault: {} lamports", deposit_amount);
    println!(
        "Available vault balance after deposit: {} lamports",
        vault_balance_after_deposit
    );
    assert_eq!(
        vault_balance_after_deposit, deposit_amount,
        "vault balance should equal the deposited amount"
    );

    let withdraw_ix = Instruction {
        program_id: anchor_vault::id(),
        accounts: anchor_vault::accounts::Withdraw {
            user,
            vault: vault_pda,
            vault_state: vault_state_pda,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: anchor_vault::instruction::Withdraw {
            amount: withdraw_amount,
        }
        .data(),
    };
    send_instruction(&mut svm, &payer, withdraw_ix);

    let vault_balance_after_withdraw = svm.get_account(&vault_pda).unwrap().lamports;
    println!("Withdraw complete");
    println!("Withdrawn from vault: {} lamports", withdraw_amount);
    println!(
        "Available vault balance after withdraw: {} lamports",
        vault_balance_after_withdraw
    );
    assert_eq!(
        vault_balance_after_withdraw, expected_balance_after_withdraw,
        "vault balance should equal deposit amount minus withdrawn amount"
    );

    let close_ix = Instruction {
        program_id: anchor_vault::id(),
        accounts: anchor_vault::accounts::Close {
            user,
            vault: vault_pda,
            vault_state: vault_state_pda,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: anchor_vault::instruction::Close {}.data(),
    };
    send_instruction(&mut svm, &payer, close_ix);

    let vault_balance_after_close = svm
        .get_account(&vault_pda)
        .map(|account| account.lamports)
        .unwrap_or_default();
    println!("Close complete");
    println!(
        "Remaining vault balance returned to user: {} lamports",
        expected_balance_after_withdraw
    );
    println!(
        "Available vault balance after close: {} lamports",
        vault_balance_after_close
    );
    assert_eq!(
        vault_balance_after_close, 0,
        "vault balance should be zero after close"
    );
}
