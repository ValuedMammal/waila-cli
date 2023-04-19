use bitcoin::{Address, Denomination, Network, PublicKey};
use bitcoin_waila::PaymentParams;
use clap::{command, Parser};
use core::result;
use core::str::FromStr;
use lightning_invoice::Invoice;
use lnurl::lnurl::LnUrl;
use serde::Serialize;
use serde_json;

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

#[derive(Debug, PartialEq, Serialize)]
struct Waila {
    kind: String,
    network: Option<Network>,
    address: Option<Address>,
    invoice: Option<Invoice>,
    pubkey: Option<PublicKey>,
    amount: Option<String>,
    memo: Option<String>,
    lnurl: Option<LnUrl>,
    //fallback: Option<Vec<Address>>,
}

impl Waila {
    fn new() -> Self {
        Waila {
            kind: String::new(),
            network: None,
            address: None,
            invoice: None,
            pubkey: None,
            amount: None,
            memo: None,
            lnurl: None,
        }
    }
}

#[derive(Debug, PartialEq)]
enum WailaError {
    // Potentially overkill to create custom errors, but it can be helpful
    // to give context for fallible functions
    ParseParamsError(&'static str),
    SerializeError(String),
}

impl From<serde_json::Error> for WailaError {
    fn from(e: serde_json::Error) -> Self {
        WailaError::SerializeError(
            format!("error creating json output caused by: {e}")
        )
    }
}

type Result<T> = result::Result<T, WailaError>;

fn main() -> Result<()> {
    let args = Args::parse();
    let s = args.query;
    let unit = match args.unit.as_str() {
        "btc" => Denomination::Bitcoin,
        "mbtc" => Denomination::MilliBitcoin,
        "msat" => Denomination::MilliSatoshi,
        _ => Denomination::Satoshi,
    };

    let waila = parse_params(&s, unit)?;

    let json_out = if args.pretty {
        serde_json::to_string_pretty(&waila)?
    } else {
        serde_json::to_string(&waila)?
    };

    println!("{}", json_out);

    Ok(())
}

fn parse_params(s: &str, unit: Denomination) -> Result<Waila> {
    let parsed = match PaymentParams::from_str(s) {
        Ok(parsed) => parsed,
        Err(_) => return Err(
            WailaError::ParseParamsError("not a known bitcoin string")
        ),
    };

    let mut waila = Waila::new();

    // Any additional PaymentParams variants must be included here
    let kind = match parsed {
        PaymentParams::OnChain(_) => "OnChain",
        PaymentParams::Bip21(_) => "UnifiedUri",
        PaymentParams::Bolt11(_) => "Invoice",
        PaymentParams::Bolt12(_) => "Offer",
        PaymentParams::NodePubkey(_) => "PublicKey",
        PaymentParams::LnUrl(_) => "LnUrl",
        PaymentParams::LightningAddress(_) => "LnAddress",
    };

    // Currently supported methods on a PaymentParams with optional
    // return type. Additional methods should be added here
    waila.kind = String::from(kind);
    waila.network = parsed.network();
    waila.address = parsed.address();
    waila.invoice = parsed.invoice();
    waila.pubkey = parsed.node_pubkey();
    if let Some(amount) = parsed.amount() {
        waila.amount = Some(amount.to_string_with_denomination(unit));
    }
    waila.memo = parsed.memo();
    waila.lnurl = parsed.lnurl();

    Ok(waila)
}

#[test]
fn not_a_bitcoin_string() {
    let bad_string = "notabitcoinstring";
    let unit = Denomination::Satoshi;

    assert_eq!(
        parse_params(bad_string, unit).unwrap_err(),
        WailaError::ParseParamsError("not a known bitcoin string")
    );
}
    