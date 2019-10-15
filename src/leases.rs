use std::collections::HashSet;
use std::iter::Peekable;
use std::ops::Index;

use crate::common::Date;
use crate::lex::LexItem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeaseKeyword {
    Abandoned,
    ClientHostname,
    Ends,
    Hardware,
    Hostname,
    Starts,
    Uid,
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
            _ => Err(format!("'{}' is not a recognized lease option", s)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaseDates {
    pub starts: Option<Date>,
    pub ends: Option<Date>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hardware {
    pub h_type: String,
    pub mac: String,
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
    fn by_leased<S: AsRef<str>>(&self, ip: S) -> Option<Lease>;
    fn by_leased_all<S: AsRef<str>>(&self, ip: S) -> Vec<Lease>;

    fn by_mac<S: AsRef<str>>(&self, mac: S) -> Option<Lease>;
    fn by_mac_all<S: AsRef<str>>(&self, mac: S) -> Vec<Lease>;

    fn by_hostname<S: AsRef<str>>(&self, hostname: S) -> Option<Lease>;
    fn active_by_hostname<S: AsRef<str>>(&self, hostname: S, active_at: Date) -> Option<Lease>;

    fn by_client_hostname<S: AsRef<str>>(&self, hostname: S) -> Vec<Lease>;
    fn active_by_client_hostname<S: AsRef<str>>(&self, hostname: S, active_at: Date) -> Option<Lease>;

    fn new() -> Leases;
    fn push(&mut self, l: Lease);
    fn hostnames(&self) -> HashSet<String>;
}

impl LeasesMethods for Leases {
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
        let mut ls = self.0.clone();
        let hn_s = hostname.as_ref();
        ls.reverse();

        for l in ls {
            if l.is_active_at(active_at) {
                let hn = l.hostname.as_ref();
                if hn.is_some() && hn.unwrap() == hn_s {
                    return Some(l);
                }
            }
        }

        None
    }

    fn by_hostname<S: AsRef<str>>(&self, hostname: S) -> Option<Lease> {
        let mut ls = self.0.clone();
        let hn_s = hostname.as_ref();
        ls.reverse();

        for l in ls {
            let hn = l.hostname.as_ref();
            if hn.is_some() && hn.unwrap() == hn_s {
                return Some(l);
            }
        }

        None
    }

    fn active_by_client_hostname<S: AsRef<str>>(&self, hostname: S, active_at: Date) -> Option<Lease> {
        let mut ls = self.0.clone();
        let hn_s = hostname.as_ref();
        ls.reverse();

        for l in ls {
            if l.is_active_at(active_at) {
                let hn = l.client_hostname.as_ref();
                if hn.is_some() && hn.unwrap() == hn_s {
                    return Some(l);
                }
            }
        }

        None
    }

    fn by_client_hostname<S: AsRef<str>>(&self, hostname: S) -> Vec<Lease> {
        let mut res = Vec::new();
        let mut ls = self.0.clone();
        let hn_s = hostname.as_ref();
        ls.reverse();

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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lease {
    pub ip: String,
    pub dates: LeaseDates,
    pub hardware: Option<Hardware>,
    pub uid: Option<String>,
    pub client_hostname: Option<String>,
    pub hostname: Option<String>,
    pub abandoned: bool,
}

impl Lease {
    pub fn new() -> Lease {
        Lease {
            ip: "localhost".to_owned(),
            dates: LeaseDates {
                starts: None,
                ends: None,
            },
            hardware: None,
            uid: None,
            client_hostname: None,
            hostname: None,
            abandoned: false,
        }
    }

    pub fn is_active_at(&self, when: Date) -> bool {
        if self.dates.starts.is_some() && self.dates.starts.unwrap() > when {
            return false;
        }

        if self.dates.ends.is_some() && self.dates.ends.unwrap() < when {
            return false;
        }

        return true;
    }
}

pub fn parse_lease<'l, T: Iterator<Item = &'l LexItem>>(
    lease: &mut Lease,
    iter: &mut Peekable<T>,
) -> Result<(), String> {
    while let Some(&nc) = iter.peek() {
        match nc {
            LexItem::Opt(LeaseKeyword::Starts) => {
                iter.next();
                let weekday = iter
                    .peek()
                    .expect("Weekday for start date expected")
                    .to_string();
                iter.next();
                let date = iter
                    .peek()
                    .expect("Date for start date expected")
                    .to_string();
                iter.next();
                let time = iter
                    .peek()
                    .expect("Time for start date expected")
                    .to_string();
                iter.next();

                let tz = iter
                    .peek()
                    .expect("Timezone or semicolon expected")
                    .to_string();
                if tz != LexItem::Endl.to_string() {
                    iter.next();
                    // println!("tz: {:?}", iter.peek());
                    match iter.peek().expect("Semicolon expected") {
                        LexItem::Endl => (),
                        s => return Err(format!("Expected semicolon, found {}", s.to_string())),
                    }
                }

                lease.dates.starts.replace(Date::from(weekday, date, time)?);
            }
            LexItem::Opt(LeaseKeyword::Ends) => {
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
                let tz = iter
                    .peek()
                    .expect("Timezone or semicolon expected")
                    .to_string();

                if tz != LexItem::Endl.to_string() {
                    iter.next();
                    match iter.peek().expect("Semicolon expected") {
                        LexItem::Endl => (),
                        s => return Err(format!("Expected semicolon, found {}", s.to_string())),
                    }
                }

                lease.dates.ends.replace(Date::from(weekday, date, time)?);
            }
            LexItem::Opt(LeaseKeyword::Hardware) => {
                iter.next();
                let h_type = iter.peek().expect("Hardware type expected").to_string();
                iter.next();
                let mac = iter.peek().expect("MAC address expected").to_string();
                iter.next();
                match iter.peek().expect("Semicolon expected") {
                    LexItem::Endl => (),
                    s => return Err(format!("Expected semicolon, found {}", s.to_string())),
                }

                lease.hardware.replace(Hardware {
                    h_type: h_type,
                    mac: mac,
                });
            }
            LexItem::Opt(LeaseKeyword::Uid) => {
                iter.next();
                lease
                    .uid
                    .replace(iter.peek().expect("Client identifier expected").to_string());

                iter.next();
                match iter.peek().expect("Semicolon expected") {
                    LexItem::Endl => (),
                    s => return Err(format!("Expected semicolon, found {}", s.to_string())),
                }
            }
            LexItem::Opt(LeaseKeyword::ClientHostname) => {
                iter.next();
                lease.client_hostname.replace(unquote_hostname(
                    iter.peek().expect("Client hostname expected").to_string(),
                ));

                iter.next();
                match iter.peek().expect("Semicolon expected") {
                    LexItem::Endl => (),
                    s => return Err(format!("Expected semicolon, found {}", s.to_string())),
                }
            }
            LexItem::Opt(LeaseKeyword::Hostname) => {
                iter.next();
                lease.hostname.replace(unquote_hostname(
                    iter.peek().expect("Hostname expected").to_string(),
                ));

                iter.next();
                match iter.peek().expect("Semicolon expected") {
                    LexItem::Endl => (),
                    s => return Err(format!("Expected semicolon, found {}", s.to_string())),
                }
            }
            LexItem::Opt(LeaseKeyword::Abandoned) => {
                lease.abandoned = true;
                iter.next();
                match iter.peek().expect("Semicolon expected") {
                    LexItem::Endl => (),
                    s => return Err(format!("Expected semicolon, found {}", s.to_string())),
                }
            }
            LexItem::Paren('}') => {
                return Ok(());
            }
            _ => {
                return Err(format!(
                    "Unexpected option '{}'",
                    iter.peek().unwrap().to_string()
                ));
            }
        }
        iter.next();
    }

    Ok(())
}

fn unquote_hostname(hn: String) -> String {
    hn.replace("\"", "")
}
