#![feature(proc_macro_hygiene)]
#![allow(unused_imports, dead_code)]

#[macro_use] extern crate pear;

mod tables;
mod utils;
mod indexed;

use std::borrow::Cow;
use std::str::{from_utf8, from_utf8_unchecked};
use std::fmt::{self, Display};

use pear::{Length, parser, switch};
use pear::parsers::*;
use pear::combinators::*;

// use utils::merge;
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

// type Input<'a> = IndexedInput<'a, [u8]>;
// pear_declare!(Input<'a>(Token = u8, Slice = &'a [u8], Many = &'a [u8]));

// #[derive(Debug, PartialEq)]
// pub enum Error<I: Input> {
//     Empty,
//     Parse(ParseError<I>)
// }

// impl<I: Input> From<ParseError<I>> for Error<I> {
//     #[inline(always)]
//     fn from(error: ParseError<I>) -> Self {
//         Error::Parse(error)
//     }
// }

type ByteInput<'a> = IndexedInput<'a, [u8]>;
type IndexedStr<'a> = Indexed<'a, str>;
type IndexedBytes<'a> = Indexed<'a, [u8]>;

#[derive(Debug, PartialEq)]
pub enum Uri<'a> {
    Origin(Origin<'a>),
    Authority(Authority<'a>),
    Absolute(Absolute<'a>),
    Asterisk,
}

macro_rules! impl_uri_from {
    ($type:ident) => (
        impl<'a> From<$type<'a>> for Uri<'a> {
            fn from(other: $type<'a>) -> Uri<'a> {
                Uri::$type(other)
            }
        }
    )
}

impl_uri_from!(Origin);
impl_uri_from!(Authority);
impl_uri_from!(Absolute);

impl<'a> Uri<'a> {
    fn origin(path: &'a str, query: Option<&'a str>) -> Uri<'a> {
        Uri::Origin(Origin::new(path, query))
    }

    #[inline]
    unsafe fn raw_absolute(
        source: Cow<'a, [u8]>,
        scheme: Indexed<'a, [u8]>,
        path: Indexed<'a, [u8]>,
        query: Option<Indexed<'a, [u8]>>,
    ) -> Uri<'a> {
        let origin = Origin::raw(source.clone(), path, query);
        Uri::Absolute(Absolute::raw(source.clone(), scheme, None, Some(origin)))
    }
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

pub unsafe fn as_utf8_unchecked(input: Cow<[u8]>) -> Cow<str> {
    match input {
        Cow::Borrowed(bytes) => Cow::Borrowed(::std::str::from_utf8_unchecked(bytes)),
        Cow::Owned(bytes) => Cow::Owned(String::from_utf8_unchecked(bytes))
    }
}

impl<'a> Origin<'a> {
    #[inline]
    unsafe fn raw(source: Cow<'a, [u8]>, path: Indexed<'a, [u8]>, query: Option<Indexed<'a, [u8]>>) -> Origin<'a> {
        Origin {
            source: Some(as_utf8_unchecked(source)),
            path: path.coerce(),
            query: query.map(|q| q.coerce())
        }
    }

    fn new<P, Q>(path: P, query: Option<Q>) -> Origin<'a>
        where P: Into<Cow<'a, str>>, Q: Into<Cow<'a, str>>
    {
        Origin {
            source: None,
            path: Indexed::from(path),
            query: query.map(|q| Indexed::from(q))
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
pub enum Host<T> {
    Bracketed(T),
    Raw(T)
}

impl<T> Host<T> {
    #[inline]
    fn inner(&self) -> &T {
        match *self {
            Host::Bracketed(ref inner) | Host::Raw(ref inner) => inner
        }
    }

    #[inline]
    fn is_bracketed(&self) -> bool {
        match *self {
            Host::Bracketed(_) => true,
            _ => false
        }
    }

    #[inline]
    fn map_inner<F, U>(self, f: F) -> Host<U>
        where F: FnOnce(T) -> U
    {
        match self {
            Host::Bracketed(inner) => Host::Bracketed(f(inner)),
            Host::Raw(inner) => Host::Raw(f(inner))
        }
    }
}

#[derive(Debug)]
pub struct Authority<'a> {
    source: Option<Cow<'a, str>>,
    user_info: Option<IndexedStr<'a>>,
    host: Host<IndexedStr<'a>>,
    port: Option<u16>,
}

impl<'a> Authority<'a> {
    unsafe fn raw(
        source: Cow<'a, [u8]>,
        user_info: Option<Indexed<'a, [u8]>>,
        host: Host<Indexed<'a, [u8]>>,
        port: Option<u16>
    ) -> Authority<'a> {
        Authority {
            source: Some(as_utf8_unchecked(source)),
            user_info: user_info.map(|u| u.coerce()),
            host: host.map_inner(|inner| inner.coerce()),
            port: port
        }
    }

    fn new(
        user_info: Option<&'a str>,
        host: Host<&'a str>,
        port: Option<u16>
    ) -> Authority<'a> {
        Authority {
            source: None,
            user_info: user_info.map(|u| u.into()),
            host: host.map_inner(|inner| inner.into()),
            port: port
        }
    }

    pub fn user_info(&self) -> Option<&str> {
        self.user_info.as_ref().map(|u| u.to_source(&self.source))
    }

    #[inline(always)]
    pub fn host(&self) -> &str {
        self.host.inner().to_source(&self.source)
    }

    #[inline(always)]
    pub fn port(&self) -> Option<u16> {
        self.port
    }
}

impl<'a, 'b> PartialEq<Authority<'b>> for Authority<'a> {
    fn eq(&self, other: &Authority<'b>) -> bool {
        self.user_info() == other.user_info()
            && self.host() == other.host()
            && self.host.is_bracketed() == other.host.is_bracketed()
            && self.port() == other.port()
    }
}

impl<'a> Display for Authority<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(user_info) = self.user_info() {
            write!(f, "{}@", user_info)?;
        }

        match self.host {
            Host::Bracketed(_) => write!(f, "[{}]", self.host())?,
            Host::Raw(_) => write!(f, "{}", self.host())?
        }

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
    #[inline]
    unsafe fn raw(
        source: Cow<'a, [u8]>,
        scheme: Indexed<'a, [u8]>,
        authority: Option<Authority<'a>>,
        origin: Option<Origin<'a>>,
    ) -> Absolute<'a> {
        Absolute {
            source: Some(as_utf8_unchecked(source)),
            scheme: scheme.coerce(),
            authority: authority,
            origin: origin,
        }
    }

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
    pub fn authority(&self) -> Option<&Authority<'a>> {
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

