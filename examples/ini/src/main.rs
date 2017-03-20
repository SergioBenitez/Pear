#![feature(plugin)]
#![plugin(pear_codegen)]

#[macro_use] extern crate pear;
extern crate time;

use std::fmt;

use pear::ParseResult;
use pear::parsers::*;

#[derive(Debug, PartialEq, Eq)]
struct Property<'s> {
    name: &'s str,
    value: &'s str
}

#[derive(Debug, PartialEq, Eq)]
struct Section<'s> {
    name: Option<&'s str>,
    properties: Vec<Property<'s>>
}

#[derive(Debug, PartialEq, Eq)]
struct IniConfig<'s> {
    sections: Vec<Section<'s>>
}

impl<'s> fmt::Display for IniConfig<'s> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for section in self.sections.iter() {
            if let Some(name) = section.name {
                write!(f, "[[{}]]\n", name)?;
            }

            for property in section.properties.iter() {
                write!(f, "({})=({})\n", property.name, property.value)?;
            }
        }

        Ok(())
    }
}

#[inline]
fn is_whitespace(byte: char) -> bool {
    byte == ' ' || byte == '\t' || byte == '\n'
}

#[parser]
fn heading<'a>(input: &mut &'a str) -> ParseResult<&'a str, &'a str> {
    delimited('[', ']')
}

#[parser]
fn properties<'a>(input: &mut &'a str) -> ParseResult<&'a str, Vec<Property<'a>>> {
    let mut properties = Vec::new();
    repeat! {
        skip_while(is_whitespace);
        switch! {
            any!(peek(';'), peek('[')) => break,
            eof() => break,
            _ => {
                let name = take_some_while(|c| !"=\n;".contains(c));
                eat('=');
                let value = take_some_while(|c| !"\n;".contains(c));
                skip_while(is_whitespace);
                properties.push(Property { name: name.trim_right(), value: value.trim() });
            }
        }
    }

    properties
}

#[parser]
fn ini<'a>(ini_string: &mut &'a str) -> ParseResult<&'a str, IniConfig<'a>> {
    let mut sections = Vec::new();
    repeat! {
        skip_while(is_whitespace);
        let (name, ps) = switch! {
            peek('[') => (Some(heading()), properties()),
            eat(';') => {
                skip_while(|c| c != '\n');
                (None, Vec::new())
            },
            _ => (None, properties())
        };

        sections.push(Section { name: name, properties: ps })
    };

    IniConfig { sections: sections }
}

fn main() {
    // let ini_string = r#"
    //     name = "sergio"
    //     age = 100

    //     ; hello there!
    //     [section]
    //     a=b
    //     c=1

    //     [section1]
    //     a=1
    //     b=c

    //     [section2]
    //     a=1
    // "#;
    let ini_string = "\
[section]
a=b
c=1

[section]
a=b
c=1

[section1]
a=1 ; comment
b=c

[section2
a=1
";

    let start = time::precise_time_ns();
    let result = ini(&mut &*ini_string);
    let end = time::precise_time_ns();

    if let ParseResult::Error(ref e) = result {
        println!("Error: {}", e);
    }

    // TODO: Make sure we can use the same parser for files and strings.
    println!("Result (in {}us): {:?}", (end - start) / 1000, result);
}
