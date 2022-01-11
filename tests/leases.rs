extern crate dhcpd_parser;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use crate::dhcpd_parser::common::Date;
use crate::dhcpd_parser::parser;
use crate::dhcpd_parser::parser::LeasesMethods;

#[test]
fn basic_test() {
    let res = parser::parse(
        "
    lease 192.0.0.2 {

    }"
        .to_string(),
    );
    assert!(res.is_ok());
}

#[test]
fn dates_test() {
    let res = parser::parse(
        "lease 255.254.253.252 {
        starts 2 2019/01/01 22:00:00 UTC;
        ends 2 2019/01/01 22:00:00 UTC;
    }"
        .to_string(),
    );
    assert!(res.is_ok());
}

#[test]
fn all_options_test() {
    let res = parser::parse(
        "
    lease 192.168.0.2 {
        starts 2 2019/01/01 22:00:00 UTC;
        ends 2 2019/01/01 22:00:00 UTC;
        hardware type 11:11:11:11:11:11;
        uid Client1;
        client-hostname \"CLIENTHOSTNAME\";
        hostname \"TESTHOSTNAME\";
        abandoned;
    }",
    );

    assert!(res.is_ok());
}

#[test]
fn multiple_leases_test() {
    let res = parser::parse(
        "
    lease 192.168.0.2 {
        starts 2 2019/01/01 22:00:00 UTC;
        ends 2 2019/01/01 22:00:00 UTC;
        hardware type 11:11:11:11:11:11;
        uid Client1;
        client-hostname \"CLIENTHOSTNAME\";
        hostname \"TESTHOSTNAME\";
        abandoned;
    }

    lease 192.168.0.3 {
        starts 1 1985/01/01 00:00:00 UTC;
        hardware type 22:22:22:22:22:22;
        uid Client2;
        hostname \"TESTHOSTNAME\";
    }
    ",
    );

    assert!(res.is_ok());

    let leases = res.unwrap().leases;
    assert_eq!(leases[0].hostname.as_ref().unwrap(), "TESTHOSTNAME");
    assert_eq!(
        leases[1].dates.starts.unwrap().to_string(),
        "Monday 1985/01/01 00:00:00"
    );
    assert!(leases[1].dates.ends.is_none());

    assert!(leases[0].abandoned);
    assert!(!leases[1].abandoned);
}

