#![feature(proc_macro)]
#![feature(proc_macro_non_items)]

#[macro_use] extern crate pear;
extern crate time;

use pear::Result;
use pear::parsers::*;
use pear::{parser, switch};

declare!(Input<'a>(Token = char, Slice = &'a str, Many = &'a str));

macro_rules! pear_try {
    ([$name:ident; $input:expr] $e:expr) => {{
        switch! { [$name;$input] result@$e => { Some(result) }, _ => { None } }
    }};
    ([$name:ident; $input:expr] $e:expr => $r:expr) => {{
        switch! { [$name;$input] $e => { Some($r) }, _ => { None } }
    }};
    ([$name:ident; $input:expr] $pat:ident@$e:expr => $r:expr) => {{
        switch! { [$name;$input] $pat@$e => { Some($r) }, _ => { None } }
    }}
}

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
