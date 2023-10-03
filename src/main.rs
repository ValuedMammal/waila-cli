use bitcoin::{Amount, Denomination};
use bitcoin_waila::PaymentParams;
use clap::{command, Parser};
use nostr::{
    key::XOnlyPublicKey,
    nips::nip19::{self, ToBech32},
};
use serde_json::{json, Map, Value};
use std::fmt;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        short = 'a',
        long,
        help = "Show all results including None type",
        requires = "query"
    )]
    all: bool,

    #[arg(
        short = 'n',
        long,
        help = "Parse a nostr pubkey in hex and bech32 (experimental)",
        requires = "query"
    )]
    nostr: bool,

    #[arg(
        short = 'f',
        long,
        help = "Remove extra whitespace in JSON output",
        requires = "query"
    )]
    flatten: bool,

    #[arg(
        short = 'u',
        long = "units",
        help = "Bitcoin denomination to display (btc, mbtc, sat, msat)",
        default_value("sat"),
        requires = "query"
    )]
    unit: String,

    #[arg(help = "bitcoin string to parse", required(true))]
    query: String,
}

macro_rules! bail {
    ($err:expr) => {
        println!($err);
        std::process::exit(1);
    };
}

#[derive(Debug)]
enum Error {
    Serialize(serde_json::Error),
    Bech32(nip19::Error),
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Serialize(e)
    }
}

