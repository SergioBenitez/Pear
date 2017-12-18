#![feature(plugin)]
#![plugin(pear_codegen)]

#[macro_use] extern crate pear;

mod tables;
mod utils;
mod indexed;

use std::borrow::Cow;
use std::str::{from_utf8, from_utf8_unchecked};
use std::fmt::{self, Display};

use pear::{ParseResult, ParseError, Input, Length};
use pear::parsers::*;

use utils::merge;
use indexed::{Indexed, IndexedInput};
use self::tables::{is_reg_name_char, is_pchar};

/*
 *
 * request-target = origin-form / absolute-form / authority-form / asterisk-form
 *
 * -------------------------------------------------------------------------------
 *
 * asterisk-form = "*"
 *
 * -------------------------------------------------------------------------------
 *
 * origin-form = absolute-path [ "?" query ]
 *
 * absolute-path = 1*( "/" segment )
 *
 * -------------------------------------------------------------------------------
 *
 * authority-form = authority
 *
 * -------------------------------------------------------------------------------
 *
 * 1. look for ':', '@', '?'
 * 2. if neither is found, you have an authority, text is `host`
 * 3. if ':' is found, have either 'host', 'scheme', or 'userinfo'
 *  * can only be host if: next four characters are port
 *  * must be host if: text before ':' is empty, requires port
 *  * if next (at most) four characters are numbers, then we have a host/port.
 *  * if next character is '/' or there is none, then scheme
 *  * otherwise try as scheme, fallback to userinfo if find '@'
 * 4. if '?' is found, have either 'host', 'scheme', or 'userinfo'
 * 5. if '@' is found, have 'userinfo'
 *
 * Alternatively, don't support path-rootless or path-empty, then it's not
 * ambigous: look for ':', '@', or '?':
 *  * if none is found or found ':'  but text before ':' is empty: authority
 *  * if '@', must have authority,
 *  * if '?', absolute
 *  * if ':' followed by '/', must have absolute
 *  * if ':' _not_ followed by '/', must have authority
 *
 * -------------------------------------------------------------------------------
 *
 * absolute-form = absolute-URI
 *
 * absolute-URI  = scheme ":" hier-part [ "?" query ]
 *
 * scheme        = ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )
 *
 * hier-part     = "//" authority path-abempty
 *              / path-absolute
 *              / path-rootless
 *              / path-empty
 *
 * query         = *( pchar / "/" / "?" )
 *
 * authority     = [ userinfo "@" ] host [ ":" port ]
 * userinfo      = *( unreserved / pct-encoded / sub-delims / ":" )
 * host          = IP-literal / IPv4address / reg-name
 * port          = *DIGIT
 *
 * reg-name      = *( unreserved / pct-encoded / sub-delims )
 *
 * path-abempty  = *( "/" segment )
 *
 * path-absolute = "/" [ segment-nz *( "/" segment ) ]
 * path-noscheme = segment-nz-nc *( "/" segment )
 * path-rootless = segment-nz *( "/" segment )
 * path-empty    = 0<pchar>
 *
 * segment       = *pchar
 * segment-nz    = 1*pchar
 *
 * pchar         = unreserved / pct-encoded / sub-delims / ":" / "@"
 *
 * unreserved    = ALPHA / DIGIT / "-" / "." / "_" / "~"
 * pct-encoded   = "%" HEXDIG HEXDIG
 * sub-delims    = "!" / "$" / "&" / "'" / "(" / ")"
 *              / "*" / "+" / "," / ";" / "="
 *
 * IP-literal    = "[" ( IPv6address / IPvFuture  ) "]"
 *
 * IPvFuture     = "v" 1*HEXDIG "." 1*( unreserved / sub-delims / ":" )
 *
 * IPv6address   =                            6( h16 ":" ) ls32
 *              /                       "::" 5( h16 ":" ) ls32
 *              / [               h16 ] "::" 4( h16 ":" ) ls32
 *              / [ *1( h16 ":" ) h16 ] "::" 3( h16 ":" ) ls32
 *              / [ *2( h16 ":" ) h16 ] "::" 2( h16 ":" ) ls32
 *              / [ *3( h16 ":" ) h16 ] "::"    h16 ":"   ls32
 *              / [ *4( h16 ":" ) h16 ] "::"              ls32
 *              / [ *5( h16 ":" ) h16 ] "::"              h16
 *              / [ *6( h16 ":" ) h16 ] "::"
 *
 * IPv4address   = dec-octet "." dec-octet "." dec-octet "." dec-octet
 *
 * dec-octet     = DIGIT                 ; 0-9
 *              / %x31-39 DIGIT         ; 10-99
 *              / "1" 2DIGIT            ; 100-199
 *              / "2" %x30-34 DIGIT     ; 200-249
 *              / "25" %x30-35          ; 250-255
 *
 * ALPHA          =  %x41-5A / %x61-7A   ; A-Z / a-z
 * HEXDIG         =  DIGIT / "A" / "B" / "C" / "D" / "E" / "F"
 * DIGIT          =  %x30-39 ; 0-9
 *
 * -------------------------------------------------------------------------------
**/

