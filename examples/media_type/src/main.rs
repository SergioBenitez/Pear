#![feature(proc_macro)]
#![feature(proc_macro_non_items)]

#[macro_use] extern crate pear;

use pear::{Result, parser, switch};
use pear::parsers::*;

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

declare!(Input<'a>(Token = char, Slice = &'a str, Many = &'a str));

#[parser]
fn quoted_string<'a, I: Input<'a>>(input: &mut I) -> Result<&'a str, I> {
    eat('"')?;

    let mut is_escaped = false;
    let inner = take_while(|c| {
        if is_escaped { is_escaped = false; return true; }
        if c == '\\' { is_escaped = true; return true; }
        c != '"'
    })?;

    eat('"')?;
    inner
}

#[parser]
fn media_param<'a, I: Input<'a>>(input: &mut I) -> Result<(&'a str, &'a str), I> {
    let key = (take_some_while_until(is_valid_token, '=')?, eat('=')?).0;
    let value = switch! {
        peek('"') => quoted_string()?,
        _ => take_some_while_until(is_valid_token, ';')?
    };

    (key, value)
}

#[parser]
fn media_type<'a, I: Input<'a>>(input: &mut I) -> Result<MediaType<'a>, I> {
    MediaType {
        top: take_some_while_until(is_valid_token, '/')?,
        sub: (eat('/')?, take_some_while_until(is_valid_token, ';')?).1,
        params: series(true, ';', is_whitespace, media_param)?
    }
}

#[parser]
fn weighted_media_type<'a, I: Input<'a>>(input: &mut I) -> Result<(MediaType<'a>, Option<f32>), I> {
    let media_type = media_type()?;
    let weight = match media_type.params.iter().next() {
        Some(&("q", value)) => match value.parse::<f32>().ok() {
            Some(q) if q > 1.0 => return Err(pear_error!("media-type weight >= 1.0")),
            Some(q) => Some(q),
            None => return Err(pear_error!("invalid media-type weight"))
        },
        _ => None
    };

    (media_type, weight)
}

#[parser]
fn accept<'a, I: Input<'a>>(input: &mut I) -> Result<Vec<(MediaType<'a>, Option<f32>)>, I> {
    Ok(series(false, ',', is_whitespace, weighted_media_type)?)
}

fn main() {
    println!("MEDIA TYPE: {:?}", parse!(media_type: &mut ::pear::Text::from("a/b; a=\"abc\"; c=d")));
    println!("MEDIA TYPE: {:?}", parse!(media_type: &mut "a/b; a=\"ab=\\\"c\\\"\"; c=d"));
    println!("MEDIA TYPE: {:?}", parse!(media_type: &mut "a/b; a=b; c=d"));
    println!("ACCEPT: {:?}", accept(&mut "a/b; a=b, c/d"));
    println!("ACCEPT: {:?}", accept(&mut "a/b; q=0.7, c/d"));
}