impl From<nip19::Error> for Error {
    fn from(e: nip19::Error) -> Self {
        Error::Bech32(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Serialize(e) => write!(f, "{e}"),
            Error::Bech32(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {}

type Result<T> = core::result::Result<T, Error>;

fn main() -> Result<()> {
    let args = Args::parse();
    let s = args.query;
    let unit = match args.unit.as_str() {
        "btc" => Denomination::Bitcoin,
        "mbtc" => Denomination::MilliBitcoin,
        "msat" => Denomination::MilliSatoshi,
        _ => Denomination::Satoshi,
    };

    let Ok(payment_params) = PaymentParams::from_str(&s) else {
        bail!("not a bitcoin string");
    };

    /* Build a `serde_json::Map` with the following keys. All fields, if applicable, are of type String,
    or `Map<String, String>` in the case of 'nostr'.
        kind
        network
        address
        invoice
        pubkey
        amount
        memo
        lnurl
        lnaddr
        payjoin
        nostr
    */
    let mut map = Map::new();

    // Any additional `PaymentParams` variants must be included here
    let kind = match payment_params {
        PaymentParams::OnChain(_) => "OnChain",
        PaymentParams::Bip21(_) => "UnifiedUri",
        PaymentParams::Bolt11(_) => "Invoice",
        PaymentParams::Bolt12(_) => "Offer",
        PaymentParams::NodePubkey(_) => "PublicKey",
        PaymentParams::LnUrl(_) => "LnUrl",
        PaymentParams::LightningAddress(_) => "LnAddress",
        PaymentParams::Nostr(_) => "NostrValue",
    };
    if kind == "NostrValue" && !args.nostr {
        // don't expose nostr results unsolicited
        bail!("not a bitcoin string");
    }
    map.insert("kind".to_string(), Value::String(kind.to_string()));

    if args.all {
        map = build(&payment_params, map, unit);
    } else {
        map = build_sparse(&payment_params, map, unit);
    };

    if args.nostr {
        map.insert("nostr".to_string(), parse_nostr(&payment_params)?);
    }

    let json_out = if args.flatten {
        serde_json::to_string(&map)?
    } else {
        serde_json::to_string_pretty(&map)?
    };

    println!("{json_out}");

    Ok(())
}

/// Construct a json map with all keys
fn build(
    payment_params: &PaymentParams,
    mut map: Map<String, Value>,
    unit: Denomination,
) -> Map<String, Value> {
    map.insert(
        "network".to_string(),
        if let Some(net) = payment_params.network() {
            Value::String(net.to_string())
        } else {
            json!(null)
        },
    );

    map.insert(
        "address".to_string(),
        if let Some(addr) = payment_params.address() {
            Value::String(addr.to_string())
        } else {
            json!(null)
        },
    );

    map.insert(
        "invoice".to_string(),
        if let Some(inv) = payment_params.invoice() {
            Value::String(inv.to_string())
        } else {
            json!(null)
        },
    );

    map.insert(
        "pubkey".to_string(),
        if let Some(pk) = payment_params.node_pubkey() {
            Value::String(pk.to_string())
        } else {
            json!(null)
        },
    );

    map.insert(
        "amount".to_string(),
        if let Some(amt) = payment_params.amount() {
            // convert to the correct type for our imports
            let sat = amt.to_sat();
            let amt = Amount::from_sat(sat);
            Value::String(amt.to_string_with_denomination(unit))
        } else {
            json!(null)
        },
    );

    map.insert(
        "memo".to_string(),
        if let Some(m) = payment_params.memo() {
            Value::String(m)
        } else {
            json!(null)
        },
    );

    map.insert(
        "lnurl".to_string(),
        if let Some(lnurl) = payment_params.lnurl() {
            Value::String(lnurl.to_string())
        } else {
            json!(null)
        },
    );

    map.insert(
        "lnaddr".to_string(),
        if let Some(lnaddr) = payment_params.lightning_address() {
            Value::String(lnaddr.to_string())
        } else {
            json!(null)
        },
    );

    map.insert(
        "payjoin".to_string(),
        if let Some(url) = payment_params.payjoin_endpoint() {
            Value::String(url.to_string())
        } else {
            json!(null)
        },
    );

    map
}

/// Construct a json map with non-null fields only
fn build_sparse(
    payment_params: &PaymentParams,
    mut map: Map<String, Value>,
    unit: Denomination,
) -> Map<String, Value> {
    if let Some(net) = payment_params.network() {
        map.insert("network".to_string(), Value::String(net.to_string()));
    }

    if let Some(addr) = payment_params.address() {
        map.insert("address".to_string(), Value::String(addr.to_string()));
    }

    if let Some(inv) = payment_params.invoice() {
        map.insert("invoice".to_string(), Value::String(inv.to_string()));
    }

    if let Some(pk) = payment_params.node_pubkey() {
        map.insert("pubkey".to_string(), Value::String(pk.to_string()));
    }

    if let Some(amt) = payment_params.amount() {
        // convert to the correct type for our imports
        let sat = amt.to_sat();
        let amt = Amount::from_sat(sat);
        map.insert(
            "amount".to_string(),
            Value::String(amt.to_string_with_denomination(unit)),
        );
    }

    if let Some(m) = payment_params.memo() {
        map.insert("memo".to_string(), Value::String(m));
    }

    if let Some(lnurl) = payment_params.lnurl() {
        map.insert("lnurl".to_string(), Value::String(lnurl.to_string()));
    }

    if let Some(lnurl) = payment_params.lnurl() {
        map.insert("lnurl".to_string(), Value::String(lnurl.to_string()));
    }

    if let Some(lnaddr) = payment_params.lightning_address() {
        map.insert("lnaddr".to_string(), Value::String(lnaddr.to_string()));
    }

    if let Some(url) = payment_params.payjoin_endpoint() {
        map.insert("payjoin".to_string(), Value::String(url.to_string()));
    }

    map
}

/// Attempts to parse a nostr pubkey from [`PaymentParams`].
/// Returns both hex and bech32 encoding.
///
/// ## Errors
/// If unable to encode bech32
fn parse_nostr(payment_params: &PaymentParams) -> Result<serde_json::Value> {
    let Some(k) = payment_params.nostr_pubkey() else {
        return Ok(json!(null));
    };

    // convert to the correct type for our imports
    let mykey = XOnlyPublicKey::from_str(&k.to_string()).expect("same value");
    let bech32 = mykey.to_bech32()?;

    let hex = k.to_string();

    let mut obj = Map::new();
    obj.insert("hex".to_string(), Value::String(hex));
    obj.insert("bech32".to_string(), Value::String(bech32));

    Ok(Value::Object(obj))
}
