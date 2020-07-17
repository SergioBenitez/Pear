use pear::input::{Text, Span, Result};
use pear::{macros::*, parsers::*};

type FourMarkers = (usize, usize, usize, usize);

#[parser]
fn simple<'a>(input: &mut Text<'a>) -> Result<FourMarkers, Text<'a>> {
    let first = parse_last_marker!();
    eat('.')?;
    let second = parse_last_marker!();
    eat_slice("..")?;
    let third = parse_last_marker!();
    eat_slice("..")?;
    let fourth = parse_last_marker!();
    (first, second, third, fourth)
}

#[parser]
fn simple_updating<'a>(input: &mut Text<'a>) -> Result<FourMarkers, Text<'a>> {
    let first = parse_current_marker!();
    eat('.')?;
    let second = parse_current_marker!();
    eat_slice("..")?;
    let third = parse_current_marker!();
    eat_slice("..")?;
    let fourth = parse_current_marker!();
    (first, second, third, fourth)
}

#[parser]
fn resetting<'a>(input: &mut Text<'a>) -> Result<FourMarkers, Text<'a>> {
    let first = parse_last_marker!();
    eat('.')?;
    parse_mark!();
    let second = parse_last_marker!();
    eat_slice("..")?;
    let third = parse_last_marker!();
    eat_slice("..")?;
    parse_mark!();
    let fourth = parse_last_marker!();
    (first, second, third, fourth)
}

#[test]
fn test_simple_marker() {
    let result = parse!(simple: &mut Text::from(".....")).unwrap();
    assert_eq!(result, (0, 0, 0, 0));
}

#[test]
fn test_updating_marker() {
    let result = parse!(simple_updating: &mut Text::from(".....")).unwrap();
    assert_eq!(result, (0, 1, 3, 5));
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
        cursor: Some('\n'),
    });

    assert_eq!(second, Span {
        start: (1, 1, 0),
        end: (2, 3, 6),
        snippet: Some("...\n.."),
        cursor: None,
    });
}

#[test]
fn test_resetting_context() {
    let (first, second) = parse!(resetting_context: &mut Text::from("...\n..")).unwrap();

    assert_eq!(first, Span {
        start: (1, 1, 0),
        end: (1, 4, 3),
        snippet: Some("..."),
        cursor: Some('\n'),
    });

    assert_eq!(second, Span {
        start: (2, 1, 4),
        end: (2, 3, 6),
        snippet: Some(".."),
        cursor: None,
    });
}