#[derive(Debug, PartialEq)]
pub enum Error<I: Input> {
    Empty,
    Parse(ParseError<I>)
}

impl<I: Input> From<ParseError<I>> for Error<I> {
    #[inline(always)]
    fn from(error: ParseError<I>) -> Self {
        Error::Parse(error)
    }
}

type ByteInput<'a> = IndexedInput<'a, [u8]>;
type IndexedStr<'a> = Indexed<'a, str>;

#[derive(Debug, PartialEq)]
pub enum Uri<'a> {
    Origin(Origin<'a>),
    Authority(Authority<'a>),
    Absolute(Absolute<'a>),
    Asterisk,
}

impl<'a> Display for Uri<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Uri::Origin(ref origin) => write!(f, "{}", origin),
            Uri::Authority(ref authority) => write!(f, "{}", authority),
            Uri::Absolute(ref absolute) => write!(f, "{}", absolute),
            Uri::Asterisk => write!(f, "*")
        }
    }
}

#[derive(Debug)]
pub struct Origin<'a> {
    source: Option<Cow<'a, str>>,
    path: IndexedStr<'a>,
    query: Option<IndexedStr<'a>>,
}

impl<'a, 'b> PartialEq<Origin<'b>> for Origin<'a> {
    fn eq(&self, other: &Origin<'b>) -> bool {
        self.path() == other.path() && self.query() == other.query()
    }
}

impl<'a> Origin<'a> {
    fn new<P: Into<Cow<'a, str>>>(path: P) -> Origin<'a> {
        Origin {
            source: None,
            path: Indexed::from(path),
            query: None
        }
    }

    fn new_with_query<P, Q>(path: P, query: Q) -> Origin<'a>
        where P: Into<Cow<'a, str>>, Q: Into<Cow<'a, str>>
    {
        Origin {
            source: None,
            path: Indexed::from(path),
            query: Some(Indexed::from(query))
        }
    }

    #[inline]
    fn path(&self) -> &str {
        self.path.to_source(&self.source)
    }

    #[inline]
    fn query(&self) -> Option<&str> {
        self.query.as_ref().map(|q| q.to_source(&self.source))
    }
}

impl<'a> Display for Origin<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.path())?;
        if let Some(q) = self.query() {
            write!(f, "?{}", q)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Authority<'a> {
    source: Option<Cow<'a, str>>,
    userinfo: Option<IndexedStr<'a>>,
    host: IndexedStr<'a>,
    port: Option<u16>,
}

impl<'a> Authority<'a> {
    fn new(userinfo: Option<&'a str>, host: &'a str, port: Option<u16>) -> Authority<'a> {
        Authority {
            source: None,
            userinfo: userinfo.map(|u| u.into()),
            host: host.into(),
            port: port
        }
    }

    fn userinfo(&self) -> Option<&str> {
        self.userinfo.as_ref().map(|u| u.to_source(&self.source))
    }

    #[inline(always)]
    fn host(&self) -> &str {
        self.host.to_source(&self.source)
    }

    #[inline(always)]
    fn port(&self) -> Option<u16> {
        self.port
    }
}

impl<'a, 'b> PartialEq<Authority<'b>> for Authority<'a> {
    fn eq(&self, other: &Authority<'b>) -> bool {
        self.userinfo() == other.userinfo()
            && self.host() == other.host()
            && self.port() == other.port()
    }
}

