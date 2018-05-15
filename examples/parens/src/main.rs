#![feature(proc_macro)]
#![feature(proc_macro_non_items)]

#[macro_use] extern crate pear;
extern crate time;

use pear::Result;
use pear::parsers::*;
use pear::{parser, switch};

declare!(Input<'a>(Token = char, Slice = &'a str, Many = &'a str));

// FIXME: Make this possible without the `input`. I think this is a rustc bug,
// actually.
macro_rules! pear_try {
    ($input:expr, $e:expr) => {{
        let input = &mut *$input;
        switch! { $e => {  }, _ => {  } }
    }};
    ($input:expr, $e:expr => $r:expr) => (
        switch! { $e => { Some($r) }, _ => { None } }
    );
    ($input:expr, $pat:ident@$e:expr => $r:expr) => (
        switch! { $pat@$e => { Some($r) }, _ => { None } }
    )
}

#[parser]
fn parens<'a, I: Input<'a>>(input: &mut I) -> Result<(), I> {
    eat('(')?;
    pear_try!(input, parens());
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
