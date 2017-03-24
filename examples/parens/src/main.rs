#![feature(plugin)]
#![plugin(pear_codegen)]

#[macro_use] extern crate pear;
extern crate time;

use pear::{ParseResult, Input, Text};
use pear::parsers::*;

trait StrLikeInput<'a>: Input<Token=char, Slice=&'a str, Many=&'a str> {  }
impl<'a, T: Input<Token=char, Slice=&'a str, Many=&'a str> + 'a> StrLikeInput<'a> for T {  }

#[parser]
fn parens<'a, I: StrLikeInput<'a>>(input: &mut I, top: bool) -> ParseResult<I, usize> {
    eat('(');
    let n = maybe!(parens(false));
    eat(')');

    if top { eof(); }
    n.unwrap_or(0) + 1
}

fn main() {
    let start = time::precise_time_ns();
    let mut text = Text::from("(())");
    let result = parens(&mut text, true);
    let end = time::precise_time_ns();

    if let ParseResult::Error(ref e) = result {
        println!("Error: {}", e);
        println!("{}", text.context().unwrap());
    }

    // TODO: Make sure we can use the same parser for files and strings.
    println!("Result (in {}us): {:?}", (end - start) / 1000, result);
}
