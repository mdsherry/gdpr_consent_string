extern crate gdpr_consent_string;

use gdpr_consent_string::{ConsentString, Purpose};
use std::cmp;
use std::env;
use std::io::stdin;

fn main() {
    let arg = env::args().skip(1).next();
    let consent_str = arg.unwrap_or_else(|| {
        let mut buf = String::new();
        stdin().read_line(&mut buf).expect("Unable to read from stdin");
        buf.trim().to_string()
    });

    let gdpr = ConsentString::parse(&consent_str);
    match gdpr {
        None => println!("Unable to decode GDPR consent string"),
        Some(gdpr) => {
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
    }
}
