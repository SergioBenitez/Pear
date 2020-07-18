#![warn(rust_2018_idioms)]

use std::fmt;

use pear::parsers::*;
use pear::input::{Pear, Result};
use pear::macros::{parser, parse, switch, parse_declare, parse_error};

#[derive(Debug, PartialEq)]
enum Value<'s> {
    Boolean(bool),
    String(&'s str),
    Number(f64)
}

#[derive(Debug, PartialEq)]
struct Property<'s> {
    name: &'s str,
    value: Value<'s>
}

#[derive(Debug, PartialEq)]
struct Section<'s> {
    name: Option<&'s str>,
    properties: Vec<Property<'s>>
}

#[derive(Debug, PartialEq)]
struct IniConfig<'s> {
    sections: Vec<Section<'s>>
}

impl fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Display for IniConfig<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for section in self.sections.iter() {
            if let Some(name) = section.name {
                write!(f, "[({})]\n", name)?;
            }

            for property in section.properties.iter() {
                write!(f, "({})=({})\n", property.name, property.value)?;
            }
        }

        Ok(())
    }
}

#[inline]
fn is_whitespace(&byte: &char) -> bool {
    byte == ' ' || byte == '\t' || byte == '\n'
}

#[inline]
fn is_num_char(&byte: &char) -> bool {
    match byte { '0'..='9' | '.' => true, _ => false }
}

parse_declare!(Input<'a>(Token = char, Slice = &'a str, Many = &'a str));

#[parser]
fn comment<'a, I: Input<'a>>(input: &mut Pear<I>) -> Result<(), I> {
    (eat(';')?, skip_while(|c| *c != '\n')?);
}

#[parser]
fn float<'a, I: Input<'a>>(input: &mut Pear<I>) -> Result<f64, I> {
    take_some_while(is_num_char)?.parse().or_else(|e| parse_error!("{}", e)?)
}

#[parser]
fn value<'a, I: Input<'a>>(input: &mut Pear<I>) -> Result<Value<'a>, I> {
    switch! {
        eat_slice("true") | eat_slice("yes") => Value::Boolean(true),
        eat_slice("false") | eat_slice("no") => Value::Boolean(false),
        peek_if(is_num_char) => Value::Number(float()?),
        _ => Value::String(take_some_while(|&c| !"\n;".contains(c))?.trim()),
    }
}

#[parser]
fn heading<'a, I: Input<'a>>(input: &mut Pear<I>) -> Result<&'a str, I> {
    delimited_some('[', |c| !is_whitespace(c), ']')?
}

#[parser]
fn name<'a, I: Input<'a>>(input: &mut Pear<I>) -> Result<&'a str, I> {
    take_some_while(|&c| !"=\n;".contains(c))?.trim_end()
}

#[parser]
fn properties<'a, I: Input<'a>>(input: &mut Pear<I>) -> Result<Vec<Property<'a>>, I> {
    let mut properties = Vec::new();
    loop {
        skip_while(is_whitespace)?;
        switch! {
            peek(';') | peek('[') | eof() => break,
            _ => {
                let (name, _, value) = (name()?, eat('=')?, value()?);
                skip_while(is_whitespace)?;
                properties.push(Property { name, value });
            }
        }
    }

    properties
}

#[parser]
fn ini<'a, I: Input<'a>>(input: &mut Pear<I>) -> Result<IniConfig<'a>, I> {
    let mut sections = Vec::new();
    loop {
        skip_while(is_whitespace)?;
        let (name, properties) = switch! {
            eof() => break,
            comment() => continue,
            peek('[') => (Some(heading()?), properties()?),
            _ => (None, properties()?),
        };

        sections.push(Section { name, properties })
    }

    IniConfig { sections }
}

const INI_STRING: &str = "\
; a section
a=b
; c is very special
; and don't you know it
c=2.0

[section]
a=3
c=1

[section1]
a=2 ; comment
b=c

[section2]
a=1

[section10
";

fn main() {
    // let start = time::precise_time_ns();
    // let result = parse!(ini: &mut Pear<I>NI_STRING);
    let result = parse!(ini: pear::input::Text::from(INI_STRING));
    // let result = parse!(ini: &mut Pear<I>NI_STRING);
    // let end = time::precise_time_ns();

    match result {
        Err(ref e) => println!("Error: {}", e),
        Ok(v) => println!("Got: {}", v)
    }

    // TODO: Make sure we can use the same parser for files and strings.
    // println!("Result (in {}us): {:?}", (end - start) / 1000, result);
}
