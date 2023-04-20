# waila-cli

basic cli for [bitcoin-waila](https://crates.io/crates/bitcoin-waila)  

### Samples
![](/doc/waila-help.png?raw=true)

Use `-p` or pipe the output to `jq` for "pretty"

![](/doc/testnet-onchain.png?raw=true)

### Try it  
    # Clone the repo
    $ git clone https://github.com/ValuedMammal/waila-cli.git
    $ cd waila-cli
    
    # Build (requires rust, cargo)
    $ cargo build --release --locked

    # Install
    $ cargo install --path .

    # Run
    $ waila-cli --version
    >  waila-cli 0.1.0
