//! # DHCPD Leases Parser
//!
//! Library for parsing the contents of a "dhcpd.leases" file. Provides support for BSD and Linux
//! formats. The Linux support is (currently) for IPv4 only - the implementation does not attempt
//! to be complete and exhaustive per either the Linux or ISC specifications. It should be sufficient
//! for reading the contents of a leases file.
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
