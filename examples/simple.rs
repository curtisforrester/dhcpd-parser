//! Example of simple use of the DHCP leases parser from a static string.
extern crate dhcpd_parser;

use crate::dhcpd_parser::parser;
use dhcpd_parser::leases::LeasesMethods;

pub fn main() {
    println!("Simple example to parse leases from string");

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