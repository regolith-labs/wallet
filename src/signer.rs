use keyring::Entry;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use solana_sdk::signature::Keypair;

use crate::error::Error;

const SERVICE_MULTISIG: &str = "ore-app-xyz-multisig";
const USER: &str = "ore-user-xyz";

/// embeded keypair on device.
/// field names from sqauds multisig api.
#[derive(Debug, Deserialize, Serialize)]
pub struct Multisig {
    /// signer embeded on this device
    #[serde(serialize_with = "serialize", deserialize_with = "deserialize")]
    pub creator: Keypair,
    /// ephemeral keypair used to seed multisig pda
    /// persisted to derive pda
    #[serde(serialize_with = "serialize", deserialize_with = "deserialize")]
    pub create_key: Keypair,
}

fn set(secret: &[u8]) -> Result<(), Error> {
    let keyring = Entry::new(SERVICE_MULTISIG, USER)?;
    keyring.set_secret(secret).map_err(From::from)
}

fn get() -> Result<Multisig, Error> {
    let keyring = Entry::new(SERVICE_MULTISIG, USER)?;
    let secret = keyring.get_secret()?;
    let multisig = bincode::deserialize(secret.as_slice()).map_err(|err| {
        println!("{:?}", err);
        Error::BincodeDeserialize
    })?;
    Ok(multisig)
}

pub fn get_or_set() -> Result<Multisig, Error> {
    match get() {
        ok @ Ok(_) => ok,
        Err(_err) => {
            let creator = Keypair::new();
            let create_key = Keypair::new();
            let multisig = Multisig {
                creator,
                create_key,
            };
            let bytes = bincode::serialize(&multisig).map_err(|err| {
                println!("{:?}", err);
                Error::BincodeSerialize
            })?;
            set(bytes.as_slice())?;
            Ok(multisig)
        }
    }
}

fn serialize<S: Serializer>(keypair: &Keypair, serializer: S) -> Result<S::Ok, S::Error> {
    let bytes = keypair.to_bytes();
    serializer.serialize_bytes(&bytes)
}

fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Keypair, D::Error> {
    let bytes: &[u8] = <&[u8]>::deserialize(deserializer)?;
    Keypair::from_bytes(bytes).map_err(|_| serde::de::Error::custom("invalid keypair bytes"))
}
