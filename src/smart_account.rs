use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::{Transaction, VersionedTransaction};
use squads_multisig::anchor_lang::AccountDeserialize;
use squads_multisig::state::{Member, Permission, Permissions};

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
    let data = get(&gateway, multisig).await?;
    Ok(data)
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
