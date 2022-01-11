use std::collections::HashSet;
use std::iter::Peekable;
use std::ops::Index;

use crate::common::Date;
use crate::lex::LexItem;
use chrono::prelude::*;

// TODO: I'm thinking I might write a serialize/deserialize library with "nom" and support serde instead.

/// Keywords for in a Lease entry line
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeaseKeyword {
    Abandoned,
    ClientHostname,
    Ends,
    Hardware,
    Hostname,
    Starts,
    Uid,
    Binding,
    State,
    Next,
    Rewind,
    Tstp,
    Tsfp,
    Atsfp,
    Cltt,
    BiteOrder,
    /// The keyword is one of the set we are ignoring/not supporting
    /// (i.e., we ignore the kw and all tokens through the next _Endl_ ";" character).
    Ignored,
}

impl LeaseKeyword {
    pub fn to_string(&self) -> String {
        match self {
            &LeaseKeyword::Abandoned => "abandoned".to_owned(),
            &LeaseKeyword::ClientHostname => "client-hostname".to_owned(),
            &LeaseKeyword::Ends => "ends".to_owned(),
            &LeaseKeyword::Hardware => "hardware".to_owned(),
            &LeaseKeyword::Hostname => "hostname".to_owned(),
            &LeaseKeyword::Starts => "starts".to_owned(),
            &LeaseKeyword::Uid => "uid".to_owned(),
            &LeaseKeyword::Binding => "binding".to_owned(),
            &LeaseKeyword::State => "state".to_owned(),
            &LeaseKeyword::Next => "next".to_owned(),
            &LeaseKeyword::Rewind => "rewind".to_owned(),
            &LeaseKeyword::Tstp => "tstp".to_owned(),
            &LeaseKeyword::Tsfp => "tsfp".to_owned(),
            &LeaseKeyword::Atsfp => "atsfp".to_owned(),
            &LeaseKeyword::Cltt => "cltt".to_owned(),
            &LeaseKeyword::BiteOrder => "authoring-byte-order".to_owned(),
            &LeaseKeyword::Ignored => "(ignored kw)".to_owned(),
        }
    }

    pub fn from(s: &str) -> Result<LeaseKeyword, String> {
        match s {
            "abandoned" => Ok(LeaseKeyword::Abandoned),
            "client-hostname" => Ok(LeaseKeyword::ClientHostname),
            "ends" => Ok(LeaseKeyword::Ends),
            "hardware" => Ok(LeaseKeyword::Hardware),
            "hostname" => Ok(LeaseKeyword::Hostname),
            "starts" => Ok(LeaseKeyword::Starts),
            "uid" => Ok(LeaseKeyword::Uid),
            "binding" => Ok(LeaseKeyword::Binding),
            "state" => Ok(LeaseKeyword::State),
            "next" => Ok(LeaseKeyword::Next),
            "rewind" => Ok(LeaseKeyword::Rewind),
            "tstp" => Ok(LeaseKeyword::Tstp),
            "tsfp" => Ok(LeaseKeyword::Tsfp),
            "atsfp" => Ok(LeaseKeyword::Atsfp),
            "cltt" => Ok(LeaseKeyword::Cltt),
            "authoring-byte-order" => Ok(LeaseKeyword::BiteOrder),
            // Doubtful we will need support
            "option" | "set" | "on" | "bootp" | "reserved" | "failover" | "server-duid" => Ok(LeaseKeyword::Ignored),
            _ => Err(format!("'{}' is not a recognized lease option", s)),
        }
    }
}

/// The dates found in the lease entry. The only two that we retain are the "starts" and "ends"
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaseDates {
    /// The lease start DTS
    pub starts: Option<Date>,
    /// The lease end DTS
    pub ends: Option<Date>,
    /// The time the peer has been told the lease expires when failover protocol is used
    pub tstp: Option<Date>,
    /// The lease expiry time that the peer has acknowledged when failover protocol is used
    pub tsfp: Option<Date>,
    /// The actual time sent from the failover partner
    pub atsfp: Option<Date>,
    /// The client's last transaction time
    pub cltt: Option<Date>,
}

