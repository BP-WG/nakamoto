use std::net;
use std::path::PathBuf;

use argh::FromArgs;

use nakamoto_common::bitcoin::Address;

use nakamoto_common::bitcoin::util::bip32::DerivationPath;
use nakamoto_common::block::Height;
use nakamoto_wallet::logger;

/// A Bitcoin wallet.
#[derive(FromArgs)]
pub struct Options {
    /// watch the following addresses
    #[argh(option)]
    pub addresses: Vec<Address>,
    /// wallet birth height, from which to start scanning
    #[argh(option)]
    pub birth_height: Height,
    /// connect to this node
    #[argh(option)]
    pub connect: net::SocketAddr,
    /// wallet file
    #[argh(option)]
    pub wallet: PathBuf,
    /// wallet derivation path, eg. m/84'/0'/0'/0.
    #[argh(option)]
    pub hd_path: DerivationPath,
    /// enable debug logging
    #[argh(switch)]
    pub debug: bool,
}

impl Options {
    pub fn from_env() -> Self {
        argh::from_env()
    }
}

fn main() {
    let opts = Options::from_env();

    let level = if opts.debug {
        log::Level::Debug
    } else {
        log::Level::Error
    };
    logger::init(level).expect("initializing logger for the first time");

    if let Err(err) =
        nakamoto_wallet::run(&opts.wallet, opts.birth_height, opts.connect, opts.hd_path)
    {
        log::error!("Fatal: {}", err);
        std::process::exit(1);
    }
}