impl<'a> Display for Authority<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(userinfo) = self.userinfo() {
            write!(f, "{}@", userinfo)?;
        }

        write!(f, "{}", self.host())?;
        if let Some(port) = self.port {
            write!(f, ":{}", port)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Absolute<'a> {
    source: Option<Cow<'a, str>>,
    scheme: IndexedStr<'a>,
    authority: Option<Authority<'a>>,
    origin: Option<Origin<'a>>,
}

impl<'a> Absolute<'a> {
    fn new(
        scheme: &'a str,
        authority: Option<Authority<'a>>,
        origin: Option<Origin<'a>>
    ) -> Absolute<'a> {
        Absolute {
            source: None, scheme: scheme.into(), authority, origin
        }
    }

    #[inline(always)]
    fn scheme(&self) -> &str {
        self.scheme.to_source(&self.source)
    }

    #[inline(always)]
    fn authority(&self) -> Option<&Authority<'a>> {
        self.authority.as_ref()
    }

    #[inline(always)]
    fn origin(&self) -> Option<&Origin<'a>> {
        self.origin.as_ref()
    }
}

impl<'a, 'b> PartialEq<Absolute<'b>> for Absolute<'a> {
    fn eq(&self, other: &Absolute<'b>) -> bool {
        self.scheme() == other.scheme()
            && self.authority() == other.authority()
            && self.origin() == other.origin()
    }
}

impl<'a> Display for Absolute<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.scheme())?;
        match self.authority {
            Some(ref authority) => write!(f, "://{}", authority)?,
            None => write!(f, ":")?
        }

        if let Some(ref origin) = self.origin {
            write!(f, "{}", origin)?;
        }

        Ok(())
    }
}

fn main() {
    println!("{}", Origin::new_with_query("/a/b/c", "hey"));
    println!("{}", Origin::new("hi"));

    let y = "hey".to_string();
}

// impl<'a> Uri<'a> {
//     fn origin(path: &'a [u8], query: Option<&'a [u8]>) -> Uri<'a> {
//         Uri::Origin(Origin { path , query })
//     }

//     fn host_authority(host: &'a [u8]) -> Uri<'a> {
//         Uri::Authority(Authority { host: host, userinfo: None, port: None })
//     }

//     #[cfg(test)]
//     fn authority(
//         userinfo: Option<&'a [u8]>,
//         host: &'a [u8],
//         port: Option<u16>
//     ) -> Uri<'a> {
//         Uri::Authority(Authority { host, userinfo, port })
//     }

//     fn absolute_path(
//         scheme: &'a [u8],
//         path: &'a [u8],
//         query: Option<&'a [u8]>
//     ) -> Uri<'a> {
//         Uri::Absolute(Absolute {
//             scheme: scheme,
//             authority: None,
//             origin: Some(Origin { path, query })
//         })
//     }
// }

// type UriParseResult<'a, I> = ParseResult<I, Result<Uri<'a>, Error<I>>>;

// #[parser]
// fn uri<'a>(input: &mut &'a [u8]) -> UriParseResult<'a, &'a [u8]> {
//     match input.len() {
//         0 => Err(Error::Empty),
//         1 => switch! {
//             eat(b'*') => Ok(Uri::Asterisk),
//             eat(b'/') => Ok(Uri::origin(b"/", None)),
//             _ => Ok(Uri::host_authority(take_n_while(1, is_reg_name_char)))
//         },
//         _ => switch! {
//             peek(b'/') => Ok(Uri::Origin(origin())),
//             _ => Ok(absolute_or_authority())
//         }
//     }
// }

// #[parser]
// fn origin<'a, I: SBI<'a>>(input: &mut I) -> ParseResult<I, Origin<'a>> {
//     (peek(b'/'), path_and_query()).1
// }

// #[parser]
// fn path_and_query<'a, I: SBI<'a>>(input: &mut I) -> ParseResult<I, Origin<'a>> {
//     let path = take_while(is_pchar);
//     let query = switch! {
//         eat(b'?') => Some(take_while(is_pchar)),
//         _ => None
//     };

//     if path.is_empty() && query.is_none() {
//         parse_error!("path_and_query", "expected path or query");
//     } else {
//         Origin { path, query }
//     }
// }

