#![feature(plugin)]
#![plugin(pear_codegen)]

#[macro_use] extern crate pear;

use pear::ParseResult;
use pear::parsers::*;
use pear::combinators::*;

#[derive(Debug)]
struct MediaType<'s> {
    top: &'s str,
    sub: &'s str,
    params: Vec<(&'s str, &'s str)>
}

#[inline]
fn is_valid_token(c: char) -> bool {
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
    eat('"');

    let mut is_escaped = false;
    let inner = take_while(|c| {
        if is_escaped { is_escaped = false; return true; }
        if c == '\\' { is_escaped = true; return true; }
        c != '"'
    });

    eat('"');
    inner
}

#[parser]
fn media_type<'a>(input: &mut &'a str) -> ParseResult<&'a str, MediaType<'a>> {
    let top = take_some_while(|c| is_valid_token(c) && c != '/');
    eat('/');
    let sub = take_some_while(is_valid_token);

    let mut params = vec![];
    switch_repeat! {
        surrounded(|i| eat(i, ';'), is_whitespace) => {
            let key = take_some_while(|c| is_valid_token(c) && c != '=');
            eat('=');

            let value = switch! {
                peek('"') => quoted_string(),
                _ => take_some_while(|c| is_valid_token(c) && c != ';')
            };

            params.push((key, value))
        },
        _ => break
    }

    MediaType { top: top, sub: sub, params: params }
}

// FIXME: Autogenerate this by default? Disable with #[parser(bare)]?
fn parse_media_type(mut input: &str) -> ParseResult<&str, MediaType> {
    parse!(&mut input, (media_type(), eof()).0)
}

#[parser]
fn accept<'a>(input: &mut &'a str) -> ParseResult<&'a str, Vec<(MediaType<'a>, Option<f32>)>> {
    let mut media_types = vec![];
    repeat_while!(eat(','), {
        skip_while(is_whitespace);
        let media_type = media_type();
        let weight = match media_type.params.iter().next() {
            Some(&("q", value)) => match value.parse::<f32>().ok() {
                Some(q) if q > 1.0 => parse_error!("accept", "a"),
                Some(q) => Some(q),
                None => parse_error!("accept", "b")
            },
            _ => None
        };

        media_types.push((media_type, weight));
    });

    media_types
}

fn main() {
    println!("MEDIA TYPE: {:?}", parse_media_type("a/b; a=\"abc\"; c=d"));
    println!("MEDIA TYPE: {:?}", parse_media_type("a/b; a=\"ab=\\\"c\\\"\"; c=d"));
    println!("MEDIA TYPE: {:?}", parse_media_type("a/b; a=b; c=d"));
    println!("ACCEPT: {:?}", accept(&mut "a/b; a=b, c/d"));
    println!("ACCEPT: {:?}", accept(&mut "a/b; q=0.7, c/d"));
}
