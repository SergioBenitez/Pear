#![feature(plugin)]
#![plugin(pear_codegen)]

#[macro_use] extern crate pear;

use pear::ParseResult;
use pear::parsers::*;

#[derive(Debug)]
struct MediaType<'s> {
    top: &'s str,
    sub: &'s str,
    params: Vec<(&'s str, &'s str)>
}

#[inline]
fn is_valid_byte(c: char) -> bool {
    match c {
        '0'...'9' | 'a'...'z' | '^'...'~' | '#'...'\''
            | '!' | '*' | '+' | '-' | '.'  => true,
        _ => false
    }
}

#[inline(always)]
fn is_whitespace(byte: char) -> bool {
    byte == ' ' || byte == '\t'
}

#[parser]
fn quoted_string<'a>(input: &mut &'a str) -> ParseResult<&'a str, &'a str> {
    delimited('"', '"')
}

#[parser]
fn media_type<'a>(input: &mut &'a str) -> ParseResult<&'a str, MediaType<'a>> {
    let top = take_some_while(|c| is_valid_byte(c) && c != '/');
    eat('/');
    let sub = take_some_while(is_valid_byte);

    // OWS* ; OWS*
    let mut params = Vec::new();
    try_repeat! {
        skip_while(is_whitespace);
        eat(';');
        skip_while(is_whitespace);

        let key = take_some_while(|c| is_valid_byte(c) && c != '=');
        eat('=');

        let value = switch! {
            peek('"') => quoted_string(),
            _ => take_some_while(|c| is_valid_byte(c) && c != ';')
        };

        params.push((key, value))
    }

    MediaType { top: top, sub: sub, params: vec![] }
}

#[parser]
fn accept<'a>(input: &mut &'a str) -> ParseResult<&'a str, Vec<MediaType<'a>>> {
    let mut media_types = Vec::new();
    let _ = repeat! {
        let media_type = media_type();
        switch! {
            eat(',') => skip_while(is_whitespace),
            _ => ()
        };

        media_types.push(media_type)
    };

    media_types
}

fn main() {
    println!("MEDIA TYPE: {:?}", media_type(&mut "a/b; a=b; c=d"));
    println!("ACCEPT: {:?}", accept(&mut "a/b; a=b, c/d"));
}
