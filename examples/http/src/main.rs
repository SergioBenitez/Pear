extern crate pear;

use pear::{parsers::*, combinators::*};
use pear::macros::{parser, switch, parse_error};

#[derive(Debug, PartialEq)]
enum Method {
    Get, Head, Post, Put, Delete, Connect, Options, Trace, Patch
}

#[derive(Debug, PartialEq)]
struct RequestLine<'a> {
    method: Method,
    uri: &'a str,
    version: (u8, u8)
}

#[derive(Debug, PartialEq)]
struct Header<'a> {
    name: &'a str,
    value: &'a [u8],
}

#[derive(Debug, PartialEq)]
struct Request<'a> {
    request_line: RequestLine<'a>,
    headers: Vec<Header<'a>>
}

type Input<'a> = pear::input::Pear<pear::input::Cursor<&'a [u8]>>;

type Result<'a, T> = pear::input::Result<T, Input<'a>>;

#[parser]
fn version<'a>(input: &mut Input<'a>) -> Result<'a, (u8, u8)> {
    eat_slice(b"HTTP/1.")?;
    let minor = eat_if(|&c| c == b'0' || c == b'1')?;
    (1, minor - b'0')
}

#[parser]
fn method<'a>(input: &mut Input<'a>) -> Result<'a, Method> {
    switch! {
        eat_slice(b"GET") => Method::Get,
        eat_slice(b"HEAD") => Method::Head,
        eat_slice(b"POST") => Method::Post,
        eat_slice(b"PUT") => Method::Put,
        eat_slice(b"DELETE") => Method::Delete,
        eat_slice(b"CONNECT") => Method::Connect,
        eat_slice(b"OPTIONS") => Method::Options,
        eat_slice(b"PATCH") => Method::Patch,
        eat_slice(b"TRACE") => Method::Trace,
        _ => {
            let rogue = take_while(|&c| c != b' ')?;
            parse_error!("unknown method: {:?}", rogue)?
        }
    }
}

// This is incredibly permissive.
#[parser]
fn string<'a>(input: &mut Input<'a>) -> Result<'a, &'a str> {
    string_until(None)?
}

// This is incredibly permissive.
#[parser]
fn string_until<'a>(input: &mut Input<'a>, c: Option<u8>) -> Result<'a, &'a str> {
    let value = match c {
        Some(c) => take_some_while_until(is_ascii_line_byte, c)?,
        None => take_some_while(is_ascii_line_byte)?,
    };

    unsafe { std::str::from_utf8_unchecked(&value) }
}

#[parser]
fn request_line<'a>(input: &mut Input<'a>) -> Result<'a, RequestLine<'a>> {
    RequestLine {
        method: method()?,
        uri: (eat(b' ')?, string()?).1,
        version: (eat(b' ')?, version()?).1
    }
}

#[inline(always)]
fn is_ascii_line_byte(&byte: &u8) -> bool {
    byte.is_ascii() && byte != b'\r' && byte != b'\n' && !is_whitespace(&byte)
}

#[inline(always)]
fn is_line_byte(&byte: &u8) -> bool {
    byte != b'\r' && byte != b'\n'
}

#[inline(always)]
fn is_whitespace(&byte: &u8) -> bool {
    byte == b' ' || byte == b'\t'
}

#[parser]
fn line_end<'a>(input: &mut Input<'a>) -> Result<'a, ()> {
    eat_slice(b"\r\n")?;
}

// This is very, very liberal.
#[parser]
fn header<'a>(input: &mut Input<'a>) -> Result<'a, Header<'a>> {
    let name = string_until(Some(b':'))?;
    (eat(b':')?, skip_while(is_whitespace)?);
    let value = take_some_while(is_line_byte)?.values;
    line_end()?;

    Header { name, value }
}

#[parser]
fn request<'a>(input: &mut Input<'a>) -> Result<'a, Request<'a>> {
    Request {
        request_line: (request_line()?, line_end()?).0,
        headers: (try_collect(header)?, line_end()?).0
    }
}

