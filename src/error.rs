#[derive(Debug)]
pub enum Error {
    AnchorDeserialize,
    BincodeDeserialize,
    BincodeSerialize,
    Keyring,
    SolanaClient,
    SquadsCompileTransaction,
    SquadsClient,
    UpdaterFetch,
    UpdaterParseUrl,
    SemverParse,
}

impl From<cargo_packager_updater::Error> for Error {
    fn from(value: cargo_packager_updater::Error) -> Self {
        println!("{:?}", value);
        Self::UpdaterFetch
    }
}

impl From<cargo_packager_updater::semver::Error> for Error {
    fn from(value: cargo_packager_updater::semver::Error) -> Self {
        println!("{:?}", value);
        Self::SemverParse
    }
}

impl From<cargo_packager_updater::url::ParseError> for Error {
    fn from(value: cargo_packager_updater::url::ParseError) -> Self {
        println!("{:?}", value);
        Self::UpdaterParseUrl
    }
}

impl From<squads_multisig::error::ClientError> for Error {
    fn from(value: squads_multisig::error::ClientError) -> Self {
        println!("{:?}", value);
        Self::SquadsClient
    }
}

impl From<squads_multisig::solana_program::message::CompileError> for Error {
    fn from(value: squads_multisig::solana_program::message::CompileError) -> Self {
        println!("{:?}", value);
        Self::SquadsCompileTransaction
    }
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
