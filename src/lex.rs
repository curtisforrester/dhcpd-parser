use std::fmt;
use std::iter::Peekable;

use crate::leases::LeaseKeyword;
use crate::parser::ConfigKeyword;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexItem {
    Paren(char),
    Endl,
    Word(String),
    Opt(LeaseKeyword),
    Decl(ConfigKeyword),
}

impl fmt::Display for LexItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LexItem::Paren(v) => v.fmt(f),
            LexItem::Word(v) => v.fmt(f),
            LexItem::Opt(v) => write!(f, "{}", v.to_string()),
            LexItem::Decl(v) => write!(f, "{}", v.to_string()),
            LexItem::Endl => write!(f, ";"),
        }
    }
}

pub fn lex<S>(input: S) -> Result<Vec<LexItem>, String>
where
    S: Into<String>,
{
    let mut result = Vec::new();
    let input_str = input.into();
    let mut it = input_str.chars().peekable();

    while let Some(&c) = it.peek() {
        match c {
            '(' | ')' | '[' | ']' | '{' | '}' => {
                result.push(LexItem::Paren(c));
                it.next();
            }
            '#'  => {
                consume_comment(&mut it);
            }
            ' ' | '\n' | '\t' => {
                it.next();
            }
            ';' => {
                result.push(LexItem::Endl);
                it.next();
            }
            _ => {
                let w = get_word(&mut it);

                // Is this the keyword "lease"?
                let kw = ConfigKeyword::from(&w);
                if kw.is_ok() {
                    result.push(LexItem::Decl(kw.unwrap()));
                } else {
                    // Is this one of the other valid `LeaseKeyword` words?
                    let kw = LeaseKeyword::from(&w);
                    if kw.is_ok() {
                        result.push(LexItem::Opt(kw.unwrap()));
                    } else {
                        result.push(LexItem::Word(w));
                    }
                }
            }
        }
    }
    Ok(result)
}

/// Get the next word up to either whitespace or a line terminator ';"
fn get_word<T: Iterator<Item = char>>(iter: &mut Peekable<T>) -> String {
    let mut word = String::new();

    while let Some(&nc) = iter.peek() {
        if nc.is_whitespace() || nc == ';' {
            break;
        }

        word.push(nc);
        iter.next();
    }
    word
}

/// Advance iterator past comment. The iterator will be sitting either on a '\n' or EOF.
fn consume_comment<T: Iterator<Item = char>>(iter: &mut Peekable<T>) -> () {
    // Advance past the '#' we peeked at
    iter.next();

    // Skip until we peek a '\n'
    // while let Some(&nc) = iter.peek() {
    //     match iter.next_if(|&k | k != '\n') {
    //         Some(state) => (),
    //         None => break
    //     };
    // }

    while let Some(_nc) = iter.next_if(|&k | k != '\n') {}
}
