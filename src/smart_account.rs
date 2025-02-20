use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use squads_multisig::anchor_lang::AccountDeserialize;
use squads_multisig::state::{Member, Permission, Permissions, TransactionMessage};
use squads_multisig::vault_transaction::VaultTransactionMessageExt;

use crate::error::Error;
use crate::gateway::Gateway;
use crate::signer::Multisig;

pub async fn get_or_create(multisig: &Multisig) -> Result<squads_multisig::state::Multisig, Error> {
    let gateway = Gateway::new();
    // look for existing onchain account
    if let Ok(data) = get(&gateway, multisig).await {
        return Ok(data);
    }
    // or create new one
    create(&gateway, multisig).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    let data = get(&gateway, multisig).await?;
    Ok(data)
}

pub fn vault(multisig: &Multisig) -> Pubkey {
    let (multisig_pda, _) =
        squads_multisig::pda::get_multisig_pda(&multisig.create_key.pubkey(), None);
    let (vault_pda, _) = squads_multisig::pda::get_vault_pda(&multisig_pda, 0, None);
    vault_pda
}

pub async fn dummy(multisig: &Multisig) -> Result<(), Error> {
    let vault_pda = vault(multisig);
    let ix = solana_sdk::system_instruction::transfer(&vault_pda, &multisig.creator.pubkey(), 1);
    transaction(multisig, vec![ix]).await
}

pub async fn transaction(multisig: &Multisig, instructions: Vec<Instruction>) -> Result<(), Error> {
    let gateway = Gateway::new();
    // derive pdas
    let mut proposal_instructions = vec![];
    let (multisig_pda, _) =
        squads_multisig::pda::get_multisig_pda(&multisig.create_key.pubkey(), None);
    let multisig_pda_data = get(&gateway, multisig).await?;
    let transaction_index = multisig_pda_data.transaction_index + 1;
    let (proposal_pda, _) =
        squads_multisig::pda::get_proposal_pda(&multisig_pda, transaction_index, None);
    let (transaction_pda, _) =
        squads_multisig::pda::get_transaction_pda(&multisig_pda, transaction_index, None);
    // create proposal
    let (create_proposal_instructions, transaction_message) = create_proposal(
        multisig,
        instructions,
        multisig_pda,
        proposal_pda,
        transaction_pda,
        transaction_index,
    )?;
    // approval proposal
    let approve_proposal_instruction = approve_proposal(multisig, multisig_pda, proposal_pda);
    // execute proposal
    let execute_proposal_instruction = execute_proposal(
        multisig,
        multisig_pda,
        proposal_pda,
        transaction_pda,
        &transaction_message,
    )?;
    // close proposal
    let close_proposal_instruction =
        close_proposal(multisig, multisig_pda, proposal_pda, transaction_pda);
    // transaction
    for ix in create_proposal_instructions.into_iter() {
        proposal_instructions.push(ix);
    }
    proposal_instructions.push(approve_proposal_instruction);
    proposal_instructions.push(execute_proposal_instruction);
    proposal_instructions.push(close_proposal_instruction);
    let mut tx = Transaction::new_with_payer(
        proposal_instructions.as_slice(),
        Some(&multisig.creator.pubkey()),
    );
    let hash = gateway.rpc_client.get_latest_blockhash().await?;
    tx.sign(&[&multisig.creator], hash);
    let _sig = gateway.rpc_client.send_transaction(&tx).await?;
    Ok(())
}

fn close_proposal(
    multisig: &Multisig,
    multisig_pda: Pubkey,
    proposal_pda: Pubkey,
    transaction_pda: Pubkey,
) -> Instruction {
    let accounts = squads_multisig::client::VaultTransactionAccountsCloseAccounts {
        multisig: multisig_pda,
        proposal: proposal_pda,
        transaction: transaction_pda,
        rent_collector: multisig.creator.pubkey(),
        system_program: solana_sdk::system_program::ID,
    };
    squads_multisig::client::vault_transaction_accounts_close(accounts, None)
}

fn execute_proposal(
    multisig: &Multisig,
    multisig_pda: Pubkey,
    proposal_pda: Pubkey,
    transaction_pda: Pubkey,
    transaction_message: &TransactionMessage,
) -> Result<Instruction, Error> {
    let accounts = squads_multisig::client::VaultTransactionExecuteAccounts {
        multisig: multisig_pda,
        proposal: proposal_pda,
        transaction: transaction_pda,
        member: multisig.creator.pubkey(),
    };
    let ix = squads_multisig::client::vault_transaction_execute(
        accounts,
        0, // default vault index
        0, // no ephemeral signers
        transaction_message,
        &[],
        None,
    )?;
    Ok(ix)
}

