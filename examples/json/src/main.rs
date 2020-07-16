#![warn(rust_2018_idioms)]
#![cfg_attr(test, feature(test))]

use std::collections::HashMap;

use pear::input::Result;
use pear::macros::{parser, switch, parse_declare, parse, parse_error};
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

parse_declare!(Input<'a>(Token = char, Slice = &'a str, Many = &'a str));

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
fn value<'a, I: Input<'a>>(input: &mut I) -> Result<JsonValue<'a>, I> {
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

fn main() {
    let test = r#"
    {
        "Image": {
            "Width":  800,
            "Height": 600,
            "Title":  "View from 15th Floor",
            "Thumbnail": {
                "Url":    "http://www.example.com/image/481989943",
                "Height": 125,
                "Width":  100e10
            },
            "Animated" : false,
            "IDs": [116, 943, 234, 38793)
        },
        "escaped characters": "\u2192\uD83D\uDE00\"\t\uD834\uDD1E"
    }"#;

    let result = parse!(value: &mut pear::input::Text::from(test));
    match result {
        Ok(v) => println!("Value: {:#?}", v),
        Err(e) => println!("Error: {}", e)
    }
}

#[cfg(test)]
mod bench {
    extern crate test;

    use super::*;
    use self::test::Bencher;

    #[inline(always)]
    fn parse_json<'a, I: Input<'a>>(mut input: I) -> Result<JsonValue<'a>, I> {
        let result = parse!(value: &mut input);
        assert!(result.is_ok());
        result
    }

    // #[bench]
    // fn canada(b: &mut Bencher) {
    //     let data = include_str!("../assets/canada.json");
    //     b.iter(|| parse_json(data));
    // }

    // This is the benchmark from PEST. Unfortunately, our parser here is fully
    // fleshed out: it actually creates the `value`, while the PEST one just checks
    // if it parses. As a result, our parser will be much slower. You can immitate
    // the PEST parser's behavior by changing the parser so that it doesn't build
    // real values and instead returns dummy values.
    #[bench]
    fn json_data(b: &mut Bencher) {
        let data = r#"[
  {
    "_id": "5741cfe6bf9f447a509a269e",
    "index": 0,
    "guid": "642f0c2a-3d87-43ac-8f82-25f004e0c96a",
    "isActive": false,
    "balance": "$3,666.68",
    "picture": "http://placehold.it/32x32",
    "age": 39,
    "eyeColor": "blue",
    "name": "Leonor Herman",
    "gender": "female",
    "company": "RODEOMAD",
    "email": "leonorherman@rodeomad.com",
    "phone": "+1 (848) 456-2962",
    "address": "450 Seeley Street, Iberia, North Dakota, 7859",
    "about": "Reprehenderit in anim laboris labore sint occaecat labore proident ipsum exercitation. Ut ea aliqua duis occaecat consectetur aliqua anim id. Dolor ea fugiat excepteur reprehenderit eiusmod enim non sit nisi. Mollit consequat anim mollit et excepteur qui laborum qui eiusmod. Qui ea amet incididunt cillum quis occaecat excepteur qui duis nisi. Dolore labore eu sunt consequat magna.\r\n",
    "registered": "2015-03-06T02:49:06 -02:00",
    "latitude": -29.402032,
    "longitude": 151.088135,
    "tags": [
      "Lorem",
      "voluptate",
      "aute",
      "ullamco",
      "elit",
      "esse",
      "culpa"
    ],
    "friends": [
      {
        "id": 0,
        "name": "Millicent Norman"
      },
      {
        "id": 1,
        "name": "Vincent Cannon"
      },
      {
        "id": 2,
        "name": "Gray Berry"
      }
    ],
    "greeting": "Hello, Leonor Herman! You have 4 unread messages.",
    "favoriteFruit": "apple"
  },
  {
    "_id": "5741cfe69424f42d4493caa2",
    "index": 1,
    "guid": "40ec6b43-e6e6-44e1-92a8-dc80cd5d7179",
    "isActive": true,
    "balance": "$2,923.78",
    "picture": "http://placehold.it/32x32",
    "age": 36,
    "eyeColor": "blue",
    "name": "Barton Barnes",
    "gender": "male",
    "company": "BRAINQUIL",
    "email": "bartonbarnes@brainquil.com",
    "phone": "+1 (907) 553-3739",
    "address": "644 Falmouth Street, Sedley, Michigan, 5602",
    "about": "Et nulla laboris consectetur laborum labore. Officia dolor sint do amet excepteur dolore eiusmod. Occaecat pariatur sunt velit sunt ullamco labore commodo mollit sint dolore occaecat.\r\n",
    "registered": "2014-08-28T01:07:22 -03:00",
    "latitude": 14.056553,
    "longitude": -61.911624,
    "tags": [
      "laboris",
      "sunt",
      "esse",
      "tempor",
      "pariatur",
      "occaecat",
      "et"
    ],
    "friends": [
      {
        "id": 0,
        "name": "Tillman Mckay"
      },
      {
        "id": 1,
        "name": "Rivera Berg"
      },
      {
        "id": 2,
        "name": "Rosetta Erickson"
      }
    ],
    "greeting": "Hello, Barton Barnes! You have 2 unread messages.",
    "favoriteFruit": "banana"
  }
]"#;

        b.iter(|| parse_json(data));
    }
}
