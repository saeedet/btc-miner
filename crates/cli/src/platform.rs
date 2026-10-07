//! Everything that assumes macOS, in one place.
//!
//! There are three such assumptions: where files live, how the node binary is
//! found, and how to stop the machine sleeping while it mines. Keeping them
//! here rather than scattered through the commands means a Linux or Windows
//! port is a matter of filling in this file, not of hunting for every path
//! built by hand elsewhere.

use std::path::PathBuf;
use std::process::{Child, Command};

/// The user's home directory.
fn home() -> PathBuf {
    std::env::var_os("HOME").map_or_else(|| PathBuf::from("."), PathBuf::from)
}

/// Where this program keeps its own state: settings and lifetime totals.
///
/// Moves the directory used before the project was renamed, if that is all
/// there is, so nothing saved under the old name is lost.
pub fn state_dir() -> PathBuf {
    miner::state_dir_in(&home())
}

/// The node's data directory, unless the settings say otherwise.
///
/// Deliberately not Bitcoin Core's own default, so nothing this program does
/// can touch another node or wallet on the same machine.
pub fn default_node_datadir() -> PathBuf {
    home().join(".bitcoin-solo")
}

/// Where the node's programs live, unless the settings say otherwise.
///
/// The first directory on `PATH` that holds the daemon, falling back to where
/// Homebrew installs Bitcoin Core.
pub fn default_node_binaries() -> PathBuf {
    std::env::var_os("PATH")
        .and_then(|path| std::env::split_paths(&path).find(|dir| dir.join(node_daemon()).is_file()))
        .unwrap_or_else(|| PathBuf::from("/opt/homebrew/opt/bitcoin/bin"))
}

/// The node daemon's file name.
pub const fn node_daemon() -> &'static str {
    if cfg!(windows) { "bitcoind.exe" } else { "bitcoind" }
}

/// Keeps the machine from sleeping for as long as this value lives.
///
/// A laptop that idles to sleep stops mining without a word, which a newcomer
/// would reasonably read as the program having crashed. On macOS this runs the
/// system's own `caffeinate`, tied to this process so it cannot outlive it.
pub struct KeepAwake(Option<Child>);

impl KeepAwake {
    /// Starts preventing sleep. Does nothing where that isn't supported yet.
    pub fn start() -> Self {
        if cfg!(target_os = "macos") {
            // -i: no idle sleep. -w: exit when this process does, even if it
            // is killed rather than stopped cleanly.
            let child = Command::new("caffeinate")
                .arg("-i")
                .arg("-w")
                .arg(std::process::id().to_string())
                .spawn()
                .ok();
            Self(child)
        } else {
            Self(None)
        }
    }

    /// Whether sleep is actually being prevented.
    pub fn active(&self) -> bool {
        self.0.is_some()
    }
}

impl Drop for KeepAwake {
    fn drop(&mut self) {
        if let Some(child) = &mut self.0 {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
