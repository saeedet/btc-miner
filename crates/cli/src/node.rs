//! The node: finding it, starting and stopping it, and checking it is new
//! enough to mine with.
//!
//! The node is a separate long-lived program, Bitcoin Core. It deliberately
//! outlives a mining session — stopping it would drop its peers and let it fall
//! behind, so the next start would pay to catch up — which is why mining stops
//! on Ctrl-C but the node only stops on `btc-miner stop`.

use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use bitcoind_rpc::{Network, RpcClient};

use crate::config::{Settings, network_key};
use crate::platform;

/// The oldest Bitcoin Core this program will start: 28.0.
///
/// 28.0 is the first release that knows testnet4, and the configuration this
/// program writes names it. An older node refuses that configuration outright,
/// so it is better to say why up front than to pass on its error.
pub const MIN_CORE: (u32, u32, u32) = (28, 0, 0);

/// The minimum under the name shared code uses, whichever node this is.
pub use self::MIN_CORE as MIN_VERSION;

/// The node's configuration for each network, compiled in so an installed
/// binary does not depend on the source tree.
fn template(network: Network) -> &'static str {
    match network {
        Network::Mainnet => include_str!("../../../config/bitcoin.mainnet.conf"),
        Network::Testnet4 => include_str!("../../../config/bitcoin.testnet4.conf"),
        Network::Regtest => include_str!("../../../config/bitcoin.regtest.conf"),
    }
}

/// The RPC port the node's configuration sets, if it sets one.
pub fn configured_rpc_port(network: Network) -> Option<u16> {
    template(network)
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("rpcport="))
        .and_then(|port| port.trim().parse().ok())
}

/// Where the node's configuration file is written.
pub fn conf_path(network: Network) -> PathBuf {
    platform::state_dir().join("node").join(format!("{}.conf", network_key(network)))
}

/// Writes the node's configuration file, replacing any older copy.
///
/// The file belongs to this program: a newer release may carry different
/// settings, and they should take effect.
fn write_conf(network: Network) -> Result<PathBuf, String> {
    let path = conf_path(network);
    let wanted = template(network);
    if std::fs::read_to_string(&path).ok().as_deref() != Some(wanted) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
        }
        std::fs::write(&path, wanted)
            .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    }
    Ok(path)
}

/// An RPC client for the node, if it is running.
pub fn client(settings: &Settings) -> Result<RpcClient, String> {
    RpcClient::connect(&settings.datadir, settings.network, settings.rpc_port)
        .map_err(|error| error.to_string())
}

/// Whether the node is up and answering.
pub fn is_running(settings: &Settings) -> bool {
    client(settings)
        .and_then(|client| client.get_blockchain_info().map_err(|error| error.to_string()))
        .is_ok()
}

/// The installed Bitcoin Core version, read from the binary itself.
pub fn installed_version(settings: &Settings) -> Result<(u32, u32, u32), String> {
    let daemon = settings.binaries.join(platform::node_daemon());
    let output = Command::new(&daemon)
        .arg("-version")
        .output()
        .map_err(|_| format!("Bitcoin Core is not installed at {}", settings.binaries.display()))?;
    let text = String::from_utf8_lossy(&output.stdout);
    parse_version(&text).ok_or_else(|| format!("cannot tell which version {} is", daemon.display()))
}

/// Pulls `31.1.0` out of `Bitcoin Core daemon version v31.1.0 bitcoind`.
fn parse_version(text: &str) -> Option<(u32, u32, u32)> {
    // A `v` followed by a digit — not merely a `v`, or the word "version"
    // earlier on the same line would match first.
    let after = text
        .split_whitespace()
        .filter_map(|word| word.strip_prefix('v'))
        .find(|rest| rest.starts_with(|c: char| c.is_ascii_digit()))?;
    let mut numbers = after.split(['.', '-']).map(|part| part.parse::<u32>().ok());
    Some((numbers.next()??, numbers.next()??, numbers.next().flatten().unwrap_or(0)))
}

