//! The pool on its own.
//!
//! ```text
//! pool [--network regtest] [--listen 127.0.0.1:3333] [--address bc1...]
//! ```
//!
//! For pointing other mining hardware — a Bitaxe, say — at this machine.
//! Everything the pool does is in the library; this only reads arguments and
//! prints what it reports.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use bitcoind_rpc::Network;
use events::Plain;
use pool::{Options, Reported};

fn main() {
    let result = parse_args()
        .and_then(|options| pool::run(&options, Arc::new(Plain), Arc::new(AtomicBool::new(false))));

    if let Err(error) = result {
        // Already printed by the pool itself, in its own words.
        if error.downcast_ref::<Reported>().is_none() {
            eprintln!("error: {error}");
            let mut source = error.source();
            while let Some(cause) = source {
                eprintln!("  caused by: {cause}");
                source = cause.source();
            }
        }
        std::process::exit(1);
    }
}

fn parse_args() -> Result<Options, pool::Error> {
    let mut network = Network::Regtest;
    let mut listen = "127.0.0.1:3333".to_owned();
    let mut address = None;

    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{flag} needs a value"));

        match flag.as_str() {
            "--network" => {
                let name = value()?;
                network = Network::parse(&name).ok_or_else(|| format!("unknown network {name:?}"))?;
            }
            "--listen" => listen = value()?,
            "--address" => address = Some(value()?),
            "--help" | "-h" => {
                println!("pool [--network regtest|testnet4|mainnet] [--listen ADDR] [--address ADDR]");
                std::process::exit(0);
            }
            other => return Err(format!("unknown option {other:?}").into()),
        }
    }

    Ok(Options {
        network,
        listen: listen.parse()?,
        address,
        datadir: std::env::var("SOLO_DATADIR").map_or_else(
            |_| PathBuf::from(std::env::var("HOME").expect("HOME is set")).join(".bitcoin-solo"),
            PathBuf::from,
        ),
        rpc_port: None,
    })
}
