use cargo_packager_updater::{semver::Version, url::Url, Config};

use crate::error::Error;

pub fn updater() -> Result<(), Error> {
    // releases endpoint
    let endpoint = "https://ore-wallet-xyz.s3.us-east-1.amazonaws.com";
    let endpoint = Url::parse(endpoint)?;
    // signer pubkey
    let pubkey = String::from("dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEE1RkFDQUFCQ0M0NDhBRTQKUldUa2lrVE1xOHI2cGJSaXdCS0NVWGdBQTYzSGFNTXlBRlc5NThYVFhwUEVab29UaGpiSk1WWloK");
    // config
    let config = Config {
        endpoints: vec![endpoint],
        pubkey,
        ..Default::default()
    };
    // current version for reference
    let current_version = env!("CARGO_PKG_VERSION");
    let current_version = Version::parse(current_version)?;
    // check for update
    let update = cargo_packager_updater::check_update(current_version, config)?;
    if let Some(update) = update {
        update.download_and_install()?;
        println!("update installed");
    } else {
        println!("no update available");
    }
    Ok(())
}