type RawInput<'a> = IndexedInput<'a, [u8]>;
type Result<'a, T> = ::pear::Result<T, RawInput<'a>>;

#[parser]
fn uri<'a>(input: &mut RawInput<'a>) -> Result<'a, Uri<'a>> {
    match input.len() {
        0 => return Err(pear_error!("empty URI")),
        1 => switch! {
            eat(b'*') => Uri::Asterisk,
            eat(b'/') => Uri::origin("/", None),
            _ => unsafe {
                // the `is_reg_name_char` guarantees ASCII
                let host = Host::Raw(take_n_if(1, is_reg_name_char)?);
                Uri::Authority(Authority::raw(input.cow_source(), None, host, None))
            }
        },
        _ => switch! {
            peek(b'/') => Uri::Origin(origin()?),
            _ => absolute_or_authority()?
        }
    }
}

#[parser]
fn origin<'a>(input: &mut RawInput<'a>) -> Result<'a, Origin<'a>> {
    (peek(b'/')?, path_and_query()?).1
}

#[parser]
fn path_and_query<'a>(input: &mut RawInput<'a>) -> Result<'a, Origin<'a>> {
    let path = take_while(is_pchar)?;
    let query = pear_try!(eat(b'?') => take_while(is_pchar)?);

    if path.is_empty() && query.is_none() {
        Err(pear_error!("expected path or query, found neither"))
    } else {
        // We know the string is ASCII because of the `is_pchar` checks above.
        Ok(unsafe { Origin::raw(input.cow_source(), path, query) })
    }
}

#[parser]
fn port<'a>(input: &mut RawInput<'a>) -> Result<'a, u16> {
    let port_str = take_n_while(5, |c| c >= b'0' && c <= b'9')?;

    let mut port_num: u32 = 0;
    let source = &Some(input.cow_source());
    let string = port_str.to_source(source);
    for (b, i) in string.iter().rev().zip(&[1, 10, 100, 1000, 10000]) {
        port_num += (*b - b'0') as u32 * i;
    }

    if port_num > u16::max_value() as u32 {
        return Err(pear_error!("port value out of range: {}", port_num));
    }

    port_num as u16
}

#[parser]
fn authority<'a>(
    input: &mut RawInput<'a>,
    user_info: Option<Indexed<'a, [u8]>>
) -> Result<'a, Authority<'a>> {
    let host = switch! {
        peek(b'[') => Host::Bracketed(delimited(b'[', is_pchar, b']')?),
        _ => Host::Raw(take_while(is_reg_name_char)?)
    };

    // The `is_pchar`,`is_reg_name_char`, and `port()` functions ensure ASCII.
    let port = pear_try!(eat(b':') => port()?);
    unsafe { Authority::raw(input.cow_source(), user_info, host, port) }
}

