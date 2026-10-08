# Changelog

All notable changes to this project. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

The first release, as one command anyone can run.

### Added

- `btc-miner`: one command that sets up whatever is missing and then mines,
  with `setup`, `status`, `wallet`, `doctor` and `stop` alongside it.
- Setup in the terminal, asking only about what isn't ready: installs or
  upgrades Bitcoin Core with Homebrew; takes a reward address the node checks
  first, or creates a passphrase-protected wallet that loads on every start and
  is backed up; and offers a quick start from a UTXO snapshot, then follows the
  sync until mining can begin.
- A live dashboard that fits 80×24 and updates in place: hashrate with a graph,
  the odds in plain words, lifetime totals, recent events, a screen for a found
  block, and help on `?`. Keys to pause and to change the power level.
- Power levels (eco, balanced, max) instead of thread counts, remembered
  between runs.
- Plain log output whenever the output isn't a terminal, or with `--plain`.
- The Mac is kept awake while mining, unless `--allow-sleep` is given.
- Lifetime totals per network, so testing never inflates the mainnet figures.
- Downloads resume after an interruption. A busy RPC or mining port is moved
  to a free one instead of failing.

### Changed

- Renamed from mac-solo-miner. Settings in `~/.solo-mac-miner` move to
  `~/.btc-miner` on first run.
- Requires Bitcoin Core 28.0 or newer, the first release that knows testnet4.
- The pool and miner are libraries, reporting through a shared event stream;
  the `pool` and `miner` binaries remain for debugging.

### Fixed

- Quitting no longer waits for the next block before the hashing threads stop.
- A node one block behind its own headers is no longer treated as syncing.
- A pool whose node falls behind pauses until it recovers, rather than exiting.
- Wallet calls name their wallet explicitly, so they work with several loaded.

## Before the first release

Developed in stages, each with its result checked against real data:

- SHA-256d reproducing the genesis and block 100,000 hashes.
- Block primitives rebuilding a real block's merkle root from its txids.
- A regtest miner whose blocks the node accepts, then a pool and miner split
  over Stratum V1.
- Hashing sped up from 1.4 MH/s to 96.6 MH/s on an M3 with ARM crypto
  extensions, midstate and threads.
- A testnet4 block accepted by a real node as its tip, and a pruned mainnet
  node.
