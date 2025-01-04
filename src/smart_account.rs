use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::{Transaction, VersionedTransaction};
use squads_multisig::anchor_lang::AccountDeserialize;
use squads_multisig::state::{Member, Permission, Permissions};

use crate::error::Error;
use crate::gateway::Gateway;

pub async fn create_smart_account(signer: Keypair) -> Result<(), Error> {
    let gateway = Gateway::new();
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
    // ephemeral create key
    let create_key = Keypair::new();
    // multisig
    let (multisig_pda, _) = squads_multisig::pda::get_multisig_pda(&create_key.pubkey(), None);
    // create accounts
    let accounts = squads_multisig::client::MultisigCreateAccountsV2 {
        program_config: program_config_pda,
        treasury,
        multisig: multisig_pda,
        create_key: create_key.pubkey(),
        creator: signer.pubkey(),
        system_program: solana_sdk::system_program::ID,
    };
    // args
    let args = squads_multisig::client::MultisigCreateArgsV2 {
        config_authority: None,
        threshold: 1,
        members: vec![Member {
            key: signer.pubkey(),
            permissions: Permissions::from_vec(
                (vec![Permission::Initiate, Permission::Vote, Permission::Execute]).as_slice(),
            ),
        }],
        time_lock: 0,
        rent_collector: Some(signer.pubkey()),
        memo: None,
    };
    // instruction
    let ix = squads_multisig::client::multisig_create_v2(accounts, args, None);
    // transaction
    let mut tx = Transaction::new_with_payer(&[ix], Some(&signer.pubkey()));
    let hash = gateway.rpc_client.get_latest_blockhash().await?;
    tx.sign(&[&signer, &create_key], hash);
    // submit
    let sig = gateway.rpc_client.send_transaction(&tx).await?;
    println!("{:?}", sig);
    Ok(())
}
