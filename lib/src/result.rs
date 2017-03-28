use std::borrow::Cow;
use std::fmt;

use Input;
use ParseResult::*;

#[derive(Debug)]
pub enum Expected<I: Input> {
    Token(Option<I::Token>, Option<I::Token>),
    Slice(Option<I::Slice>, Option<I::Slice>),
    Custom(Cow<'static, str>),
    EOF
}

impl<I: Input> fmt::Display for Expected<I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Expected::Token(Some(expected), Some(found)) => {
                write!(f, "expected token {:?}, but found {:?}", expected, found)
            }
            Expected::Token(None, Some(found)) => {
                write!(f, "the token {:?} was not expected", found)
            }
            Expected::Token(Some(expected), None) => {
                write!(f, "expected the token {:?}, but none was found", expected)
            }
            Expected::Token(None, None) => {
                write!(f, "expected any token, but none was found")
            }
            Expected::Slice(Some(expected), Some(found)) => {
                write!(f, "expected slice {:?}, but found {:?}", expected, found)
            }
            Expected::Slice(None, Some(found)) => {
                write!(f, "the slice {:?} was not expected", found)
            }
            Expected::Slice(Some(expected), None) => {
                write!(f, "expected the slice {:?}, but none was found", expected)
            }
            Expected::Slice(None, None) => {
                write!(f, "expected any slice, but none was found")
            }
            Expected::Custom(ref message) => {
                write!(f, "{}", message)
            }
            Expected::EOF => {
                write!(f, "expected EOF but input remains")
            }
        }
    }
}

#[derive(Debug)]
pub struct ParseError<I: Input> {
    pub parser: &'static str,
    pub expected: Expected<I>
}

impl<I: Input> ParseError<I> {
    #[inline(always)]
    pub fn custom<T, R>(parser: &'static str, message: T) -> ParseResult<I, R>
        where T: Into<Cow<'static, str>>
    {
        ParseResult::Error(ParseError {
            parser: parser,
            expected: Expected::Custom(message.into())
        })
    }
}

impl<I: Input> fmt::Display for ParseError<I> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'{}': {}", self.parser, self.expected)
    }
}

#[derive(Debug)]
pub enum ParseResult<I: Input, R> {
    Done(R),
    Error(ParseError<I>)
}

impl<I: Input, R> ParseResult<I, R> {
    #[inline(always)]
    pub fn unwrap(self) -> R {
        match self {
            Done(result) => result,
            Error(e) => panic!("Unwrap on ParseResult::Err: {}", e)
        }
    }

    #[inline(always)]
    pub fn map<U, F: FnOnce(R) -> U>(self, f: F) -> Result<U, ParseError<I>> {
        match self {
            Done(result) => Ok(f(result)),
            Error(e) => Err(e)
        }
    }

    #[inline(always)]
    pub fn ok(self) -> Option<R> {
        match self {
            Done(result) => Some(result),
            Error(_) => None
        }
    }
}

impl<I: Input, T, E: fmt::Display> From<Result<T, E>> for ParseResult<I, T> {
    #[inline]
    fn from(result: Result<T, E>) -> ParseResult<I, T> {
        match result {
            Ok(val) => ParseResult::Done(val),
            Err(e) => ParseResult::Error(ParseError {
                parser: "std::Result",
                expected: Expected::Custom(Cow::Owned(e.to_string()))
            })
        }
    }
}

impl<I: Input, R> Into<Result<R, ParseError<I>>> for ParseResult<I, R> {
    #[inline(always)]
    fn into(self) -> Result<R, ParseError<I>> {
        self.map(|r| r)
    }
}

#[inline(always)]
pub fn error<I: Input, R>(parser: &'static str, expected: Expected<I>) -> ParseResult<I, R> {
    Error(ParseError { parser: parser, expected: expected })
}