// Callers must ensure that `scheme` is actually ASCII.
#[parser]
fn absolute<'a>(
    input: &mut RawInput<'a>,
    scheme: Indexed<'a, [u8]>
) -> Result<'a, Absolute<'a>> {
    let (authority, path_and_query) = switch! {
        eat_slice(b"://") => {
            let left = take_while(|c| is_reg_name_char(c) || c == b':')?;
            let authority = switch! {
                eat(b'@') => authority(Some(left))?,
                _ => {
                    input.backtrack(left.len())?;
                    authority(None)?
                }
            };

            let path_and_query = pear_try!(path_and_query());
            (Some(authority), path_and_query)
        },
        eat(b':') => (None, Some(path_and_query()?)),
        _ => return Err(pear_error!("something"))
    };

    // `authority` and `path_and_query` parsers ensure ASCII.
    unsafe { Absolute::raw(input.cow_source(), scheme, authority, path_and_query) }
}

#[parser]
fn absolute_or_authority<'a>(
    input: &mut RawInput<'a>,
) -> Result<'a, Uri<'a>> {
    let left = take_while(is_reg_name_char)?;
    switch! {
        peek_slice(b":/") => Uri::Absolute(absolute(left)?),
        eat(b'@') => Uri::Authority(authority(Some(left))?),
        colon@take_n_if(1, |b| b == b':') => {
            // could be authority or an IP with ':' in it
            let rest = take_while(|c| is_reg_name_char(c) || c == b':')?;
            switch! {
                eat(b'@') => Uri::Authority(authority(Some(left + colon + rest))?),
                peek(b'/') => {
                    input.backtrack(rest.len() + 1)?;
                    Uri::Absolute(absolute(left)?)
                },
                _ => unsafe {
                    // `left` and `rest` are reg_name, `query` is pchar.
                    let query = pear_try!(eat(b'?') => take_while(is_pchar)?);
                    Uri::raw_absolute(input.cow_source(), left, rest, query)
                }
            }
        },
        _ => {
            input.backtrack(left.len())?;
            Uri::Authority(authority(None)?)
        }
    }
}

pub fn parse_bytes<'a>(data: &'a [u8]) -> Result<Uri<'a>> {
    parse!(uri: &mut IndexedInput::from(data))
}

#[cfg(test)]
mod test {
    use super::*;
    use super::Host::*;

    fn parse_str(string: &str) -> ::pear::Result<Uri, RawInput> {
        parse!(uri: &mut IndexedInput::from(string.as_bytes()))
    }

    macro_rules! assert_parse_eq {
        ($($from:expr => $to:expr),+) => (
            $(
                let expected = $to.into();
                match parse_str($from) {
                    Ok(output) => {
                        if output != expected {
                            println!("Failure on: {:?}", $from);
                            assert_eq!(output, expected);
                        }
                    }
                    Err(e) => {
                        println!("{:?} failed to parse!", $from);
                        panic!("Error: {}", e);
                    }
                }
             )+
        );

        ($($from:expr => $to:expr),+,) => (assert_parse_eq!($($from => $to),+))
    }

    macro_rules! assert_no_parse {
        ($($from:expr),+) => (
            $(
                if let Ok(uri) = parse_str($from) {
                    println!("{:?} parsed unexpectedly!", $from);
                    panic!("Parsed as: {:?}", uri);
                }
             )+
        );

        ($($from:expr),+,) => (assert_no_parse!($($from),+))
    }

    macro_rules! assert_displays_eq {
        ($($string:expr),+) => (
            $(
                let string = $string.into();
                match parse_str(string) {
                    Ok(output) => {
                        let output_string = output.to_string();
                        if output_string != string {
                            println!("Failure on: {:?}", $string);
                            println!("Got: {:?}", output_string);
                            println!("Parsed as: {:?}", output);
                            panic!("failed");
                        }
                    }
                    Err(e) => {
                        println!("{:?} failed to parse!", $string);
                        panic!("Error: {}", e);
                    }
                }
             )+
        );

        ($($string:expr),+,) => (assert_parse_eq!($($string),+))
    }


    #[test]
    #[should_panic]
    fn test_assert_parse_eq() {
        assert_parse_eq!("*" => Uri::origin("*", None));
    }

    #[test]
    #[should_panic]
    fn test_assert_parse_eq_consecutive() {
        assert_parse_eq!("/" => Uri::origin("/", None), "/" => Uri::Asterisk);
    }

    #[test]
    #[should_panic]
    fn test_assert_no_parse() {
        assert_no_parse!("/");
    }

    #[test]
    fn bad_parses() {
        assert_no_parse!("://z7:77777777777777777777777777777`77777777777");
    }

