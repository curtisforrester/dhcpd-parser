use crate::leases::parse_lease;
use crate::leases::Lease;
use crate::leases::Leases;
pub use crate::leases::LeasesMethods;
use crate::lex::lex;
use crate::lex::LexItem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserResult {
    pub leases: Leases,
}

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
            _ => {
                return Err(format!("Unexpected {:?}", it.peek()));
            }
        }
    }

    Ok(ParserResult { leases })
}

pub fn parse<S>(input: S) -> Result<ParserResult, String>
where
    S: Into<String>,
{
    let tokens = lex(input).unwrap();
    return parse_config(tokens);
}
