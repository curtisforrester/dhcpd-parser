//! Example showing how to perform filtering on the [Leases] based on some criteria.
//!
//! The leases are loaded (from a file that is contained within the "tests/data" directory),
//! and filter is applied. The results are printed in a simple table.
extern crate dhcpd_parser;

use crate::dhcpd_parser::parser;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use chrono::{TimeZone, Utc};
use dhcpd_parser::leases::{Leases, LeasesMethods};
use dhcpd_parser::util::{LeaseFilterBuilder, LeasesFilter};

struct Divider {
    thin: String,
    bold: String
}
impl Divider {
    pub fn new() -> Divider {
        Divider {
            thin: String::from_utf8(vec![b'-'; 70]).unwrap(),
            bold: String::from_utf8(vec![b'='; 70]).unwrap()
        }
    }
}

/// Load the contents of the leases file
pub fn load_file(filename: &PathBuf) -> Result<String, String> {
    return match File::open(&filename) {
        Ok(mut f) => {
            let mut s = String::new();
            f.read_to_string(&mut s).unwrap();
            Result::Ok(s)
        },
        Err(e) => {
            Result::Err(format!("Failed to open: {}. Error: {}", filename.display(), e))
        }
    }
}

/// Load the leases from the leases file
pub fn load_leases(filename: &PathBuf) -> Result<Leases, String> {
    if let Result::Ok(content) = load_file(filename) {
        return match parser::parse(content) {
            Result::Ok(res) => {
                Result::Ok(res.leases)
            },
            Result::Err(e) => Result::Err(e)
        }
    }
    else {
        Result::Err("Failed to load leases".to_string())
    }
}

/// List the loaded leases
pub fn list_leases(leases: &Leases) {
    for lease in leases.all() {
        println!("IP: {}, Client: {}, Ends: {}, IsActive: {}",
                 lease.ip,
                 lease.client(),
                 lease.lease_end_dts(),
                 lease.is_active()
        );
    }
}

/// Demo of simple filtering of leases.
pub fn demo_simple_filter_leases(leases: &Leases) {
    let filtered = LeasesFilter::by_mac_all(leases, "00:ad:d4:39:0d:04");
    println!("Simple filtered lease list: Count={}", filtered.len());

    let active = LeasesFilter::by_mac_active(leases, "00:ad:d4:39:0d:04");
    println!("Active lease list: Count={}", active.len());
}

/// Demo of filtering with the LeaseFilterBuilder.
///
/// The [LeaseFilterBuilder] provides methods to build the filtering. The sequence of "on_"
/// methods should be specified from broader to narrower filtering.
pub fn demo_filter_builder(leases: &Leases) {
    let compare_dt = Utc.ymd(2022, 1, 11).and_hms(0, 7, 10);
    let div = Divider::new();

    println!("\n{}", div.thin);
    println!("Filtering to one MAC, active leases as of {}", compare_dt);

    let mut builder = LeaseFilterBuilder::new(&leases);
    let filtered = builder.on_mac("00:ea:d4:39:0d:04")
        .on_active_now(Some(compare_dt))
        .collect();

    list_leases(&filtered);
}

/// Demo of filtering with LeaseFilterBuilder to get the latest active lease for a MAC
pub fn demo_get_latest_lease(leases: &Leases) {
    let compare_dt = Utc.ymd(2022, 1, 11).and_hms(0, 7, 10);
    let div = Divider::new();

    println!("\n{}", div.thin);
    println!("Filtering to one MAC, most recent active lease as of {}", compare_dt);

    let mut builder = LeaseFilterBuilder::new(&leases);
    let filtered = builder.on_mac("00:ea:d4:39:0d:04")
        .on_active_now(Some(compare_dt))
        .latest()
        .collect();

    list_leases(&filtered);
}

fn simple_filtering() -> Result<(), String> {
    let filename = PathBuf::from_str("tests/data/dhcpd-linux.leases").unwrap();
    let div = Divider::new();

    println!("\n{}", div.thin);
    println!("Demo simple filtering from file: {}", filename.display());

    let leases = load_leases(&filename)?;
    println!("Loaded {} leases", leases.count());

    list_leases(&leases);
    demo_simple_filter_leases(&leases);
    Ok(())
}

fn builder_filtering() -> Result<(), String> {
    let filename = PathBuf::from_str("tests/data/dhcpd-multiple.leases").unwrap();
    let div = Divider::new();

    println!("\n{}", div.thin);
    println!("Demo builder filtering from: {}", filename.display());

    let leases = load_leases(&filename)?;
    println!("Loaded {} leases", leases.count());

    demo_filter_builder(&leases);
    demo_get_latest_lease(&leases);
    Ok(())
}

pub fn main() -> Result<(), String> {
    let div = Divider::new();

    println!("{}\nSimple Filtering Demo", div.bold);
    simple_filtering()?;

    println!("\n{}\nBuilder Filtering Demo", div.bold);
    builder_filtering()?;

    Result::Ok(())
}