pub fn main() {
    let request_str = &[
        "GET http://localhost:8080 HTTP/1.1",
        "Content-Type: application/json",
        "Accept: application/json",
        "X-Real-IP: 12.12.12.12",
        "", "this is the body"
    ].join("\r\n");

    let mut cursor = Input::new(request_str.as_bytes());
    let result = request(&mut cursor);
    match result {
        Ok(request) => println!("Parsed: {:?}", request),
        Err(e) => eprint!("Error: {}", e)
    }

    println!("Cursor: {:?}", std::str::from_utf8(cursor.items));
}

#[cfg(test)]
mod test {
    use super::*;
    use pear::macros::parse;

    macro_rules! assert_parse_eq {
        ($name:ident, $($from:expr => $to:expr),+) => (
            $(
                match parse!($name: Input::new($from)) {
                    Ok(output) => assert_eq!(output, $to),
                    Err(e) => {
                        println!("{:?} failed to parse as '{}'!", $from, stringify!($name));
                        panic!("Error: {}", e);
                    }
                }
             )+
        );

        ($name:ident, $($from:expr => $to:expr),+,) => (assert_parse_eq!($name, $($from => $to),+))
    }

    macro_rules! assert_no_parse {
        ($name:ident, $($val:expr),+) => ($(
            if let Ok(v) = parse!($name: Input::new($val)) {
                panic!("{:?} unexpectedly parsed as '{}' {:?}!", $val, stringify!($name), v);
            }
        )+);

        ($name:ident, $($val:expr),+,) => (assert_no_parse!($name, $($val),+))
    }

    #[test]
    fn test_http_version() {
        assert_parse_eq!(version,
            b"HTTP/1.1" as &[u8] => (1, 1),
            b"HTTP/1.0" as &[u8] => (1, 0)
        );

        assert_no_parse!(version,
            b"HTTP/2.1" as &[u8],
            b"HTTP/1." as &[u8],
            b"HTTP/1" as &[u8],
            b"http/1.1" as &[u8],
            b"HTTP1.1" as &[u8],
            b".1" as &[u8],
            b"" as &[u8],
        );
    }

    #[test]
    fn test_method() {
        assert_parse_eq!(method,
            b"GET" as &[u8] => Method::Get,
            b"PUT" as &[u8] => Method::Put,
            b"POST" as &[u8] => Method::Post,
            b"DELETE" as &[u8] => Method::Delete,
            b"HEAD" as &[u8] => Method::Head,
            b"OPTIONS" as &[u8] => Method::Options,
            b"TRACE" as &[u8] => Method::Trace,
            b"CONNECT" as &[u8] => Method::Connect,
            b"PATCH" as &[u8] => Method::Patch,
        );

        assert_no_parse!(method,
            b"get" as &[u8],
            b"GeT" as &[u8],
            b"" as &[u8],
            b"GERT" as &[u8],
        );
    }

    #[test]
    fn test_header() {
        assert_parse_eq!(header,
            b"Content-Type: application/json\r\n" as &[u8] => Header {
                name: "Content-Type",
                value: b"application/json"
            },
            b"Content-Type:application/json\r\n" as &[u8] => Header {
                name: "Content-Type",
                value: b"application/json"
            },
            b"Content-Type:  application/json\r\n" as &[u8] => Header {
                name: "Content-Type",
                value: b"application/json"
            },
            b"a:b\r\n" as &[u8] => Header {
                name: "a",
                value: b"b"
            },
        );

        assert_no_parse!(header,
            b"Content-Type application/json\r\n" as &[u8],
            b"Content-Type: application/json" as &[u8],
            b": application/json\r\n" as &[u8],
            b":\r\n" as &[u8],
        );
    }

    #[test]
    fn test_request() {
        assert_parse_eq!(header,
            b"Content-Type: application/json\r\n" as &[u8] => Header {
                name: "Content-Type",
                value: b"application/json"
            },
            b"Content-Type:application/json\r\n" as &[u8] => Header {
                name: "Content-Type",
                value: b"application/json"
            },
            b"Content-Type:  application/json\r\n" as &[u8] => Header {
                name: "Content-Type",
                value: b"application/json"
            },
            b"a:b\r\n" as &[u8] => Header {
                name: "a",
                value: b"b"
            },
        );

        assert_no_parse!(header,
            b"Content-Type application/json\r\n" as &[u8],
            b"Content-Type: application/json" as &[u8],
            b": application/json\r\n" as &[u8],
            b":\r\n" as &[u8],
        );
    }
}
