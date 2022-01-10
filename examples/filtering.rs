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
use dhcpd_parser::leases::{Leases, LeasesMethods};
use dhcpd_parser::util::{LeaseFilterBuilder, LeasesFilter};

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
    println!("Filtered lease list: Count={}", filtered.len());

    let active = LeasesFilter::by_mac_active(leases, "00:ad:d4:39:0d:04");
    println!("Active lease list: Count={}", active.len());
}

/// Demo of filtering with the LeaseFilterBuilder.
///
/// The [LeaseFilterBuilder] provides methods to build the filtering. The sequence of "on_"
/// methods should be specified from broader to narrower filtering.
pub fn demo_using_filter_builder(leases: &Leases) {
    println!("\nFiltering to one MAC, active leases");

    let mut builder = LeaseFilterBuilder::new(&leases);
    let filtered = builder.on_mac("00:ad:d4:39:0d:04")
        .on_active()
        .collect();

    list_leases(&filtered);
}

pub fn main() -> Result<(), String> {
    println!("Filtering example from loaded (Linux) leases file");
    let filename = PathBuf::from_str("tests/data/dhcpd-linux.leases").unwrap();

    let leases = load_leases(&filename)?;
    println!("Loaded {} leases from {}", leases.count(), filename.display());

    list_leases(&leases);
    filter_leases(&leases);
    filter_builder(&leases);

    Result::Ok(())
}
