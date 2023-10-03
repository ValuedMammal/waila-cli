use bitcoin::Denomination;
use bitcoin_waila::PaymentParams;
use clap::{command, Parser};
use core::fmt;
use core::str::FromStr;
use nostr::nips::nip19;
use nostr::prelude::ToBech32;
use serde_json::{json, Map, Value};

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
        help = "Expose NIPs (experimental)",
        requires = "query"
    )]
    nips: bool,

    #[arg(short = 'p', long, help = "Pretty printed JSON", requires = "query")]
    pretty: bool,

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

/// Key in the resulting json output
static KEYS: &[&str] = &[
    "kind", "network", "address", "invoice", "pubkey", "amount", "memo", "lnurl", "lnaddr", "payjoin", "nostr",
];

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

    let Ok(parsed) = PaymentParams::from_str(&s) else {
        println!("not a bitcoin string");
        return Ok(())
    };

    // Any additional PaymentParams variants must be included here
    let kind = match parsed {
        PaymentParams::OnChain(_) => "OnChain",
        PaymentParams::Bip21(_) => "UnifiedUri",
        PaymentParams::Bolt11(_) => "Invoice",
        PaymentParams::Bolt12(_) => "Offer",
        PaymentParams::NodePubkey(_) => "PublicKey",
        PaymentParams::LnUrl(_) => "LnUrl",
        PaymentParams::LightningAddress(_) => "LnAddress",
        PaymentParams::Nostr(_) => "NostrValue",
    };
    let kind = String::from(kind);

    let mut map = Map::new();
    map.insert(KEYS[0].into(), Value::String(kind));

    if args.all {
        map = build(&parsed, map, unit);
    } else {
        map = build_sparse(&parsed, map, unit);
    };

    if args.nips {
        map.insert(KEYS[10].into(), parse_nostr(&parsed)?);
    }

    let json_out = if args.pretty {
        serde_json::to_string_pretty(&map)?
    } else {
        serde_json::to_string(&map)?
    };

    println!("{json_out}");

    Ok(())
}

fn build(
    parsed: &PaymentParams,
    mut map: Map<String, Value>,
    unit: Denomination,
) -> Map<String, Value> {
    // net, addr, inv, pubk, amt, memo, lnurl, lnaddr, payjoin
    let val = if let Some(n) = parsed.network() {
        Value::String(n.to_string())
    } else {
        json!(null)
    };
    map.insert(KEYS[1].into(), val);

    let val = if let Some(a) = parsed.address() {
        Value::String(a.to_string())
    } else {
        json!(null)
    };
    map.insert(KEYS[2].into(), val);

    let val = if let Some(i) = parsed.invoice() {
        Value::String(i.to_string())
    } else {
        json!(null)
    };
    map.insert(KEYS[3].into(), val);

    let val = if let Some(k) = parsed.node_pubkey() {
        Value::String(k.to_string())
    } else {
        json!(null)
    };
    map.insert(KEYS[4].into(), val);

    let val = if let Some(a) = parsed.amount() {
        Value::String(a.to_string_with_denomination(unit))
    } else {
        json!(null)
    };
    map.insert(KEYS[5].into(), val);

    let val = if let Some(m) = parsed.memo() {
        Value::String(m)
    } else {
        json!(null)
    };
    map.insert(KEYS[6].into(), val);

    let val = if let Some(u) = parsed.lnurl() {
        Value::String(u.to_string())
    } else {
        json!(null)
    };
    map.insert(KEYS[7].into(), val);

    let val = if let Some(a) = parsed.lightning_address() {
        Value::String(a.to_string())
    } else {
        json!(null)
    };
    map.insert(KEYS[8].into(), val);
    
    let val = if let Some(url) = parsed.payjoin_endpoint() {
        Value::String(url.to_string())
    } else {
        json!(null)
    };
    map.insert(KEYS[9].into(), val);

    map
}

fn build_sparse(
    parsed: &PaymentParams,
    mut map: Map<String, Value>,
    unit: Denomination,
) -> Map<String, Value> {
    // net, addr, inv, pubk, amt, memo, lnurl, lnaddr
    if let Some(n) = parsed.network() {
        let v = Value::String(n.to_string());
        map.insert(KEYS[1].into(), v);
    }

    if let Some(a) = parsed.address() {
        let v = Value::String(a.to_string());
        map.insert(KEYS[2].into(), v);
    }

    if let Some(i) = parsed.invoice() {
        let v = Value::String(i.to_string());
        map.insert(KEYS[3].into(), v);
    }

    if let Some(k) = parsed.node_pubkey() {
        let v = Value::String(k.to_string());
        map.insert(KEYS[4].into(), v);
    }

    if let Some(amt) = parsed.amount() {
        let s = amt.to_string_with_denomination(unit);
        let v = Value::String(s);
        map.insert(KEYS[5].into(), v);
    }

    if let Some(m) = parsed.memo() {
        map.insert(KEYS[6].into(), Value::String(m));
    }

    if let Some(u) = parsed.lnurl() {
        let v = Value::String(u.to_string());
        map.insert(KEYS[7].into(), v);
    }

    if let Some(a) = parsed.lightning_address() {
        let v = Value::String(a.to_string());
        map.insert(KEYS[8].into(), v);
    }
    
    if let Some(url) = parsed.payjoin_endpoint() {
        let v = Value::String(url.to_string());
        map.insert(KEYS[9].into(), v);
    }

    map
}

fn parse_nostr(parsed: &PaymentParams) -> Result<serde_json::Value> {
    if let Some(k) = parsed.nostr_pubkey() {
        let bech32_str = k.to_bech32()?;
        let mut bech32 = String::from("bech32: ");
        bech32.push_str(&bech32_str);

        let hex_str = k.to_string();
        let mut hex = String::from("hex: ");
        hex.push_str(&hex_str);

        Ok(Value::Array(vec![
            Value::String(hex),
            Value::String(bech32),
        ]))
    } else {
        Ok(json!(null))
    }
}
