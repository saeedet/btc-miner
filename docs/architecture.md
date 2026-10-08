# How btc-miner fits together

For using the program, see the [README](../README.md).

## One command, three parts

```
                      ┌───────────────── btc-miner ───────────────────┐
Bitcoin Core ──RPC──▶ │  pool  ──Stratum V1──▶  miner                 │
(~/.bitcoin-solo)     │    │                      │                   │
                      │    └────── events ────────┴──▶  dashboard     │
                      │                                 or plain log  │
                      └───────────────────────────────────────────────┘
```

- **The node** is Bitcoin Core, a separate program that keeps running between
  mining sessions so the next start is instant. It validates everything and
  builds block templates.
- **The pool** asks the node for a template, turns it into Stratum jobs, checks
  the shares that come back, and submits any that make a block.
- **The miner** hashes. It knows nothing about blocks or the node — only the
  jobs the pool sends it.

The pool and miner talk real Stratum V1 over TCP, even inside one process. That
keeps the protocol boundary honest: an ASIC such as a Bitaxe could replace the
miner without the pool changing. The pool listens on 127.0.0.1 only.

Neither half prints anything itself. Both emit `Event`s into a sink. The plain
sink prints the same lines the programs always printed; the dashboard folds the
same events into one screen. So a log and the dashboard can never disagree about
what happened.

## Crate map

| Crate | Responsibility |
|---|---|
| `sha256d` | SHA-256 and double-SHA-256. A readable reference implementation and an ARM crypto-extension one, tested against each other |
| `btc-primitives` | Block headers, transactions, varints, merkle trees, difficulty targets. Pure, no I/O. Byte order is enforced by the type system |
| `bitcoind-rpc` | Typed JSON-RPC for `getblocktemplate` / `submitblock` and friends. Cookie auth; hand-rolled HTTP and base64 |
| `mining` | Coinbase construction, block assembly, nonce search. Pure, no I/O |
| `stratum` | Stratum V1 wire types, shared by pool and miner so they cannot disagree. Owns the byte-order conventions |
| `events` | What the pool and miner have to say, as data, and the plain renderer. Identical in bip110-miner |
| `pool` | The solo pool: the node on one side, Stratum on the other. A library, with a thin `pool` binary |
| `miner` | The hashing client, with live controls for pause, stop and thread count. A library, with a thin `miner` binary |
| `cli` | The `btc-miner` command: setup, the dashboard, and the other subcommands |
| `regtest-miner` | A self-contained miner for a local regtest chain |
| `blk-scan` | A separate utility: reports which chain a set of Bitcoin Core block files contains, and joins files while handling their obfuscation |

Everything Bitcoin-specific is written here. The few dependencies do generic
jobs that teach nothing about mining: JSON (`serde`), argument parsing (`clap`),
the settings file (`toml`), Ctrl-C (`ctrlc`), and drawing the terminal
(`ratatui`).

## Inside the command

| Module | What it does |
|---|---|
| `setup/` | The checklist that runs before mining and asks only about what's missing |
| `dashboard/` | The live screen: `state.rs` turns events into what's shown, `view.rs` lays it out |
| `commands/` | One file per subcommand |
| `config.rs` | The settings file, and which source wins: flag, then environment, then file |
| `node.rs` | Finding, starting and stopping the node, and checking its version |
| `chain.rs` | Everything that differs from bip110-miner's chain, in one file |
| `platform.rs` | Everything that assumes macOS, in one file — the place a Linux or Windows port starts |

The dashboard and setup screens are shared with bip110-miner, file for file;
only `chain.rs` and `setup/install.rs` differ.

## Speed

Measured on an M3, `cargo run --release --example hashrate -p sha256d`:

| | Single core | |
|---|---|---|
| Portable reference | 1.4 MH/s | mirrors FIPS 180-4, not trying to be fast |
| ARMv8 crypto extensions | 6.0 MH/s | hardware SHA-256 instructions |
| \+ midstate, no allocation | 17.9 MH/s | the header's first 64 bytes never change |

then near-linear scaling: 34.1 MH/s on 2 threads, 68.5 on 4, 96.6 on 8. The step
from 4 to 8 adds less than the first four because the M3's second four cores are
efficiency cores.

At a difficulty of about 127.5 trillion, the expected time to a block is
`difficulty × 2³² / hashrate`:

| Hardware | Hashrate | Expected time to a block |
|---|---|---|
| M3, all 8 cores | 96.6 MH/s | about 180,000,000 years |
| Bitaxe Gamma | 1.2 TH/s | about 14,500 years |

## Reading the lifetime line

The plain log ends each report with lifetime totals:

```
  30.83 MH/s (avg  30.50)   session  610.27M   best 30/78 bits   00000002894d5...
          lifetime  215.90G   best ever 38/78 bits (2^40 short)   ~1 in 2.503e12 of a block
```

`38/78` is the best hash ever found against the leading zero bits a block
actually needs. Read as a fraction it looks like halfway; it is not. Bits are
exponential, which is what `2^40 short` says: the best hash in 215 billion
attempts is still about a trillion times too easy.

The best-ever figure is worth nothing in consensus terms — a near miss is a miss,
and it says nothing about the next hash. It is tracked because it is the only
feedback solo mining ever gives.

## Running the pieces by hand

The `pool` and `miner` binaries still exist, and are what to reach for when
debugging one side alone:

```bash
cargo run --release -p pool -- --network regtest
cargo run --release -p miner -- --pool 127.0.0.1:3333 --worker test.0
```

`cargo run --release -p regtest-miner -- 10` mines ten regtest blocks with no
pool at all.

`scripts/` holds the development tools that predate the command: `node.sh`
controls a node with a repo config, `mine.sh` runs the pool and miner as
separate processes, and `sync-status.sh` reports a sync.

## How it was built

- **0** — Toolchains, repo skeleton, regtest node running
- **1** — `sha256d`: reproduces the genesis and block-100000 hashes
- **2** — `btc-primitives`: rebuilds a real block's merkle root from its txids
- **3** — A monolithic regtest miner: the node accepts a block mined here
- **4** — Split into pool and miner over Stratum V1
- **5** — Optimised: midstate, ARM crypto extensions, multithreading
- **6** — testnet4: built a valid block a real node accepted as its tip
- **7** — A mainnet pruned node, "lottery mode"
- **8** — One command, a live dashboard, and setup that asks for what's missing