#[test]
fn invalid_format_test() {
    let res = parser::parse(
        "
    lease 192.0.0.2 {

    ",
    );
    assert!(res.is_err());
}

#[test]
fn invalid_date_format_test() {
    let res = parser::parse(
        "
    lease 192.0.0.2 {
        starts 2 2019-01-02 00:00:00;
    }",
    );
    assert!(res.is_err());
}

#[test]
fn is_active_test() {
    let res = parser::parse(
        "
    lease 192.168.0.2 {
        starts 2 2019/01/01 22:00:00 UTC;
        ends 2 2019/01/01 23:00:00 UTC;
        hardware type 11:11:11:11:11:11;
        uid Client1;
        client-hostname \"CLIENTHOSTNAME\";
        hostname \"TESTHOSTNAME\";
        abandoned;
    }

    lease 192.168.0.3 {
        starts 1 1985/01/02 00:00:00 UTC;
        hardware type 22:22:22:22:22:22;
        uid Client2;
        hostname \"TESTHOSTNAME\";
    }
    ",
    );

    let leases = res.unwrap().leases;

    assert!(leases[0].is_active_at(Date::from("2", "2019/01/01", "22:30:00").unwrap()));

    assert_eq!(
        leases[1].is_active_at(Date::from("1", "1985/01/01", "22:30:00").unwrap()),
        false
    );

    assert_eq!(
        leases[0].is_active_at(Date::from("2", "2019/01/01", "21:59:00").unwrap()),
        false
    );

    assert_eq!(
        leases[0].is_active_at(
            Date::from(
                "2".to_string(),
                "2019/01/01".to_string(),
                "23:59:00".to_string()
            )
            .unwrap()
        ),
        false
    );
}

#[test]
fn hostnames_test() {
    let res = parser::parse(
        "
    lease 192.168.0.2 {
        starts 2 2019/01/01 22:00:00 UTC;
        ends 2 2019/01/01 23:00:00 UTC;
        hardware type 11:11:11:11:11:11;
        uid Client1;
        client-hostname \"CLIENTHOSTNAME\";
        hostname \"TESTHOSTNAME\";
    }

    lease 192.168.0.3 {
        starts 1 1985/01/02 00:00:00 UTC;
        ends 1 1985/01/02 02:00:00 UTC;
        hardware type 22:22:22:22:22:22;
        uid Client2;
        hostname \"TESTHOSTNAME\";
    }
    ",
    );

    let leases = res.unwrap().leases;

    assert_eq!(
        leases.hostnames(),
        ["TESTHOSTNAME".to_owned()].iter().cloned().collect()
    );
}

#[test]
fn client_hostnames_test() {
    let res = parser::parse(
        "
    lease 192.168.0.2 {
        starts 2 2019/01/01 22:00:00 UTC;
        ends 2 2019/01/01 23:00:00 UTC;
        hardware type 11:11:11:11:11:11;
        uid Client1;
        client-hostname \"CLIENTHOSTNAME\";
        hostname \"TESTHOSTNAME\";
        abandoned;
    }

    lease 192.168.0.3 {
        starts 1 1985/01/02 00:00:00 UTC;
        ends 1 1985/01/02 02:00:00 UTC;
        hardware type 22:22:22:22:22:22;
        uid Client2;
        hostname \"TESTHOSTNAME\";
        client-hostname \"HN\";
    }

    lease 192.168.0.3 {
        starts 1 1986/01/02 00:00:00 UTC;
        ends 1 1986/12/02 02:00:00 UTC;
        hardware type 22:22:22:22:22:22;
        uid Client2;
        client-hostname \"HN\";
    }
    ",
    );

    let leases = res.unwrap().leases;

    assert_eq!(
        leases.client_hostnames(),
        ["CLIENTHOSTNAME".to_owned(), "HN".to_owned()]
            .iter()
            .cloned()
            .collect()
    );
}

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
fn load_file_test() {
    let content = load_file(&PathBuf::from_str("tests/data/dhcpd-linux.leases").unwrap());
    assert!(content.is_ok());
}

#[test]
fn load_linux_leases_from_file_test() {
    if let Result::Ok(content) = load_file(&PathBuf::from_str("tests/data/dhcpd-linux.leases").unwrap()) {
        match parser::parse(content) {
            Result::Ok(res) => {
                let leases = res.leases;
                assert_eq!(leases.count(), 6);
                assert_eq!(leases[0].byte_order, Some("little-endian".to_string()));
            },
            Result::Err(e) => assert!(false, "{}", e)
        }
    }
}

#[test]
fn load_linux_multi_leases_from_file_test() {
    if let Result::Ok(content) = load_file(&PathBuf::from_str("tests/data/dhcpd-multiple.leases").unwrap()) {
        match parser::parse(content) {
            Result::Ok(res) => {
                let leases = res.leases;
                assert_eq!(leases.count(), 16);
                assert_eq!(leases[0].byte_order, Some("little-endian".to_string()));
            },
            Result::Err(e) => assert!(false, "{}", e)
        }
    }
}

#[test]
fn load_bsd_leases_from_file_test() {
    if let Result::Ok(content) = load_file(&PathBuf::from_str("tests/data/dhcpd-bsd.leases").unwrap()) {
        match parser::parse(content) {
            Result::Ok(res) => {
                let leases = res.leases;
                assert_eq!(leases.count(), 3, "Count of leases in BSD file is not 3. Found: {}", leases.count());
                assert_eq!(leases[0].byte_order, None);
            },
            Result::Err(e) => assert!(false, "{}", e)
        }
    }
}

#[test]
fn linux_test() {
    let res = parser::parse(
        "
    lease 192.168.4.105 {
      starts 3 2022/01/05 16:51:33;
      ends 3 2022/01/05 18:51:33;
      tstp 3 2022/01/05 18:51:33;
      cltt 3 2022/01/05 16:51:33;
      binding state free;
      hardware ethernet 00:ea:d4:39:0d:04;
      uid \"\\001\\000\\352\\3249\\015\\004\";
      reserved a b c d;
    }

    lease 192.168.4.108 {
      starts 6 2022/01/08 17:46:16;
      ends 6 2022/01/08 17:56:16;
      cltt 6 2022/01/08 17:46:16;
      binding state active;
      next binding state free;
      rewind binding state free;
      hardware ethernet 00:ea:d4:39:0d:04;
      client-hostname \"clsomimx6\";
    }
    ",
    );
    assert!(res.is_ok(), "{}", res.err().unwrap());

    let leases = res.unwrap().leases;

    assert_eq!(leases.count(), 2);
}