    #[test]
    fn single_byte() {
        assert_parse_eq!(
            "*" => Uri::Asterisk,
            "/" => Uri::origin("/", None),
            "." => Authority::new(None, Raw("."), None),
            "_" => Authority::new(None, Raw("_"), None),
            "1" => Authority::new(None, Raw("1"), None),
            "b" => Authority::new(None, Raw("b"), None),
        );

        assert_no_parse!("?", "#", "%");
    }

    #[test]
    fn origin() {
        assert_parse_eq!(
            "/a/b/c" => Uri::origin("/a/b/c", None),
            "/a/b/c?" => Uri::origin("/a/b/c", Some("")),
            "/a/b/c?abc" => Uri::origin("/a/b/c", Some("abc")),
            "/?abc" => Uri::origin("/", Some("abc")),
            "/hi%20there?a=b&c=d" => Uri::origin("/hi%20there", Some("a=b&c=d")),
            "/c/d/fa/b/c?abc" => Uri::origin("/c/d/fa/b/c", Some("abc")),
            "/xn--ls8h?emoji=poop" => Uri::origin("/xn--ls8h", Some("emoji=poop")),
        );
    }

    #[test]
    fn authority() {
        assert_parse_eq!(
            "abc" => Authority::new(None, Raw("abc"), None),
            "@abc" => Authority::new(Some(""), Raw("abc"), None),
            "sergio:benitez@spark" => Authority::new(Some("sergio:benitez"), Raw("spark"), None),
            "a:b:c@1.2.3:12121" => Authority::new(Some("a:b:c"), Raw("1.2.3"), Some(12121)),
            "sergio@spark" => Authority::new(Some("sergio"), Raw("spark"), None),
            "sergio@spark:230" => Authority::new(Some("sergio"), Raw("spark"), Some(230)),
            "sergio@[1::]:230" => Authority::new(Some("sergio"), Bracketed("1::"), Some(230)),
        );
    }

    #[test]
    fn absolute() {
        assert_parse_eq!(
            "http://foo.com:8000" => Absolute::new(
                "http",
                Some(Authority::new(None, Raw("foo.com"), Some(8000))),
                None
            ),
            "http://foo:8000" => Absolute::new(
                "http",
                Some(Authority::new(None, Raw("foo"), Some(8000))),
                None,
            ),
            "foo:bar" => Absolute::new(
                "foo",
                None,
                Some(Origin::new::<_, &str>("bar", None)),
            ),
            "http://sergio:pass@foo.com:8000" => Absolute::new(
                "http",
                Some(Authority::new(Some("sergio:pass"), Raw("foo.com"), Some(8000))),
                None,
            ),
            "foo:/sergio/pass?hi" => Absolute::new(
                "foo",
                None,
                Some(Origin::new("/sergio/pass", Some("hi"))),
            ),
            "bar:" => Absolute::new(
                "bar",
                None,
                Some(Origin::new::<_, &str>("", None)),
            ),
            "foo:?hi" => Absolute::new(
                "foo",
                None,
                Some(Origin::new("", Some("hi"))),
            ),
            "foo:a/b?hi" => Absolute::new(
                "foo",
                None,
                Some(Origin::new("a/b", Some("hi"))),
            ),
            "foo:a/b" => Absolute::new(
                "foo",
                None,
                Some(Origin::new::<_, &str>("a/b", None)),
            ),
            "foo:/a/b" => Absolute::new(
                "foo",
                None,
                Some(Origin::new::<_, &str>("/a/b", None))
            ),
            "abc://u:p@foo.com:123/a/b?key=value&key2=value2" => Absolute::new(
                "abc",
                Some(Authority::new(Some("u:p"), Raw("foo.com"), Some(123))),
                Some(Origin::new("/a/b", Some("key=value&key2=value2"))),
            ),
            "ftp://foo.com:21/abc" => Absolute::new(
                "ftp",
                Some(Authority::new(None, Raw("foo.com"), Some(21))),
                Some(Origin::new::<_, &str>("/abc", None)),
            ),
            "http://google.com/abc" => Absolute::new(
                "http",
                Some(Authority::new(None, Raw("google.com"), None)),
                Some(Origin::new::<_, &str>("/abc", None)),
            ),
            "http://google.com" => Absolute::new(
                "http",
                Some(Authority::new(None, Raw("google.com"), None)),
                None
            ),
            "http://foo.com?test" => Absolute::new(
                "http",
                Some(Authority::new(None, Raw("foo.com"), None,)),
                Some(Origin::new("", Some("test"))),
            ),
            "http://google.com/abc?hi" => Absolute::new(
                "http",
                Some(Authority::new(None, Raw("google.com"), None,)),
                Some(Origin::new("/abc", Some("hi"))),
            ),
        );
    }

    #[test]
    fn display() {
        assert_displays_eq! {
            "abc", "@):0", "[a]"
        }
    }

}