// #[parser]
// fn port<'a, I: SBI<'a>>(input: &mut I) -> ParseResult<I, u16> {
//     let port_str = take_n_while(5, |c| c >= b'0' && c <= b'9');

//     let mut port_num: u16 = 0;
//     for (b, i) in port_str.iter().rev().zip(&[1, 10, 100, 1000, 10000]) {
//         port_num += (*b - b'0') as u16 * i;
//     }

//     port_num
// }

// #[parser]
// fn authority<'a, I: SBI<'a>>(
//     i: &mut I,
//     userinfo: Option<&'a [u8]>
// ) -> ParseResult<I, Authority<'a>> {
//     let host = switch! {
//         peek(b'[') => delimited(b'[', is_pchar, b']'),
//         _ => take_while(is_reg_name_char)
//     };

//     let port = switch! {
//         eat(b':') => Some(port()),
//         _ => None
//     };

//     Authority { userinfo, host, port }
// }

// #[parser]
// fn absolute<'a>(
//     input: &mut &'a [u8],
//     scheme: &'a [u8]
// ) -> ParseResult<&'a [u8], Absolute<'a>> {
//     switch! {
//         eat_slice(b"://") => {
//             let left = take_while(|c| is_reg_name_char(c) || c == b':');
//             let authority = switch! {
//                 eat(b'@') => authority(Some(left)),
//                 _ => {
//                     *input = whitelist!(merge(left, *input));
//                     authority(None)
//                 }
//             };

//             Absolute { scheme, authority: Some(authority), origin: maybe!(path_and_query()) }
//         },
//         eat(b':') => Absolute { scheme, authority: None, origin: Some(path_and_query()) }
//     }
// }

// // foo:a/b?hi

// #[parser]
// fn absolute_or_authority<'a>(input: &mut &'a [u8]) -> ParseResult<&'a [u8], Uri<'a>> {
//     let left = take_while(|c| is_reg_name_char(c));
//     switch! {
//         peek_slice(b":/") => Uri::Absolute(absolute(left)),
//         eat(b'@') => Uri::Authority(authority(Some(left))),
//         peek(b':') => {
//             let rest = take_while(|c| is_reg_name_char(c) || c == b':');
//             switch! {
//                 eat(b'@') => Uri::Authority(authority(Some(whitelist!(merge(left, rest))))),
//                 eat(b'?') => Uri::absolute_path(left, &rest[1..], Some(take_while(is_pchar))),
//                 peek(b'/') => {
//                     *input = whitelist!(merge(rest, *input));
//                     Uri::Absolute(absolute(left))
//                 },
//                 _ => Uri::absolute_path(left, &rest[1..], None)
//             }
//         },
//         _ => Uri::Authority(authority(None))
//     }
// }

// pub fn parse_uri<'a>(input: &'a str) -> Result<Uri<'a>, Error<&'a [u8]>> {
//     let input = &mut input.as_bytes();
//     match parse!(input, (uri(), eof()).0) {
//         ParseResult::Error(e) => Err(Error::Parse(e)),
//         ParseResult::Done(result) => result
//     }
// }

// use std::borrow::Cow;

// #[derive(Debug)]
// struct Ab {
//     source: Cow<'static, [u8]>,
//     a: Indexed<'static, [u8]>,
//     b: Indexed<'static, [u8]>,
// }

// impl fmt::Display for Ab {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         let a = self.a.to_source(Some(&self.source));
//         let b = self.b.to_source(Some(&self.source));
//         write!(f, "a's: {:?}, b's: {:?}",
//                std::str::from_utf8(a),
//                std::str::from_utf8(b))
//     }
// }

// #[parser]
// fn simple<'a>(input: &mut ByteInput<'a>) -> ParseResult<ByteInput<'a>, Ab> {
//     let a = take_while(|c| c != b'a');
//     let b = take_while(|c| c != b'b');
//     eat(b'b');

//     let x = input.source().to_owned().into();

//     Ab {
//         // FIXME: This doesn't work?
//         // source: input.source().to_owned().into(),
//         source: x,
//         a: a,
//         b: b,
//     }
// }

// fn parse_simple(input: &[u8]) -> Ab {
//     // parse!(&mut ByteInput::from(input), simple()).unwrap()
//     let input = &mut ByteInput::from(input);
//     parse!(input, (simple(), eof()).0).unwrap()
//     // FIXME: This should also work.
//     // parse!(&mut ByteInput::from(input), (simple(), eof()).0).unwrap()
// }

