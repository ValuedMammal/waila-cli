# waila-cli
basic cli for [bitcoin-waila](https://github.com/MutinyWallet/bitcoin-waila/)

## Install
```sh
# Build from source (requires rust, cargo).
git clone https://github.com/ValuedMammal/waila-cli.git
cd waila-cli

cargo install --path .

waila-cli --version
# waila-cli 0.1.0
```

## Usage
```
$ waila-cli --help

What am I looking at? - parser for bitcoin strings

Usage: waila-cli [OPTIONS] <QUERY>

Arguments:
  <QUERY>  bitcoin string to parse

Options:
  -a, --all           Show all results including None type
  -n, --nostr         Parse a nostr pubkey in hex and bech32 (experimental)
  -f, --flatten       Remove extra whitespace in JSON output
  -u, --units <UNIT>  Bitcoin denomination to display (btc, mbtc, sat, msat) [default: sat]
  -h, --help          Print help
  -V, --version       Print version

```

## Example
```bash
$ waila-cli "tb1pwzv7fv35yl7ypwj8w7al2t8apd6yf4568cs772qjwper74xqc99sk8x7tk"

{
  "address": "tb1pwzv7fv35yl7ypwj8w7al2t8apd6yf4568cs772qjwper74xqc99sk8x7tk",
  "kind": "OnChain",
  "network": "testnet"
}
```

