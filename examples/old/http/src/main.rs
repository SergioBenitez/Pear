#![feature(plugin)]
#![plugin(pear_codegen)]

#[macro_use] extern crate pear;

use pear::{ParseResult, Input};
use pear::parsers::*;

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

trait ByteLikeInput<'a>: Input<Token=u8, InSlice=&'a [u8], Slice=&'a [u8], Many=&'a [u8]> {  }
impl<'a, T: Input<Token=u8, InSlice=&'a [u8], Slice=&'a [u8], Many=&'a [u8]> + 'a> ByteLikeInput<'a> for T {  }

#[parser]
fn version<'a, I: ByteLikeInput<'a>>(input: &mut I) -> ParseResult<I, (u8, u8)> {
    eat_slice(b"HTTP/");
    let major = eat(b'1');
    eat(b'.');
    let minor = eat_if(|c| c == b'0' || c == b'1');

    (major - b'0', minor - b'0')
}

#[parser]
fn method<'a, I: ByteLikeInput<'a>>(input: &mut I) -> ParseResult<I, Method> {
    switch! {
        eat_slice(b"GET") => Method::Get,
        eat_slice(b"HEAD") => Method::Head,
        eat_slice(b"POST") => Method::Post,
        eat_slice(b"PUT") => Method::Put,
        eat_slice(b"DELETE") => Method::Delete,
        eat_slice(b"CONNECT") => Method::Connect,
        eat_slice(b"OPTIONS") => Method::Options,
        eat_slice(b"PATCH") => Method::Patch,
        eat_slice(b"TRACE") => Method::Trace
    }
}

// This is incredibly permissive.
#[parser]
fn target<'a, I: ByteLikeInput<'a>>(input: &mut I) -> ParseResult<I, &'a [u8]> {
    take_while(|c| c != b' ')
}

#[parser]
fn request_line<'a, I: ByteLikeInput<'a>>(input: &mut I) -> ParseResult<I, RequestLine<'a>> {
    let (method, _, uri, _, version) = (method(), eat(b' '), target(), eat(b' '), version());
    let uri_str = from!(::std::str::from_utf8(uri));
    RequestLine { method: method, uri: uri_str, version: version }
}

#[inline(always)]
fn is_line_end(byte: u8) -> bool {
    byte == b'\r' || byte == b'\n'
}

#[inline(always)]
fn is_whitespace(byte: u8) -> bool {
    byte == b' ' || byte == b'\t'
}

#[parser]
fn line_end<'a, I: ByteLikeInput<'a>>(input: &mut I) -> ParseResult<I, ()> {
    eat_slice(b"\r\n");
}

// This is very, very liberal.
#[parser]
fn header<'a, I: ByteLikeInput<'a>>(input: &mut I) -> ParseResult<I, Header<'a>> {
    let name = take_some_while(|c| c != b':' && !is_line_end(c));
    eat(b':');
    skip_while(is_whitespace);
    let value = take_some_while(|c| !is_line_end(c));
    line_end();

    let name_str = from!(::std::str::from_utf8(name));
    Header { name: name_str, value: value }
}

#[parser]
fn request<'a, I: ByteLikeInput<'a>>(input: &mut I) -> ParseResult<I, Request<'a>> {
    let request_line = request_line();
    line_end();

    let mut headers = vec![];
    try_repeat!(headers.push(header()));
    line_end();

    Request { request_line: request_line, headers: headers }
}

pub fn main() {
    let request_str = &[
        "GET http://localhost:8080 HTTP/1.1",
        "Content-Type: application/json",
        "Accept: application/json",
        "X-Real-IP: 12.12.12.12",
        "", "this is the body"
    ].join("\r\n");

    let mut bytes = request_str.as_bytes();
    let req = request(&mut bytes).unwrap();
    println!("Parsed: {:?}", req);
    println!("Remaining: {:?}", ::std::str::from_utf8(bytes).unwrap());
}

#[cfg(test)]
mod test {
    use pear::{ParseResult, StringFile, Input};
    use super::*;