/// The version of the node that is actually running, from its user agent.
///
/// Distinct from [`installed_version`]: a node started earlier keeps running
/// the old code after a newer one is installed.
pub fn running_version(client: &RpcClient) -> Option<(u32, u32, u32)> {
    let info: serde_json::Value = client.call("getnetworkinfo", serde_json::json!([])).ok()?;
    let agent = info.get("subversion")?.as_str()?;
    // "/Satoshi:31.1.0/"
    let version = agent.split('/').find_map(|part| part.strip_prefix("Satoshi:"))?;
    parse_version(&format!("v{version}"))
}

/// Formats a version triple as `31.1.0`.
pub fn version_string((major, minor, patch): (u32, u32, u32)) -> String {
    format!("{major}.{minor}.{patch}")
}

/// Starts the node and waits until it answers.
///
/// Loading a large chainstate takes minutes, so this waits patiently, calling
/// `waiting` once a second so the caller can show that something is happening.
pub fn start(settings: &Settings, mut waiting: impl FnMut(Duration)) -> Result<(), String> {
    let version = installed_version(settings)?;
    if version < MIN_CORE {
        return Err(format!(
            "Bitcoin Core {} is too old: {} or newer is needed",
            version_string(version),
            version_string(MIN_CORE),
        ));
    }

    let conf = write_conf(settings.network)?;
    std::fs::create_dir_all(&settings.datadir)
        .map_err(|error| format!("cannot create {}: {error}", settings.datadir.display()))?;

    let output = Command::new(settings.binaries.join(platform::node_daemon()))
        .arg(format!("-datadir={}", settings.datadir.display()))
        .arg(format!("-conf={}", conf.display()))
        // Wins over the config file's own port, so a port moved because
        // something else held the usual one takes effect.
        .arg(format!("-rpcport={}", settings.rpc_port))
        .arg("-daemon")
        .output()
        .map_err(|error| format!("cannot start the node: {error}"))?;
    if !output.status.success() {
        let said = String::from_utf8_lossy(&output.stderr);
        let said = if said.trim().is_empty() { String::from_utf8_lossy(&output.stdout) } else { said };
        return Err(format!("the node would not start: {}", said.trim()));
    }

    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(600) {
        if is_running(settings) {
            return Ok(());
        }
        waiting(started.elapsed());
        std::thread::sleep(Duration::from_secs(1));
    }
    Err(format!(
        "the node did not answer within ten minutes — its log is at {}",
        settings.datadir.display()
    ))
}

/// Asks the node to shut down.
pub fn stop(settings: &Settings) -> Result<(), String> {
    let client = client(settings)?;
    client
        .call::<serde_json::Value>("stop", serde_json::json!([]))
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_version_core_prints() {
        assert_eq!(parse_version("Bitcoin Core daemon version v31.1.0 bitcoind\nCopyright..."), Some((31, 1, 0)));
        assert_eq!(parse_version("Bitcoin Core daemon version v28.0"), Some((28, 0, 0)));
        assert_eq!(parse_version("nothing useful"), None);
    }

    #[test]
    fn reads_a_user_agent_version() {
        let version = "/Satoshi:31.1.0/"
            .split('/')
            .find_map(|part| part.strip_prefix("Satoshi:"))
            .and_then(|v| parse_version(&format!("v{v}")));
        assert_eq!(version, Some((31, 1, 0)));
    }

    /// The first release with testnet4, and the last one without it.
    #[test]
    fn the_minimum_is_the_first_release_with_testnet4() {
        assert!((27, 2, 0) < MIN_CORE);
        assert!((28, 0, 0) >= MIN_CORE);
    }

    #[test]
    fn knows_where_each_network_listens_for_rpc() {
        assert_eq!(configured_rpc_port(Network::Mainnet), Some(8332));
        assert_eq!(configured_rpc_port(Network::Testnet4), Some(48332));
        assert_eq!(configured_rpc_port(Network::Regtest), Some(18443));
    }
}
