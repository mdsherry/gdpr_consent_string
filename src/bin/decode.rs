extern crate gdpr_consent_string;
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use gdpr_consent_string::{ConsentString, Purpose};
use std::cmp;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Args {
    #[structopt(name = "STRING")]
    string: Option<String>,

    #[structopt(short = "f", long = "file", parse(from_os_str))]
    file: Option<PathBuf>,

    #[structopt(short = "o", long = "output")]
    format: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SConsentString {
    pub version: u8,
    pub created: u64,
    pub last_updated: u64,
    pub cmp_id: u16,
    pub cmp_version: u16,
    pub consent_screen: u8,
    pub consent_language: String,
    pub vendor_list_version: u16,
    pub purposes_allowed: Vec<u8>,
    pub max_vendor_id: u16,
    pub vendor_consents: Vec<u16>,
}

#[derive(Copy, Clone, Debug)]
enum Format {
    Human { chart: bool },
    Json,
}

fn main() {
    let args = Args::from_args();
    let format = match args.format.as_ref().map(|x| x.as_str()) {
        None | Some("human") => Format::Human { chart: true },
        Some("json") => Format::Json,
        Some(other) => {
            eprintln!("Unrecognized format {}", other);
            std::process::exit(1);
        }
    };

    match args.string {
        Some(consent_str) => decode(&consent_str, format),
        None => {
            args.file
                .map(|fname| {
                    let f = File::open(fname).expect("Unable to open file");
                    BufReader::new(f)
                        .lines()
                        .for_each(|line| decode(line.unwrap().trim(), format));
                })
                .unwrap_or_else(|| {
                    BufReader::new(stdin())
                        .lines()
                        .for_each(|line| decode(line.unwrap().trim(), format))
                });
        }
    }
}

fn decode(consent_str: &str, format: Format) {
    let gdpr = ConsentString::parse(consent_str);
    match gdpr {
        None => println!("Unable to decode GDPR consent string"),
        Some(gdpr) => match format {
            Format::Human { .. } => print_human(&gdpr),
            Format::Json => print_json(&gdpr),
        },
    }
}

fn print_json(gdpr: &ConsentString) {
    let purposes = {
        let mut rv = vec![];
        if gdpr.purposes_allowed.contains(Purpose::StorageAndAccess) {
            rv.push(1);
        }
        if gdpr.purposes_allowed.contains(Purpose::Personalization) {
            rv.push(2);
        }
        if gdpr.purposes_allowed.contains(Purpose::AdSelection) {
            rv.push(3);
        }
        if gdpr.purposes_allowed.contains(Purpose::ContentDelivery) {
            rv.push(4);
        }
        if gdpr.purposes_allowed.contains(Purpose::Measurement) {
            rv.push(5);
        }
        rv
    };
    let consents: Vec<_> = gdpr.vendor_consents
        .iter()
        .enumerate()
        .filter_map(|(id, &value)| {
            if id > 0 && value {
                Some(id as u16)
            } else {
                None
            }
        })
        .collect();
    let gdpr = SConsentString {
        version: gdpr.version,
        created: (gdpr.created.timestamp() as u64) * 10
            + (gdpr.created.timestamp_subsec_millis() / 100) as u64,
        last_updated: (gdpr.last_updated.timestamp() as u64) * 10
            + (gdpr.last_updated.timestamp_subsec_millis() / 100) as u64,
        cmp_id: gdpr.cmp_id,
        cmp_version: gdpr.cmp_version,
        consent_screen: gdpr.consent_screen,
        consent_language: gdpr.consent_language.iter().collect::<String>(),
        vendor_list_version: gdpr.vendor_list_version,
        purposes_allowed: purposes,
        max_vendor_id: gdpr.max_vendor_id,
        vendor_consents: consents,
    };

    println!(
        "{}",
        serde_json::to_string(&gdpr).expect("Unable to serialize JSON")
    );
}

fn print_human(gdpr: &ConsentString) {
    let purposes = {
        let mut rv = vec![];
        if gdpr.purposes_allowed.contains(Purpose::StorageAndAccess) {
            rv.push("Storage and access");
        }
        if gdpr.purposes_allowed.contains(Purpose::Personalization) {
            rv.push("Personalization");
        }
        if gdpr.purposes_allowed.contains(Purpose::AdSelection) {
            rv.push("Ad selection");
        }
        if gdpr.purposes_allowed.contains(Purpose::ContentDelivery) {
            rv.push("Content delivery");
        }
        if gdpr.purposes_allowed.contains(Purpose::Measurement) {
            rv.push("Measurement");
        }
        rv.join(", ")
    };
    let consents = {
        let mut rv = vec![];
        let rows = (gdpr.max_vendor_id / 100 + 1) as usize;
        for row in 0..rows {
            if row % 10 == 0 {
                rv.push("    0000000000 1111111111 2222222222 3333333333 4444444444 5555555555 6666666666 7777777777 8888888888 9999999999".to_string());
                rv.push("    0123456789 0123456789 0123456789 0123456789 0123456789 0123456789 0123456789 0123456789 0123456789 0123456789".to_string());
            }
            let mut row_str = format!("{:3}", row);
            for vid in (100 * row)..cmp::min(100 * (1 + row), (gdpr.max_vendor_id + 1) as usize) {
                if vid % 10 == 0 {
                    row_str.push(' ');
                }
                if vid == 0 {
                    row_str.push(' ')
                } else {
                    row_str.push(if gdpr.vendor_consents[vid] { '#' } else { ' ' });
                }
            }
            rv.push(row_str);
        }
        rv.join("\n")
    };
    println!(
        "
GDPR Consent String (v{version})
Created {created}; last updated {last_updated}
CMP Id: {cmp_id} (v{cmp_version})
Consent screen number: {consent_screen}
Consent language: {consent_language}
Vendor list version: {vendor_list_version}
Purposes allowed: {purposes}
Vendor consents:
{consents}
",
        version = gdpr.version,
        created = gdpr.created,
        last_updated = gdpr.last_updated,
        cmp_id = gdpr.cmp_id,
        cmp_version = gdpr.cmp_version,
        consent_screen = gdpr.consent_screen,
        consent_language = gdpr.consent_language.iter().collect::<String>(),
        vendor_list_version = gdpr.vendor_list_version,
        purposes = purposes,
        consents = consents
    );
}
