//! # DHCPD Leases Parser
//!
//! Library for parsing the contents of a "dhcpd.leases" file. Provides support for BSD and Linux
//! formats. The Linux support is (currently) for IPv4 only - the implementation does not attempt
//! to be complete and exhaustive per either the Linux or ISC specifications. It should be sufficient
//! for reading the contents of a leases file.
//!
//! # Usage
//!
//! Add the library to your project's Cargo.toml (as usual). Refer the examples for more details.
//!
//! ```rust
//! use crate::dhcpd_parser::parser;
//! use dhcpd_parser::leases::LeasesMethods;
//! 
//! pub fn main() {
//!     println!("Simple example to parse leases from string");
//! 
//!     let res = parser::parse(
//!         "
//!     lease 192.168.4.105 {
//!       starts 3 2022/01/05 16:51:33;
//!       ends 3 2022/01/05 18:51:33;
//!       tstp 3 2022/01/05 18:51:33;
//!       cltt 3 2022/01/05 16:51:33;
//!       binding state free;
//!       hardware ethernet 00:ed:d4:39:0d:04;
//!       uid \"\\001\\000\\352\\3249\\015\\004\";
//!       reserved a b c d;
//!     }
//! 
//!     lease 192.168.4.108 {
//!       starts 6 2022/01/08 17:46:16;
//!       ends 6 2022/01/08 17:56:16;
//!       cltt 6 2022/01/08 17:46:16;
//!       binding state active;
//!       next binding state free;
//!       rewind binding state free;
//!       hardware ethernet 00:ed:d4:39:0d:04;
//!       client-hostname \"clsomimx6\";
//!     }
//!     ",
//!     );
//!
//!     assert!(res.is_ok(), "{}", res.err().unwrap());
//! 
//!     let leases = res.unwrap().leases;
//!     // Expecting two leases
//!     assert_eq!(leases.count(), 2);
//! }
//! ```
//! 
//! ## References
//!
//! * [BSD leases man page](https://man.openbsd.org/dhcpd.leases.5)
//! * [Linux leases man page](https://linux.die.net/man/5/dhcpd.leases)
//! * [ISC leases man page](https://manpages.debian.org/testing/isc-dhcp-server/dhcpd.leases.5.en.html)
//! * [DHCPD Wikipedia](https://en.wikipedia.org/wiki/DHCPD)
//! * [ISC DHCP](https://www.isc.org/dhcp/)

pub mod common;
pub mod leases;
pub mod parser;

mod lex;
// TODO: Wrap this as an optional feature
pub mod util;
