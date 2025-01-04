use std::fmt::Display;

use keyring::Entry;
use solana_sdk::signature::Keypair;

use crate::error::Error;

const SERVICE_NAME: &str = "ore-app-xyzz";
const USER: &str = "ore-user-xyzz";

fn set(secret: &[u8]) -> Result<(), Error> {
    let keyring = Entry::new(SERVICE_NAME, USER)?;
    keyring.set_secret(secret).map_err(From::from)
}

fn get() -> Result<PrettyKeypair, Error> {
    let keyring = Entry::new(SERVICE_NAME, USER)?;
    let secret = keyring.get_secret()?;
    let keypair = Keypair::from_bytes(secret.as_slice()).map_err(|err| {
        println!("{:?}", err);
        crate::error::Error::Ed25519
    })?;
    Ok(PrettyKeypair(keypair))
}

pub fn get_or_set() -> Result<PrettyKeypair, Error> {
    match get() {
        ok @ Ok(_) => ok,
        Err(err) => {
            let keypair = Keypair::new();
            set(keypair.to_bytes().as_slice())?;
            Ok(PrettyKeypair(keypair))
        }
    }
}

pub struct PrettyKeypair(pub Keypair);

impl Display for PrettyKeypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