// pub fn main() {
//     // println!("{}", Origin { path: b"hi", query: Some(b"hey") });
//     // println!("{}", Origin { path: b"hi", query: None });
//     // println!("{}", Authority { userinfo: Some(b"user:pass"), host: b"sergio.bz", port: Some(10) });
//     // println!("{}", Absolute {
//     //     scheme: b"abc",
//     //     authority: Some(Authority {
//     //         userinfo: Some(b"u:p"),
//     //         host: b"foo.com",
//     //         port: Some(123)
//     //     }),
//     //     origin: Some(Origin {
//     //         path: b"/a/b",
//     //         query: Some(b"key=value&key2=value2")
//     //     }),
//     // });
//     let ab = parse_simple(b"dlfsjhklsdfakjfkljdfkb");
//     println!("{}", ab);
// }

// #[cfg(test)]
// mod test {
//     use pear::ParseResult;
//     use super::*;

//     macro_rules! assert_parse_eq {
//         ($($from:expr => $to:expr),+) => (
//             $(
//                 match parse_uri($from) {
//                     Ok(output) => {
//                         if output != $to {
//                             println!("Failure on: {:?}", $from);
//                             assert_eq!(output, $to);
//                         }
//                     }
//                     Err(e) => {
//                         println!("{:?} failed to parse!", $from);
//                         panic!("Error: {:?}", e);
//                     }
//                 }
//              )+
//         );

//         ($($from:expr => $to:expr),+,) => (assert_parse_eq!($($from => $to),+))
//     }

//     macro_rules! assert_no_parse {
//         ($($from:expr),+) => (
//             $(
//                 if let Ok(uri) = parse_uri($from) {
//                     println!("{:?} parsed unexpectedly!", $from);
//                     panic!("Parsed as: {:?}", uri);
//                 }
//              )+
//         );

//         ($($from:expr),+,) => (assert_no_parse!($($from),+))
//     }

//     #[test]
//     fn single_byte() {
//         assert_parse_eq!(
//             "*" => Uri::Asterisk,
//             "/" => Uri::origin(b"/", None),
//             "." => Uri::host_authority(b"."),
//             "_" => Uri::host_authority(b"_"),
//         );

//         assert_no_parse!("?", "#");
//     }

//     #[test]
//     fn origin() {
//         assert_parse_eq!(
//             "/a/b/c" => Uri::origin(b"/a/b/c", None),
//             "/a/b/c?" => Uri::origin(b"/a/b/c", Some(b"")),
//             "/a/b/c?abc" => Uri::origin(b"/a/b/c", Some(b"abc")),
//             "/?abc" => Uri::origin(b"/", Some(b"abc")),
//             "/hi%20there?a=b&c=d" => Uri::origin(b"/hi%20there", Some(b"a=b&c=d")),
//             "/c/d/fa/b/c?abc" => Uri::origin(b"/c/d/fa/b/c", Some(b"abc")),
//         );
//     }

//     #[test]
//     fn authority() {
//         assert_parse_eq!(
//             "sergio:benitez@spark" => Uri::authority(Some(b"sergio:benitez"), b"spark", None),
//             "a:b:c@1.2.3:12121" => Uri::authority(Some(b"a:b:c"), b"1.2.3", Some(12121)),
//             "sergio@spark" => Uri::authority(Some(b"sergio"), b"spark", None),
//             "sergio@spark:230" => Uri::authority(Some(b"sergio"), b"spark", Some(230)),
//             "sergio@[1::]:230" => Uri::authority(Some(b"sergio"), b"1::", Some(230)),
//         );
//     }

