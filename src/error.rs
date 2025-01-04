#[derive(Debug)]
pub enum Error {
    Keyring,
    SolanaClient,
    AnchorDeserialize,
    Ed25519,
}

impl From<keyring::Error> for Error {
    fn from(value: keyring::Error) -> Self {
        println!("{:?}", value);
        Self::Keyring
    }
}

impl From<solana_client::client_error::ClientError> for Error {
    fn from(value: solana_client::client_error::ClientError) -> Self {
        println!("{:?}", value);
        Self::SolanaClient
    }
}

impl From<squads_multisig::anchor_lang::error::Error> for Error {
    fn from(value: squads_multisig::anchor_lang::error::Error) -> Self {
        println!("{:?}", value);
        Self::AnchorDeserialize
    }
}
