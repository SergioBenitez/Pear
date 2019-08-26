#![feature(proc_macro_hygiene)]

use pear::input::{Text, Span};
use pear::result::Result;
use pear::{macros::*, parsers::*};

type FourMarkers = (usize, usize, usize, usize);

#[parser]
fn simple<'a>(input: &mut Text<'a>) -> Result<FourMarkers, Text<'a>> {
    let first = parse_marker!();
    eat('.')?;
    let second = parse_marker!();
    eat_slice("..")?;
    let third = parse_marker!();
    eat_slice("..")?;
    let fourth = parse_marker!();
    (first, second, third, fourth)
}

#[parser]
fn resetting<'a>(input: &mut Text<'a>) -> Result<FourMarkers, Text<'a>> {
    let first = parse_marker!();
    eat('.')?;
    parse_mark!();
    let second = parse_marker!();
    eat_slice("..")?;
    let third = parse_marker!();
    eat_slice("..")?;
    parse_mark!();
    let fourth = parse_marker!();
    (first, second, third, fourth)
}

#[test]
fn test_simple_marker() {
    let result = parse!(simple: &mut Text::from(".....")).unwrap();
    assert_eq!(result, (0, 0, 0, 0));
}

#[test]
fn test_resetting_marker() {
    let result = parse!(resetting: &mut Text::from(".....")).unwrap();
    assert_eq!(result, (0, 1, 1, 5));
}

type TwoSpans<'a> = (Span<'a>, Span<'a>);

#[parser]
fn context<'a>(input: &mut Text<'a>) -> Result<TwoSpans<'a>, Text<'a>> {
    eat_slice("...")?;
    let first = parse_context!();
    eat('\n')?;
    eat_slice("..")?;
    let second = parse_context!();
    (first.unwrap(), second.unwrap())
}

#[parser]
fn resetting_context<'a>(input: &mut Text<'a>) -> Result<TwoSpans<'a>, Text<'a>> {
    eat_slice("...")?;
    let first = parse_context!();
    eat('\n')?;
    parse_mark!();
    eat_slice("..")?;
    let second = parse_context!();
    (first.unwrap(), second.unwrap())
}

#[test]
fn test_context() {
    let (first, second) = parse!(context: &mut Text::from("...\n..")).unwrap();

    assert_eq!(first, Span {
        start: (1, 1, 0),
        end: (1, 4, 3),
        snippet: Some("..."),
    });

    assert_eq!(second, Span {
        start: (1, 1, 0),
        end: (2, 3, 6),
        snippet: Some("...\n.."),
    });
}

#[test]
fn test_resetting_context() {
    let (first, second) = parse!(resetting_context: &mut Text::from("...\n..")).unwrap();

    assert_eq!(first, Span {
        start: (1, 1, 0),
        end: (1, 4, 3),
        snippet: Some("..."),
    });

    assert_eq!(second, Span {
        start: (2, 1, 4),
        end: (2, 3, 6),
        snippet: Some(".."),
    });
}