    macro_rules! assert_parse_eq {
        ($name:ident, $($from:expr => $to:expr),+) => (
            $(
                match $name(&mut $from) {
                    ParseResult::Done(output) => assert_eq!(output, $to),
                    ParseResult::Error(e) => {
                        println!("{:?} failed to parse as '{}'!", $from, stringify!($name));
                        panic!("Error: {}", e);
                    }
                }
             )+
        );

        ($name:ident, $($from:expr => $to:expr),+,) => (assert_parse_eq!($name, $($from => $to),+))
    }

    macro_rules! assert_no_parse {
        ($name:ident, $($val:expr),+) => (
            $(
                if let ParseResult::Done(output) =  $name(&mut $val) {
                    panic!("{:?} unexpectedly parsed as '{}' {:?}!",
                           $val, stringify!($name), output);
                }
             )+
        );

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

    // #[test]
    // fn test_request() {
    //     assert_parse_eq!(header,
    //         b"Content-Type: application/json\r\n" as &[u8] => Header {
    //             name: "Content-Type",
    //             value: b"application/json"
    //         },
    //         b"Content-Type:application/json\r\n" as &[u8] => Header {
    //             name: "Content-Type",
    //             value: b"application/json"
    //         },
    //         b"Content-Type:  application/json\r\n" as &[u8] => Header {
    //             name: "Content-Type",
    //             value: b"application/json"
    //         },
    //         b"a:b\r\n" as &[u8] => Header {
    //             name: "a",
    //             value: b"b"
    //         },
    //     );

    //     assert_no_parse!(header,
    //         b"Content-Type application/json\r\n" as &[u8],
    //         b"Content-Type: application/json" as &[u8],
    //         b": application/json\r\n" as &[u8],
    //         b":\r\n" as &[u8],
    //     );
    // }
}

// #[derive(Debug)]
// struct MediaType<'s> {
//     top: &'s str,
//     sub: &'s str,
//     params: Vec<(&'s str, &'s str)>
// }

// #[inline]
// fn is_valid_byte(c: u8) -> bool {
//     match c as char {
//         '0'...'9' | 'a'...'z' | '^'...'~' | '#'...'\''
//             | '!' | '*' | '+' | '-' | '.'  => true,
//         _ => false
//     }
// }

// fn is_whitespace(byte: u8) -> bool {
//     byte == b' ' || byte == b'\t'
// }

// fn quoted_string(input: &str) -> ParseResult<&str, &str> {
//     parse!(input, {
//         eat(b'"');
//         let inner = take_some_while(|c| c != b'\"');
//         eat(b'"');

//         inner
//     })
// }

// fn media_type(input: &str) -> ParseResult<&str, MediaType> {
//     parse!(input, {
//         let top = take_some_while(|c| is_valid_byte(c) && c != b'/');
//         eat(b'/');
//         let sub = take_some_while(is_valid_byte);

//         // // OWS* ; OWS*
//         let mut params = Vec::new();
//         let _ = try_repeat! {
//             skip_while(is_whitespace);
//             eat(b';');
//             skip_while(is_whitespace);

//             let key = take_some_while(|c| is_valid_byte(c) && c != b'=');
//             eat(b'=');

//             let value = switch! {
//                 peek(b'"') => quoted_string(),
//                 _ => take_some_while(|c| is_valid_byte(c) && c != b';')
//             };

//             params.push((key, value))
//         };

//         MediaType { top: top, sub: sub, params: params }
//     })
// }

// fn accept(input: &str) -> ParseResult<&str, Vec<MediaType>> {
//     parse!(input, {
//         let mut media_types = Vec::new();
//         let _ = repeat! {
//             let media_type = media_type();
//             switch! {
//                 eat(b',') => skip_while(is_whitespace),
//                 _ => ()
//             };

//             media_types.push(media_type)
//         };

//         media_types
//     })
// }


// fn main() {
//     println!("ABC: {:?}", abc("abc"));
//     println!("ABC: {:?}", abc("bc"));
//     println!("ABC: {:?}", abc("c"));

//     println!("hi: {:?}", hi("hihihihihi"));

//     println!("MEDIA TYPE: {:?}", media_type("a/b; a=b; c=d"));
//     println!("ACCEPT: {:?}", accept("a/b; a=b, c/d"));
// }
