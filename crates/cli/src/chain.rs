//! What this program's chain does differently from its sibling's.
//!
//! The dashboard, the commands and the layout are shared with bip110-miner.
//! The handful of facts that are not — the node's name, how long a reward
//! takes to mature, which peers count as being on the right network — live
//! here, so the two projects differ in this file and not all over.

use bitcoind_rpc::{Network, RpcClient};
use serde_json::Value;

use crate::setup::address::Way;

/// The program's name, as typed and as shown.
pub const PROGRAM: &str = "btc-miner";

/// The node software, short form: `Core 31.1.0`.
pub const NODE_NAME: &str = "Core";

/// What a block reward is paid in.
pub const UNIT: &str = "BTC";

/// What setup says before asking where rewards go, a paragraph each.
pub const ADDRESS_INTRO: &[&str] = &[
    "If your computer ever finds a block, the reward goes to an address you own.",
    "Any Bitcoin wallet can give you one: a phone app, a hardware wallet, or a wallet on this \
     node. Use one whose backup you already keep safe.",
];

/// The ways to get a reward address, in the order offered, recommended first.
pub const ADDRESS_WAYS: [(Way, &str); 2] = [
    (Way::Paste, "Paste an address from my wallet   (recommended)"),
    (Way::NewWallet, "Create a wallet on this node, protected by a passphrase"),
];

/// What a wallet made by setup is called.
pub const WALLET_NAME: &str = "btc-rewards";

/// Roughly how much disk the node needs on `network`: the blocks it keeps,
/// the coin database, and room for the quick-start snapshot while it loads.
pub const fn disk_needed(network: Network) -> u64 {
    match network {
        Network::Mainnet => 27_000_000_000,
        Network::Testnet4 => 6_000_000_000,
        Network::Regtest => 1_000_000_000,
    }
}

/// A UTXO snapshot the node can start from.
pub struct Snapshot {
    /// The block it describes.
    pub height: u32,
    /// Where to download it.
    pub url: &'static str,
    /// Its file name.
    pub file: &'static str,
    /// Its size.
    pub bytes: u64,
}

/// The quick-start snapshot for `network`, where there is one.
///
/// Bitcoin Core carries this snapshot's hash. The mirror is a convenience
/// only: the node checks the file against that hash and refuses anything else.
pub const fn snapshot(network: Network) -> Option<Snapshot> {
    match network {
        Network::Mainnet => Some(Snapshot {
            height: 910_000,
            url: "https://files-vps02.jaonoctus.dev/utxo-910000.dat",
            file: "utxo-910000.dat",
            bytes: 9_637_809_744,
        }),
        Network::Testnet4 | Network::Regtest => None,
    }
}

/// Readies a regtest chain to be mined through the pool. Bitcoin's pool can
/// mine a chain from its very first block, so there is nothing to do.
pub const fn prepare_regtest(_client: &RpcClient) -> Result<Option<String>, String> {
    Ok(None)
}

/// Blocks a coinbase must wait before it can be spent: 100, on every network.
pub const fn maturity(_network: Network, _height: u32) -> u32 {
    100
}

/// The maturity wait in words, for the line under the reward address.
///
/// A hundred blocks is about 17 hours where blocks come every ten minutes.
/// Regtest blocks come whenever they are mined, so there it is just a count.
pub const fn maturity_words(network: Network, _height: u32) -> &'static str {
    match network {
        Network::Regtest => "100 blocks",
        Network::Mainnet | Network::Testnet4 => "about 17 hours",
    }
}

/// How many peers are on this chain, where that can be told apart.
///
/// Every peer of a Bitcoin node is on Bitcoin, so there is nothing to count
/// separately.
pub const fn chain_peers(_peers: &[Value]) -> Option<usize> {
    None
}

/// The label for [`chain_peers`], were there any to show.
pub const CHAIN_LABEL: &str = "Bitcoin";