fn approve_proposal(
    multisig: &Multisig,
    multisig_pda: Pubkey,
    proposal_pda: Pubkey,
) -> Instruction {
    let accounts = squads_multisig::client::ProposalVoteAccounts {
        multisig: multisig_pda,
        member: multisig.creator.pubkey(),
        proposal: proposal_pda,
    };
    let args = squads_multisig::client::ProposalVoteArgs { memo: None };
    squads_multisig::client::proposal_approve(accounts, args, None)
}

fn create_proposal(
    multisig: &Multisig,
    instructions: Vec<Instruction>,
    multisig_pda: Pubkey,
    proposal_pda: Pubkey,
    transaction_pda: Pubkey,
    transaction_index: u64,
) -> Result<(Vec<Instruction>, TransactionMessage), Error> {
    // build vault transaction
    let (vault_pda, _) = squads_multisig::pda::get_vault_pda(&multisig_pda, 0, None);
    let create_vault_accounts = squads_multisig::client::VaultTransactionCreateAccounts {
        multisig: multisig_pda,
        transaction: transaction_pda,
        creator: multisig.creator.pubkey(),
        rent_payer: multisig.creator.pubkey(),
        system_program: solana_sdk::system_program::ID,
    };
    let transaction_message = squads_multisig::state::TransactionMessage::try_compile(
        &vault_pda,
        instructions.as_slice(),
        &[],
    )?;
    let create_vault_ix = squads_multisig::client::vault_transaction_create(
        create_vault_accounts,
        0, // default vault index
        0, // no ephemeral signers
        &transaction_message,
        None,
        None,
    );
    // build proposal transaction
    let create_proposal_accounts = squads_multisig::client::ProposalCreateAccounts {
        multisig: multisig_pda,
        proposal: proposal_pda,
        creator: multisig.creator.pubkey(),
        rent_payer: multisig.creator.pubkey(),
        system_program: solana_sdk::system_program::ID,
    };
    let create_proposal_args = squads_multisig::client::ProposalCreateArgs {
        transaction_index,
        draft: false,
    };
    let create_propsal_ix = squads_multisig::client::proposal_create(
        create_proposal_accounts,
        create_proposal_args,
        None,
    );
    Ok((
        vec![create_vault_ix, create_propsal_ix],
        transaction_message,
    ))
}

async fn get(
    gateway: &Gateway,
    multisig: &Multisig,
) -> Result<squads_multisig::state::Multisig, Error> {
    let (multisig_pda, _) =
        squads_multisig::pda::get_multisig_pda(&multisig.create_key.pubkey(), None);
    let data = gateway.rpc_client.get_account_data(&multisig_pda).await?;
    let data = squads_multisig::state::Multisig::try_deserialize(&mut data.as_slice())?;
    Ok(data)
}

async fn create(gateway: &Gateway, multisig: &Multisig) -> Result<(), Error> {
    // program config
    let (program_config_pda, _) = squads_multisig::pda::get_program_config_pda(None);
    let program_config_data = gateway
        .rpc_client
        .get_account_data(&program_config_pda)
        .await?;
    let program_config =
        squads_multisig::squads_multisig_program::state::ProgramConfig::try_deserialize(
            &mut program_config_data.as_slice(),
        )?;
    // treasury
    let treasury = program_config.treasury;
    // multisig
    let (multisig_pda, _) =
        squads_multisig::pda::get_multisig_pda(&multisig.create_key.pubkey(), None);
    // create accounts
    let accounts = squads_multisig::client::MultisigCreateAccountsV2 {
        program_config: program_config_pda,
        treasury,
        multisig: multisig_pda,
        create_key: multisig.create_key.pubkey(),
        creator: multisig.creator.pubkey(),
        system_program: solana_sdk::system_program::ID,
    };
    // args
    let args = squads_multisig::client::MultisigCreateArgsV2 {
        config_authority: None,
        threshold: 1,
        members: vec![Member {
            key: multisig.creator.pubkey(),
            permissions: Permissions::from_vec(
                (vec![Permission::Initiate, Permission::Vote, Permission::Execute]).as_slice(),
            ),
        }],
        time_lock: 0,
        rent_collector: Some(multisig.creator.pubkey()),
        memo: None,
    };
    // instruction
    let ix = squads_multisig::client::multisig_create_v2(accounts, args, None);
    // transaction
    let mut tx = Transaction::new_with_payer(&[ix], Some(&multisig.creator.pubkey()));
    let hash = gateway.rpc_client.get_latest_blockhash().await?;
    tx.sign(&[&multisig.creator, &multisig.create_key], hash);
    // submit
    let sig = gateway.rpc_client.send_transaction(&tx).await?;
    println!("{:?}", sig);
    Ok(())
}
