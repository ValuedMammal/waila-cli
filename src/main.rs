#![warn(clippy::pedantic)]
use bitcoin::{Address, Denomination, Network};
use bitcoin::secp256k1::PublicKey;
use bitcoin_waila::PaymentParams;
use clap::{command, Parser};
use core::str::FromStr;
use lightning_invoice::Invoice;
use lnurl::lightning_address::LightningAddress;
use lnurl::lnurl::LnUrl;
use nostr::nips::nip19::ToBech32;
use secp256k1::XOnlyPublicKey;
use serde::Serialize;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
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

#[derive(Debug, Serialize)]
enum NostrValue {
    Hex(XOnlyPublicKey),
    Bech32(String),
}

#[derive(Debug, Serialize)]
struct Base {
    kind: String,
    network: Option<Network>,
    address: Option<Address>,
    invoice: Option<Invoice>,
    pubkey: Option<PublicKey>,
    amount: Option<String>,
    memo: Option<String>,
    lnurl: Option<LnUrl>,
    lnaddr: Option<LightningAddress>,
    nostr: Option<Vec<NostrValue>>,
}

impl Base {
    fn new() -> Self {
        Base {
            kind: String::new(),
            network: None,
            address: None,
            invoice: None,
            pubkey: None,
            amount: None,
            memo: None,
            lnurl: None,
            lnaddr: None,
            nostr: None,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Error {
    // Potentially overkill to create custom errors, but it can be helpful
    // to give context for fallible functions
    ParseParamsError(&'static str),
    SerializeError(String),
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerializeError(
            format!("error creating json output caused by: {e}")
        )
    }
}

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

    let map = parse_params(&s, unit)?;

    let json_out = if args.pretty {
        serde_json::to_string_pretty(&map)?
    } else {
        serde_json::to_string(&map)?
    };

    println!("{json_out}");

    Ok(())
}

fn parse_params(s: &str, unit: Denomination) -> Result<Base> {
    let Ok(parsed) = PaymentParams::from_str(s) else {
        return Err(
            Error::ParseParamsError("not a known bitcoin string")
        )
    };

    let mut m = Base::new();

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

    // Currently supported methods on a PaymentParams with optional
    // return type. Additional methods should be added here
    m.kind = String::from(kind);
    m.network = parsed.network();
    m.address = parsed.address();
    m.invoice = parsed.invoice();
    m.pubkey = parsed.node_pubkey();
    if let Some(amount) = parsed.amount() {
        m.amount = Some(amount.to_string_with_denomination(unit));
    }
    m.memo = parsed.memo();
    m.lnurl = parsed.lnurl();
    m.lnaddr = parsed.lightning_address();
    if let Some(k) = parsed.nostr_pubkey() {
        let bech32 = k.to_bech32().unwrap(); // handle bech32::encode err?
        let v = vec![NostrValue::Hex(k), NostrValue::Bech32(bech32)];
        m.nostr = Some(v);
    }

    Ok(m)
}

#[test]
fn not_a_bitcoin_string() {
    let bad_string = "notabitcoinstring";
    let unit = Denomination::Satoshi;

    assert_eq!(
        parse_params(bad_string, unit).unwrap_err(),
        Error::ParseParamsError("not a known bitcoin string")
    );
}
    