/// The "hardware" entry in the lease
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hardware {
    /// The type (i.e., "ethernet")
    pub h_type: String,
    /// The MAC address
    pub mac: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LeasesField {
    ClientHostname,
    Hostname,
    LeasedIP,
    MAC,
}

impl LeasesField {
    fn value_getter(&self) -> Box<dyn Fn(&Lease) -> Option<String>> {
        match &self {
            LeasesField::ClientHostname => {
                Box::new(|l: &Lease| -> Option<String> { l.client_hostname.clone() })
            }
            LeasesField::Hostname => Box::new(|l: &Lease| -> Option<String> { l.hostname.clone() }),
            LeasesField::LeasedIP => Box::new(|l: &Lease| -> Option<String> { Some(l.ip.clone()) }),
            LeasesField::MAC => Box::new(|l: &Lease| -> Option<String> {
                match &l.hardware {
                    Some(h) => Some(h.mac.clone()),
                    None => None,
                }
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Leases(Vec<Lease>);

impl Index<usize> for Leases {
    type Output = Lease;

    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}

pub trait LeasesMethods {
    fn all(&self) -> Vec<Lease>;

    #[deprecated(since = "0.4.3", note="any filtering logic should be done by user")]
    fn active_by<S: AsRef<str>>(
        &self,
        field_name: LeasesField,
        value: S,
        active_at: Date,
    ) -> Option<Lease>;

    #[deprecated(since = "0.4.3", note="any filtering logic should be done by user")]
    fn by_leased<S: AsRef<str>>(&self, ip: S) -> Option<Lease>;
    #[deprecated(since = "0.4.3", note="any filtering logic should be done by user")]
    fn by_leased_all<S: AsRef<str>>(&self, ip: S) -> Vec<Lease>;

    #[deprecated(since = "0.4.3", note="any filtering logic should be done by user")]
    fn by_mac<S: AsRef<str>>(&self, mac: S) -> Option<Lease>;
    #[deprecated(since = "0.4.3", note="any filtering logic should be done by user")]
    fn by_mac_all<S: AsRef<str>>(&self, mac: S) -> Vec<Lease>;

    #[deprecated(since = "0.4.3", note="any filtering logic should be done by user")]
    fn active_by_hostname<S: AsRef<str>>(&self, hostname: S, active_at: Date) -> Option<Lease>;
    #[deprecated(since = "0.4.3", note="any filtering logic should be done by user")]
    fn by_hostname_all<S: AsRef<str>>(&self, hostname: S) -> Vec<Lease>;

    #[deprecated(since = "0.4.3", note="any filtering logic should be done by user")]
    fn active_by_client_hostname<S: AsRef<str>>(
        &self,
        hostname: S,
        active_at: Date,
    ) -> Option<Lease>;
    #[deprecated(since = "0.4.3", note="any filtering logic should be done by user")]
    fn by_client_hostname_all<S: AsRef<str>>(&self, hostname: S) -> Vec<Lease>;

    fn new() -> Leases;
    fn push(&mut self, l: Lease);
    fn hostnames(&self) -> HashSet<String>;
    fn client_hostnames(&self) -> HashSet<String>;

    fn count(&self) -> usize;
}

impl LeasesMethods for Leases {
    fn all(&self) -> Vec<Lease> {
        self.0.clone()
    }

    /// Returns a lease by some field and it's value if it exists.
    ///
    /// The lease has to be active:
    ///
    /// - `active_at` is between it's `starts` and `ends` datetime
    /// - is not `abandoned`
    /// - no active leases that match the field value exist after it
    fn active_by<S: AsRef<str>>(
        &self,
        field: LeasesField,
        value: S,
        active_at: Date,
    ) -> Option<Lease> {
        let expected_val = value.as_ref();
        let get_val = field.value_getter();

        let mut ls = self.0.clone();
        ls.reverse();

        for l in ls {
            if l.is_active_at(active_at) && !l.abandoned {
                let val = get_val(&l);
                if val.is_some() && val.unwrap() == expected_val {
                    return Some(l);
                }
            }
        }

        None
    }

    fn by_leased<S: AsRef<str>>(&self, ip: S) -> Option<Lease> {
        let mut ls = self.0.clone();
        ls.reverse();

        for l in ls {
            if l.ip == ip.as_ref() {
                return Some(l);
            }
        }

        None
    }

    fn by_leased_all<S: AsRef<str>>(&self, ip: S) -> Vec<Lease> {
        let mut result = Vec::new();
        let ls = self.0.clone();

        for l in ls {
            if l.ip == ip.as_ref() {
                result.push(l);
            }
        }

        return result;
    }

    fn by_mac<S: AsRef<str>>(&self, mac: S) -> Option<Lease> {
        let mut ls = self.0.clone();
        ls.reverse();

        for l in ls {
            let hw = l.hardware.as_ref();
            if hw.is_some() && hw.unwrap().mac == mac.as_ref() {
                return Some(l);
            }
        }

        None
    }

    fn by_mac_all<S: AsRef<str>>(&self, mac: S) -> Vec<Lease> {
        let mut result = Vec::new();
        let ls = self.0.clone();

        for l in ls {
            let hw = l.hardware.as_ref();
            if hw.is_some() && hw.unwrap().mac == mac.as_ref() {
                result.push(l);
            }
        }

        return result;
    }

    fn active_by_hostname<S: AsRef<str>>(&self, hostname: S, active_at: Date) -> Option<Lease> {
        #[allow(deprecated)]
        self.active_by(LeasesField::Hostname, hostname, active_at)
    }

    fn by_hostname_all<S: AsRef<str>>(&self, hostname: S) -> Vec<Lease> {
        let mut res = Vec::new();
        let ls = self.0.clone();
        let hn_s = hostname.as_ref();

        for l in ls {
            let hn = l.hostname.as_ref();
            if hn.is_some() && hn.unwrap() == hn_s {
                res.push(l);
            }
        }

        res
    }

    fn active_by_client_hostname<S: AsRef<str>>(
        &self,
        hostname: S,
        active_at: Date,
    ) -> Option<Lease> {
        #[allow(deprecated)]
        self.active_by(LeasesField::ClientHostname, hostname, active_at)
    }

    fn by_client_hostname_all<S: AsRef<str>>(&self, hostname: S) -> Vec<Lease> {
        let mut res = Vec::new();
        let ls = self.0.clone();
        let hn_s = hostname.as_ref();

        for l in ls {
            let hn = l.client_hostname.as_ref();
            if hn.is_some() && hn.unwrap() == hn_s {
                res.push(l);
            }
        }

        res
    }

    fn new() -> Leases {
        Leases(Vec::new())
    }

    fn push(&mut self, l: Lease) {
        self.0.push(l);
    }

    fn hostnames(&self) -> HashSet<String> {
        let mut res = HashSet::new();
        let ls = self.0.clone();

        for l in ls {
            if l.hostname.is_some() {
                res.insert(l.hostname.unwrap());
            }
        }

        return res;
    }

    fn client_hostnames(&self) -> HashSet<String> {
        let mut res = HashSet::new();
        let ls = self.0.clone();

        for l in ls {
            if l.client_hostname.is_some() {
                res.insert(l.client_hostname.unwrap());
            }
        }

        return res;
    }

    fn count(&self) -> usize {
        self.0.len()
    }
}

/// A lease entry from the dhcpd.leases file, and contained within a [Leases] instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lease {
    /// The IP associated with the lease entry.
    pub ip: String,
    /// The dates found in the lease entry.
    pub dates: LeaseDates,
    /// The "hardware" entry in the lease.
    pub hardware: Option<Hardware>,
    /// The uid statement records the client identifier used by the client to acquire the lease.
    pub uid: Option<String>,
    /// The hostname, if the client sends the Client Hostname option.
    pub client_hostname: Option<String>,
    /// The hostname, if the client sends the Hostname option.
    pub hostname: Option<String>,
    /// Flag to indicate the server has abandoned the lease due to a detected conflict.
    /// From [Linux dhcpd.conf man page](https://linux.die.net/man/5/dhcpd.conf)
    ///
    /// > _"the IP address is in use by some host on the network that is not a DHCP client.
    /// > It marks the address as abandoned, and will not assign it to clients."_
    pub abandoned: bool,
    /// The "binding state" (_Linux only_)
    pub binding_state: Option<String>,
    /// The "next binding state" (_Linux only_)
    pub next_binding_state: Option<String>,
    /// The "rewind binding state" (_Linux only_)
    pub rewind_binding_state: Option<String>,
    /// Linux only: Contains the value from "authoring-byte-order" - either "little-endian" or "big-endian"
    pub byte_order: Option<String>,
}

impl Lease {
    /// Create a new instances with defaults.
    pub fn new() -> Lease {
        Lease {
            ip: "localhost".to_owned(),
            dates: LeaseDates {
                starts: None,
                ends: None,
                atsfp: None,
                cltt: None,
                tsfp: None,
                tstp: None,
            },
            hardware: None,
            uid: None,
            client_hostname: None,
            hostname: None,
            abandoned: false,
            binding_state: None,
            next_binding_state: None,
            rewind_binding_state: None,
            byte_order: None,
        }
    }

    /// True if the lease is active at a [Date]
    pub fn is_active_at(&self, when: Date) -> bool {
        if self.dates.starts.is_some() && self.dates.starts.unwrap() > when {
            return false;
        }

        if self.dates.ends.is_some() && self.dates.ends.unwrap() < when {
            return false;
        }

        return true;
    }

    /// Helper method to give an indication if the loaded leases file is for Linux
    pub fn is_linux(&self) -> bool {
        return match self.binding_state {
            Some(_) => true,
            None  => false
        }
    }

    /// Helper to get the client MAC hardware identifier (MAC)
    pub fn client(&self) -> String {
        return self.hardware.as_ref().unwrap().mac.to_owned()
    }

    /// Helper to get the "ends" date as a [chrono::DateTime]
    pub fn lease_end_dts(&self) -> DateTime<Utc> {
        return self.dates.ends.unwrap().to_chrono()
    }

    /// Indicates if the lease is currently active (true), or expired (false)
    /// Linux NOTE: By observation, multiple leases can be "active".
    pub fn is_active(&self) -> bool {
        if self.abandoned == true {
            return false;
        }

        // Linux only: Is there a value for binding_state?
        if let Some(binding_state) = &self.binding_state {
            match binding_state.as_str() {
                "active" => {},
                "free" => return false,
                other => println!("The value of binding_state is not recognized. Please file a bug! \"{}\"", other)
            }
        }

        // Compare ending DTS to now
        if let Some(end_dts) = self.dates.ends {
            let now = Utc::now();
            log::trace!("");
            return end_dts.to_chrono() > now;
        }

        // Note: Don't want to panic - but normally we would
        println!("Unable to accurately determine if lease is active. Please file a bug issue for the dhcpd-parser library.");
        println!("Reference: lease.dates.ends is None in 'Lease::is_active'");

        false
    }

    pub fn active_after(&self, dt: DateTime<Utc>) -> bool {
        if let Some(end_dts) = self.dates.ends {
            return end_dts.to_chrono() > dt;
        }

        false
    }
}

fn parse_date<'l, T: Iterator<Item = &'l LexItem>>(iter: &mut Peekable<T>) -> Result<Date, String> {
    iter.next();
    let weekday = iter
        .peek()
        .expect("Weekday for end date expected")
        .to_string();
    iter.next();
    let date = iter.peek().expect("Date for end date expected").to_string();
    iter.next();
    let time = iter.peek().expect("Time for end date expected").to_string();
    iter.next();

    // Consume the next token if it's "UTC" (BSD style date)
    iter.next_if(|&k| String::from("UTC") == k.to_string());

    Date::from(weekday, date, time)
}

/// Check to see if the next token is an endl.
fn expect_endl<'l, T: Iterator<Item = &'l LexItem>>(iter: &mut Peekable<T>) -> Result<(), String> {
    return match iter.peek().unwrap() {
        LexItem::Endl => Ok(()),
        s => Err(format!("Expected semicolon, found {}", s.to_string())),
    }
}

// TODO: Remove all the calls to ".expect()" - they cause wide-spread panic. Replace with errors.
pub fn parse_lease<'l, T: Iterator<Item = &'l LexItem>>(
    lease: &mut Lease,
    iter: &mut Peekable<T>,
) -> Result<(), String> {
    while let Some(&nc) = iter.peek() {
        match nc {
            LexItem::Opt(LeaseKeyword::Starts) => {
                let dt = parse_date(iter)?;
                lease.dates.starts.replace(dt);
            }
            LexItem::Opt(LeaseKeyword::Ends) => {
                lease.dates.ends.replace(parse_date(iter)?);
                expect_endl(iter)?
            }
            LexItem::Opt(LeaseKeyword::Tstp) => {
                lease.dates.tstp.replace(parse_date(iter)?);
                expect_endl(iter)?
            }
            LexItem::Opt(LeaseKeyword::Cltt) => {
                lease.dates.cltt.replace(parse_date(iter)?);
                expect_endl(iter)?
            }
            LexItem::Opt(LeaseKeyword::Tsfp) => {
                lease.dates.tsfp.replace(parse_date(iter)?);
                expect_endl(iter)?
            }
            LexItem::Opt(LeaseKeyword::Atsfp) => {
                lease.dates.atsfp.replace(parse_date(iter)?);
                expect_endl(iter)?
            }
            LexItem::Opt(LeaseKeyword::Hardware) => {
                iter.next();
                let h_type = iter.peek().expect("Hardware type expected").to_string();
                iter.next();
                let mac = iter.peek().expect("MAC address expected").to_string();
                iter.next();
                expect_endl(iter)?;

                lease.hardware.replace(Hardware {
                    h_type,
                    mac,
                });
            }
            LexItem::Opt(LeaseKeyword::Uid) => {
                iter.next();
                lease
                    .uid
                    .replace(iter.peek().expect("Client identifier expected").to_string());

                iter.next();
                expect_endl(iter)?
            }
            LexItem::Opt(LeaseKeyword::ClientHostname) => {
                iter.next();
                lease.client_hostname.replace(unquote_value(
                    iter.peek().expect("Client hostname expected").to_string(),
                ));

                iter.next();
                expect_endl(iter)?
            }
            LexItem::Opt(LeaseKeyword::Hostname) => {
                iter.next();
                lease.hostname.replace(unquote_value(
                    iter.peek().expect("Hostname expected").to_string(),
                ));

                iter.next();
                expect_endl(iter)?
            }
            LexItem::Opt(LeaseKeyword::Abandoned) => {
                lease.abandoned = true;
                iter.next();
                expect_endl(iter)?
            }
            LexItem::Opt(LeaseKeyword::Binding) => {
                iter.next();
                if Option::None == iter.next_if(|&k| String::from("state") == k.to_string()) {
                    return Err(format!("Expected kw 'state'. Found {}", iter.peek().unwrap().to_string()))
                }
                match iter.next_if(|&k | k != &LexItem::Endl) {
                    Some(state) => lease.binding_state.replace(state.to_string()),
                    None => return Err(format!("Expected binding state value. Found endl instead"))
                };
                expect_endl(iter)?
            }
            LexItem::Opt(LeaseKeyword::Next) => {
                iter.next();
                if Option::None == iter.next_if(|&k| String::from("binding") == k.to_string()) {
                    return Err(format!("Expected kw 'binding'. Found {}", iter.peek().unwrap().to_string()))
                }
                if Option::None == iter.next_if(|&k| String::from("state") == k.to_string()) {
                    return Err(format!("Expected kw 'state'. Found {}", iter.peek().unwrap().to_string()))
                }
                match iter.next_if(|&k | k != &LexItem::Endl) {
                    Some(state) => lease.next_binding_state.replace(state.to_string()),
                    None => return Err(format!("Expected next binding state value. Found endl instead"))
                };
                expect_endl(iter)?
            }
            LexItem::Opt(LeaseKeyword::Rewind) => {
                iter.next();
                if Option::None == iter.next_if(|&k| String::from("binding") == k.to_string()) {
                    return Err(format!("Expected kw 'binding'. Found {}", iter.peek().unwrap().to_string()))
                }
                if Option::None == iter.next_if(|&k| String::from("state") == k.to_string()) {
                    return Err(format!("Expected kw 'state'. Found {}", iter.peek().unwrap().to_string()))
                }
                match iter.next_if(|&k | k != &LexItem::Endl) {
                    Some(state) => lease.rewind_binding_state.replace(state.to_string()),
                    None => return Err(format!("Expected rewind binding state value. Found endl instead"))
                };
                expect_endl(iter)?
            }
            LexItem::Paren('}') => {
                return Ok(());
            }
            LexItem::Opt(LeaseKeyword::Ignored) => {
                iter.next();

                // Consume up to the endl
                loop {
                    match iter.next_if(|&k | k != &LexItem::Endl && k != &LexItem::Paren('}')) {
                        Some(_) => (),  // println!("Skipping: {}", token),
                        None => break
                    };
                }
                expect_endl(iter)?
            }
            _ => {
                let msg = format!(
                    "Unexpected option '{}'",
                    iter.peek().unwrap().to_string()
                );

                return Err(msg);
            }
        }
        iter.next();
    }

    Ok(())
}

fn unquote_value(v: String) -> String {
    v.replace("\"", "")
}
