#[derive(Debug)]
pub enum Error {
    Keyring(keyring::Error),
    Ed25519,
}

impl From<keyring::Error> for Error {
    fn from(value: keyring::Error) -> Self {
        Self::Keyring(value)
    }
}
