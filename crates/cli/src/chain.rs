//! What this program's chain does differently from its sibling's.
//!
//! The dashboard, the commands and the layout are shared with bip110-miner.
//! The handful of facts that are not — the node's name, how long a reward
//! takes to mature, which peers count as being on the right network — live
//! here, so the two projects differ in this file and not all over.

use bitcoind_rpc::Network;
use serde_json::Value;

/// The program's name, as typed and as shown.
pub const PROGRAM: &str = "btc-miner";

/// The node software, short form: `Core 31.1.0`.
pub const NODE_NAME: &str = "Core";

/// What a block reward is paid in.
pub const UNIT: &str = "BTC";

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
