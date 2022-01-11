use crate::leases::parse_lease;
use crate::leases::Lease;
use crate::leases::Leases;
#[doc(inline)]
pub use crate::leases::LeasesMethods;
use crate::lex::lex;
use crate::lex::LexItem;

/// Result for success returning a [Leases] instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserResult {
    pub leases: Leases,
}

/// Keyword "lease"
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigKeyword {
    Lease,
}

impl ConfigKeyword {
    pub fn to_string(&self) -> String {
        match self {
            &ConfigKeyword::Lease => "lease".to_owned(),
        }
    }

    pub fn from(s: &str) -> Result<ConfigKeyword, String> {
        match s {
            "lease" => Ok(ConfigKeyword::Lease),
            _ => Err(format!("'{}' declaration is not supported", s)),
        }
    }
}

/// Parse the config represented by a vector of tokens.
fn parse_config(tokens: Vec<LexItem>) -> Result<ParserResult, String> {
    use crate::leases::LeaseKeyword;

    let mut leases = Leases::new();
    let lease = Lease::new();
    let mut bite_order : Option<String> = None;

    let mut it = tokens.iter().peekable();

    while let Some(token) = it.peek() {
        match token {
            LexItem::Decl(ConfigKeyword::Lease) => {
                if lease != Lease::new() {
                    leases.push(lease.clone());
                }

                let mut lease = Lease::new();
                // ip-address
                it.next();
                lease.ip = it.peek().expect("IP address expected").to_string();

                // left curly brace
                it.next();
                assert_eq!(it.peek().unwrap().to_owned(), &LexItem::Paren('{'));

                // statements for the lease
                it.next();
                parse_lease(&mut lease, &mut it)?;

                // right curly brace
                if it.peek().is_none() || it.peek().unwrap().to_owned() != &LexItem::Paren('}') {
                    return Err(format!(
                        "Expected end of section with '}}', got '{:?}'",
                        it.peek(),
                    ));
                }

                // Set the bite_order if supplied (Linux only)
                lease.byte_order = bite_order.clone();

                leases.push(lease.clone());
                it.next();
            }
            LexItem::Opt(LeaseKeyword::BiteOrder) => {
                it.next();
                match it.next_if(|&k | k != &LexItem::Endl) {
                    Some(val) => {
                        // println!("Found: author-bite-order: {}", val.to_string());
                        bite_order = Some(val.to_string());
                    },
                    None => return Err(format!("Expected author-bite-order value. Found endl instead"))
                };
                match it.next_if_eq(&&LexItem::Endl) {
                    Some(_) => (),
                    None => return Err(format!("Expected semicolon after author-bite-order term")),
                }
            }
            LexItem::Opt(LeaseKeyword::Ignored) => {
                it.next();
                // Consume up to the endl
                loop {
                    match it.next_if(|&k | k != &LexItem::Endl && k != &LexItem::Paren('}')) {
                        Some(_) => (),  // println!("Skipping: {}", token),
                        None => break
                    };
                }
                it.next();
            }
            _ => {
                return Err(format!("Unexpected {:?}", it.peek()));
            }
        }
    }

    Ok(ParserResult { leases })
}

/// Parse the String containing the contents of the leases file.
///
/// # Overview
///
/// The "dhcpd.leases" contents should be loaded into a [String] to pass into this function. Note
/// that this library does not support loading the leases file. On Linux the default file is generally
/// found in "/var/lib/dhcp/dhcpd.leases".
///
/// This will parse the contents of the dhcpd.leases file and, if successful, return a [ParserResult]
/// with the [Leases] instance (a vector of [Lease] instances).
///
/// The parser will correctly ignore comments found in the "dhcpd.leases" file.
///
/// # Simple Example
///
/// The following is a simple illustration. (The "load" here simply hard-codes a string.)
///
/// ```rust
/// use crate::dhcpd_parser::parser;
/// use dhcpd_parser::leases::LeasesMethods;
///
/// fn load_leases_file() -> String {
///     String::from("lease 192.168.0.2 {
///         starts 2 2019/01/01 22:00:00 UTC;
///         ends 2 2019/01/01 23:00:00 UTC;
///         hardware type 11:11:11:11:11:11;
///         uid Client1;
///         client-hostname \"CLIENTHOSTNAME\";
///         hostname \"TESTHOSTNAME\";
///         abandoned;
///     }")
/// }
///
/// fn main() {
///     let my_contents = load_leases_file();
///     let res = parser::parse(&my_contents);
///     let leases = res.unwrap().leases;
///
///     println!("Loaded {} leases", leases.count());
/// }
/// ```
///
/// # Caveats
///
/// ## On Support for Keywords
///
/// While the library supports both BSD/Linux-style servers, it does not as-yet attempt to support the
/// full specifications of the Linux/ISC servers (or your specific implementation you use). The known keywords
/// that are not yet supported are in an ignored list; lines starting with these keywords will be skipped.
///
/// There is the possibility that your server adds keywords that this parser does not recognize. This will
/// result in an error. A feature that is tracked is to add support to either simply warn on unrecognized
/// keywords, or to add these to a vector on the [Lease] struct for a user's special case handling. I recommend
/// that if this scenario happens, the content of the file should have these lines stripped out before passing
/// the string into this function. (For example: `cat dhcpd.leases | grep -v "<unknown-keyword>"`)
///
/// See:
/// * [BSD leases man page](https://man.openbsd.org/dhcpd.leases.5)
/// * [Linux leases man page](https://linux.die.net/man/5/dhcpd.leases)
/// * [ISC leases man page](https://manpages.debian.org/testing/isc-dhcp-server/dhcpd.leases.5.en.html)
///
/// ## On "panic!"
///
/// This function currently can "panic!" on certain errors. This will be removed in a future release. This is
/// generally when the file has a keyword/syntax error. Please file an issue if you fina a "panic!" on a
/// valid formatted leases file.
pub fn parse<S>(input: S) -> Result<ParserResult, String>
where
    S: Into<String>,
{
    let tokens = lex(input).unwrap();
    return parse_config(tokens);
}
