use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use chrono::{TimeZone, Utc};
use dhcpd_parser::leases::LeasesMethods;
use dhcpd_parser::parser;
use dhcpd_parser::util::LeaseFilterBuilder;

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


#[test]
/// This should always filter to 0 items since it matches the ends to Utc::now()
fn filter_on_active_test() {
    if let Result::Ok(content) = load_file(&PathBuf::from_str("tests/data/dhcpd-multiple.leases").unwrap()) {
        match parser::parse(content) {
            Ok(res) => {
                let leases = res.leases;
                let mut builder = LeaseFilterBuilder::new(&leases);
                let filtered = builder.on_mac("00:ea:d4:39:0d:04")
                    .on_active()
                    .collect();

                assert_eq!(filtered.count(), 0);
            },
            Err(e) => assert!(false, "{}", e)
        }
    }
}

#[test]
/// Test to select item with "ends 2 2022/01/11 00:13:17;"
fn filter_on_active_now_test() {
    // This is after the 2nd from the end, but before the last item
    let compare_dt = Utc.ymd(2022, 1, 11).and_hms(0, 9, 10);

    if let Result::Ok(content) = load_file(&PathBuf::from_str("tests/data/dhcpd-multiple.leases").unwrap()) {
        match parser::parse(content) {
            Ok(res) => {
                let leases = res.leases;
                let mut builder = LeaseFilterBuilder::new(&leases);

                let filtered = builder.on_mac("00:ea:d4:39:0d:04")
                    .on_active_now(Some(compare_dt))
                    .collect();

                assert_eq!(filtered.count(), 1);
            },
            Err(e) => assert!(false, "{}", e)
        }
    }
}

#[test]
/// Test with date that is before the end of the last two items
fn filter_on_active_now_two_test() {
    // This is after the 2nd from the end, but before the last item
    let compare_dt = Utc.ymd(2022, 1, 11).and_hms(0, 7, 10);
    let content = load_file(&PathBuf::from_str("tests/data/dhcpd-multiple.leases").unwrap()).unwrap();

    match parser::parse(content) {
        Ok(res) => {
            let leases = res.leases;
            let mut builder = LeaseFilterBuilder::new(&leases);

            let filtered = builder.on_mac("00:ea:d4:39:0d:04")
                .on_active_now(Some(compare_dt))
                .collect();

            assert_eq!(filtered.count(), 2);
        },
        Err(e) => assert!(false, "{}", e)
    }
}

#[test]
/// Test for the latest item
fn filter_on_latest_test() {
    let content = load_file(&PathBuf::from_str("tests/data/dhcpd-multiple.leases").unwrap()).unwrap();

    match parser::parse(content) {
        Ok(res) => {
            let leases = res.leases;
            let mut builder = LeaseFilterBuilder::new(&leases);

            let filtered = builder.on_mac("00:ea:d4:39:0d:04")
                .latest()
                .collect();

            assert_eq!(filtered.count(), 1);
            let lease = &filtered[0];
            // Looking for: 2022/01/11 00:13:17
            assert_eq!(lease.lease_end_dts(), Utc.ymd(2022, 01, 11).and_hms(0, 13, 17));
        },
        Err(e) => assert!(false, "{}", e)
    }
}

#[test]
fn filter_on_mac_expired_test() {
    let content = load_file(&PathBuf::from_str("tests/data/dhcpd-multiple.leases").unwrap()).unwrap();

    match parser::parse(content) {
        Ok(res) => {
            let leases = res.leases;
            let mut builder = LeaseFilterBuilder::new(&leases);

            let filtered = builder.on_mac("00:aa:bb:cc:dd:01")
                .collect();

            assert_eq!(filtered.count(), 2);
            let lease = &filtered[0];
            assert_eq!(lease.is_active(), false);
        },
        Err(e) => assert!(false, "{}", e)
    }
}

#[test]
fn filter_on_mac_expired_latest_test() {
    let content = load_file(&PathBuf::from_str("tests/data/dhcpd-multiple.leases").unwrap()).unwrap();

    match parser::parse(content) {
        Ok(res) => {
            let leases = res.leases;
            let mut builder = LeaseFilterBuilder::new(&leases);

            let filtered = builder.on_mac("00:aa:bb:cc:dd:01")
                .latest()
                .collect();

            assert_eq!(filtered.count(), 1);
            let lease = &filtered[0];
            // Looking for: 2022/01/11 00:08:50
            assert_eq!(lease.is_active(), false);
            assert_eq!(lease.lease_end_dts(), Utc.ymd(2022, 01, 11).and_hms(0, 8, 50));
        },
        Err(e) => assert!(false, "{}", e)
    }
}

#[test]
fn filter_on_ip_expired_test() {
    let content = load_file(&PathBuf::from_str("tests/data/dhcpd-multiple.leases").unwrap()).unwrap();

    match parser::parse(content) {
        Ok(res) => {
            let leases = res.leases;
            let mut builder = LeaseFilterBuilder::new(&leases);

            let filtered = builder.on_ip("192.168.4.106")
                .collect();

            assert_eq!(filtered.count(), 3);
            let lease = &filtered[0];
            assert_eq!(lease.is_active(), false);
            assert_eq!(lease.client(), "00:ea:d4:39:0d:04");
        },
        Err(e) => assert!(false, "{}", e)
    }
}
