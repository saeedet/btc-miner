# Security

## Reporting a problem

Please report security problems privately, through GitHub: the **Security**
tab of this repository, then **Report a vulnerability**. Don't open a public
issue for anything that could cost someone coins or expose their machine.

Only the latest release is supported.

A bug in the node itself belongs with
[Bitcoin Core](https://bitcoincore.org/en/contact/).

## What this program does with your money and secrets

**It never holds a private key.** If you paste an address, the keys stay in your
own wallet. If setup creates a wallet, it lives in the node: the passphrase is
typed into the program, shown only as dots, and sent straight to your own node
over its local RPC connection. It is never displayed, written to a file, logged,
or put on a command line where other programs or your shell's history could see
it.

**A reward address is checked before it is used.** The node validates it when
it is saved and again when mining starts, because a mistyped or wrong-network
address would produce perfectly valid blocks that pay nobody.

**Downloads are checked before they are trusted.**

- Bitcoin Core is installed by Homebrew, which checks what it downloads.
- The quick-start UTXO snapshot comes from a mirror, but the mirror is not
  trusted: the node refuses any snapshot whose hash differs from the one
  compiled into it.

**The mining side listens only to this computer.** The pool accepts miners on
127.0.0.1 only, and the node's RPC is local and needs the cookie file in its
data folder. The node itself takes part in the Bitcoin network as any node
does.

**Your choices stay out of the repository.** Settings, the reward address and
lifetime totals live in `~/.btc-miner`, and the blockchain and any wallet in
`~/.bitcoin-solo`, never in this repository's folder.
