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

fn main() {
    let result = parse!(parens: &mut Text::from("((((()))))"));
    if let Err(e) = result { println!("Error 0: {}", e); }

    let result = parse!(parens: &mut Text::from("((())))"));
    if let Err(e) = result { println!("Error 1: {}", e); }

    let result = parse!(parens: &mut Text::from("(((()))"));
    if let Err(e) = result { println!("Error 2: {}", e); }
}