//     #[test]
//     fn absolute() {
//         assert_parse_eq!(
//             "http://foo.com:8000" => Uri::Absolute(Absolute {
//                 scheme: b"http",
//                 origin: None,
//                 authority: Some(Authority {
//                     userinfo: None,
//                     host: b"foo.com",
//                     port: Some(8000)
//                 })
//             }),
//             "http://foo:8000" => Uri::Absolute(Absolute {
//                 scheme: b"http",
//                 origin: None,
//                 authority: Some(Authority {
//                     userinfo: None,
//                     host: b"foo",
//                     port: Some(8000)
//                 })
//             }),
//             "foo:bar" => Uri::Absolute(Absolute {
//                 scheme: b"foo",
//                 authority: None,
//                 origin: Some(Origin {
//                     path: b"bar",
//                     query: None
//                 }),
//             }),
//             "http://sergio:pass@foo.com:8000" => Uri::Absolute(Absolute {
//                 scheme: b"http",
//                 authority: Some(Authority {
//                     userinfo: Some(b"sergio:pass"),
//                     host: b"foo.com",
//                     port: Some(8000)
//                 }),
//                 origin: None,
//             }),
//             "foo:/sergio/pass?hi" => Uri::Absolute(Absolute {
//                 scheme: b"foo",
//                 authority: None,
//                 origin: Some(Origin {
//                     path: b"/sergio/pass",
//                     query: Some(b"hi")
//                 }),
//             }),
//             "bar:" => Uri::Absolute(Absolute {
//                 scheme: b"bar",
//                 authority: None,
//                 origin: Some(Origin {
//                     path: b"",
//                     query: None
//                 }),
//             }),
//             "foo:?hi" => Uri::Absolute(Absolute {
//                 scheme: b"foo",
//                 authority: None,
//                 origin: Some(Origin {
//                     path: b"",
//                     query: Some(b"hi")
//                 }),
//             }),
//             "foo:a/b?hi" => Uri::Absolute(Absolute {
//                 scheme: b"foo",
//                 authority: None,
//                 origin: Some(Origin {
//                     path: b"a/b",
//                     query: Some(b"hi")
//                 }),
//             }),
//             "foo:a/b" => Uri::Absolute(Absolute {
//                 scheme: b"foo",
//                 authority: None,
//                 origin: Some(Origin {
//                     path: b"a/b",
//                     query: None
//                 }),
//             }),
//             "foo:/a/b" => Uri::Absolute(Absolute {
//                 scheme: b"foo",
//                 authority: None,
//                 origin: Some(Origin {
//                     path: b"/a/b",
//                     query: None
//                 }),
//             }),
//             "abc://u:p@foo.com:123/a/b?key=value&key2=value2" => Uri::Absolute(Absolute {
//                 scheme: b"abc",
//                 authority: Some(Authority {
//                     userinfo: Some(b"u:p"),
//                     host: b"foo.com",
//                     port: Some(123)
//                 }),
//                 origin: Some(Origin {
//                     path: b"/a/b",
//                     query: Some(b"key=value&key2=value2")
//                 }),
//             }),
//             "ftp://foo.com:21/abc" => Uri::Absolute(Absolute {
//                 scheme: b"ftp",
//                 authority: Some(Authority {
//                     userinfo: None,
//                     host: b"foo.com",
//                     port: Some(21),
//                 }),
//                 origin: Some(Origin {
//                     path: b"/abc",
//                     query: None
//                 }),
//             }),
//             "http://google.com/abc" => Uri::Absolute(Absolute {
//                 scheme: b"http",
//                 authority: Some(Authority {
//                     userinfo: None,
//                     host: b"google.com",
//                     port: None,
//                 }),
//                 origin: Some(Origin {
//                     path: b"/abc",
//                     query: None
//                 }),
//             }),
//             "http://google.com" => Uri::Absolute(Absolute {
//                 scheme: b"http",
//                 authority: Some(Authority {
//                     userinfo: None,
//                     host: b"google.com",
//                     port: None,
//                 }),
//                 origin: None
//             }),
//             "http://foo.com?test" => Uri::Absolute(Absolute {
//                 scheme: b"http",
//                 authority: Some(Authority {
//                     userinfo: None,
//                     host: b"foo.com",
//                     port: None,
//                 }),
//                 origin: Some(Origin {
//                     path: b"",
//                     query: Some(b"test")
//                 }),
//             }),
//             "http://google.com/abc?hi" => Uri::Absolute(Absolute {
//                 scheme: b"http",
//                 authority: Some(Authority {
//                     userinfo: None,
//                     host: b"google.com",
//                     port: None,
//                 }),
//                 origin: Some(Origin {
//                     path: b"/abc",
//                     query: Some(b"hi")
//                 }),
//             }),
//         );
//     }
// }
