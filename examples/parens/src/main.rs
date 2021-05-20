#![warn(rust_2018_idioms)]

use pear::input::{Text, Pear, Result};
use pear::macros::{parser, parse, parse_declare};
use pear::parsers::*;

parse_declare!(Input<'a>(Token = char, Slice = &'a str, Many = &'a str));

#[parser]
fn parens<'a, I: Input<'a>>(input: &mut Pear<I>) -> Result<(), I> {
    eat('(')?;

    // parse_try!(parens());
    pear::macros::switch! {
        _ if true => parens()?,
        _ => parens()?
    }

    eat(')')?;
}

fn main() {
    let result = parse!(parens: Text::from("((((()))))"));
    if let Err(e) = result { println!("Error 0: {}", e); }

    let result = parse!(parens: Text::from("((())))"));
    if let Err(e) = result { println!("Error 1: {}", e); }

    let result = parse!(parens: Text::from("(((()))"));
    if let Err(e) = result { println!("Error 2: {}", e); }
}
