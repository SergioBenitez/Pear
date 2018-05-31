#![feature(proc_macro)]
#![feature(proc_macro_non_items)]

#[macro_use] extern crate pear;
extern crate time;

use pear::Result;
use pear::parsers::*;
use pear::{parser, switch};

declare!(Input<'a>(Token = char, Slice = &'a str, Many = &'a str));

#[parser]
fn parens<'a, I: Input<'a>>(input: &mut I) -> Result<(), I> {
    eat('(')?;
    pear_try!(parens());
    eat(')')?;
}

fn main() {
    let result = parse!(parens: &mut ::pear::Text::from("((((()))))"));
    if let Err(e) = result { println!("Error: {}", e); }

    let result = parse!(parens: &mut ::pear::Text::from("((())))"));
    if let Err(e) = result { println!("Error: {}", e); }

    let result = parse!(parens: &mut ::pear::Text::from("(((()))"));
    if let Err(e) = result { println!("Error: {}", e); }
}
