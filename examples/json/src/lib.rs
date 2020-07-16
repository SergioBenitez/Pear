#![warn(rust_2018_idioms)]

use std::collections::HashMap;

use pear::input::Result;
use pear::macros::{parser, switch, parse_declare, parse_error};
use pear::combinators::*;
use pear::parsers::*;

#[derive(Debug, PartialEq)]
pub enum JsonValue<'a> {
    Null,
    Bool(bool),
    Number(f64),
    String(&'a str),
    Array(Vec<JsonValue<'a>>),
    Object(HashMap<&'a str, JsonValue<'a>>)
}

#[inline(always)]
fn is_whitespace(&c: &char) -> bool {
    c.is_ascii_whitespace()
}

#[inline(always)]
fn is_num(c: &char) -> bool {
    c.is_ascii_digit()
}

parse_declare!(pub Input<'a>(Token = char, Slice = &'a str, Many = &'a str));

#[parser]
fn int<'a, I: Input<'a>>(input: &mut I) -> Result<i64, I> {
    take_some_while(is_num)?.parse().or_else(|e| parse_error!("{}", e)?)
    // take_some_while(|c| ('0'..='9').contains(c)); // BENCH
    // 1 // BENCH
}

#[parser]
fn signed_int<'a, I: Input<'a>>(input: &mut I) -> Result<i64, I> {
    switch! { eat('-') => -int()?, _ => int()? } // NOT BENCH
    // (maybe!(eat('-')), int()).1 // BENCH
}

// This is terribly innefficient.
#[parser]
fn number<'a, I: Input<'a>>(input: &mut I) -> Result<f64, I> {
    let whole_num = signed_int()?;
    let frac = switch! { eat('.') => take_some_while(is_num)?, _ => "" };
    let exp = switch! { eat_if(|&c| "eE".contains(c)) => signed_int()?, _ => 0 };

    // NOT BENCH
    format!("{}.{}e{}", whole_num, frac, exp).parse()
        .or_else(|e| parse_error!("{}", e)?)

    // 0.0 // BENCH
}

#[parser]
fn string<'a, I: Input<'a>>(input: &mut I) -> Result<&'a str, I> {
    eat('"')?;

    let mut is_escaped = false;
    let inner = take_while(|&c| {
        if is_escaped { is_escaped = false; return true; }
        if c == '\\' { is_escaped = true; return true; }
        c != '"'
    })?;

    eat('"')?;
    inner
}

#[parser]
fn object<'a, I: Input<'a>>(input: &mut I) -> Result<HashMap<&'a str, JsonValue<'a>>, I> {
    Ok(delimited_collect('{', |i| {
        let key = surrounded(i, string, is_whitespace)?;
        let value = (eat(i, ':')?, surrounded(i, value, is_whitespace)?).1;
        Ok((key, value))
    }, ',', '}')?)
}

#[parser]
fn array<'a, I: Input<'a>>(input: &mut I) -> Result<Vec<JsonValue<'a>>, I> {
    Ok(delimited_collect('[', value, ',', ']')?)
}

#[parser]
pub fn value<'a, I: Input<'a>>(input: &mut I) -> Result<JsonValue<'a>, I> {
    skip_while(is_whitespace)?;
    let val = switch! {
        eat_slice("null") => JsonValue::Null,
        eat_slice("true") => JsonValue::Bool(true),
        eat_slice("false") => JsonValue::Bool(false),
        peek('{') => JsonValue::Object(object()?),
        peek('[') => JsonValue::Array(array()?),
        peek('"') => JsonValue::String(string()?),
        peek_if(|c| *c == '-' || is_num(c)) => JsonValue::Number(number()?),
        token@peek_any() => parse_error!("unexpected input: {:?}", token)?,
        _ => parse_error!("unknown input")?,
    };

    skip_while(is_whitespace)?;
    val
}
