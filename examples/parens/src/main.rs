#![feature(proc_macro_hygiene)]
#![warn(rust_2018_idioms)]

use pear::{input::Text, result::Result};
use pear::macros::{parser, parse, parse_declare, parse_try};
use pear::parsers::*;

parse_declare!(Input<'a>(Token = char, Slice = &'a str, Many = &'a str));

#[parser]
fn parens<'a, I: Input<'a>>(input: &mut I) -> Result<(), I> {
    eat('(')?;
    parse_try!(parens());
    eat(')')?;
}

mod try_it {
    use pear::result::Result;
    use pear::macros::{parser, switch, parse_error};
    use pear::parsers::*;

    use pear::macros::parse_declare;
    parse_declare!(Input(Token = char, Slice = &'static str, Many = String));

    #[parser]
    fn keyword<I: Input>(input: &mut I) -> Result<String, I> {
        switch! {
            kw@eat_slice("do") | kw@eat_slice("for") => kw.to_string(),
            _ => return parse_error!("unknown keyword")
        }
    }
}

fn main() {
    let result = parse!(parens: &mut Text::from("((((()))))"));
    if let Err(e) = result { println!("Error 0: {}", e); }

    let result = parse!(parens: &mut Text::from("((())))"));
    if let Err(e) = result { println!("Error 1: {}", e); }

    let result = parse!(parens: &mut Text::from("(((()))"));
    if let Err(e) = result { println!("Error 2: {}", e); }
}
