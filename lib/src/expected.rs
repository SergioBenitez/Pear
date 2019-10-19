use std::fmt;
use std::borrow::Cow;

use crate::input::{Input, Show};

pub enum Expected<I: Input> {
    // Token(Option<I::Token>, Option<I::Token>),
    // Slice(Option<I::Slice>, Option<I::Slice>),
    Token(Option<String>, Option<I::Token>),
    Slice(Option<String>, Option<I::Slice>),
    Eof(Option<I::Token>),
    Other(Cow<'static, str>),
}

impl<I: Input> From<String> for Expected<I> {
    fn from(string: String) -> Expected<I> {
        Expected::Other(string.into())
    }
}

impl<I: Input> From<&'static str> for Expected<I> {
    fn from(string: &'static str) -> Expected<I> {
        Expected::Other(string.into())
    }
}

impl<I: Input> fmt::Debug for Expected<I>
    where I::Token: fmt::Debug, I::Slice: fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expected::Token(e, v) => {
                f.debug_tuple("Expected::Token").field(&e).field(&v).finish()
            }
            Expected::Slice(e, v) => {
                f.debug_tuple("Expected::Slice").field(&e).field(&v).finish()
            }
            Expected::Eof(v) => {
                f.debug_tuple("Expected::Eof").field(&v).finish()
            }
            Expected::Other(v) => {
                f.debug_tuple("Expected::Other").field(&v).finish()
            }
        }
    }
}

impl<I: Input> Clone for Expected<I>
    where I::Token: Clone, I::Slice: Clone
{
    fn clone(&self) -> Self {
        match self {
            Expected::Token(e, f) => Expected::Token(e.clone(), f.clone()),
            Expected::Slice(e, f) => Expected::Slice(e.clone(), f.clone()),
            Expected::Eof(f) => Expected::Eof(f.clone()),
            Expected::Other(v) => Expected::Other(v.clone())
        }
    }
}

impl<I: Input> fmt::Display for Expected<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Expected::Token(Some(ref expected), Some(ref found)) => {
                let found = found as &dyn Show;
                write!(f, "expected token {} but found {}", expected, found)
            }
            Expected::Token(None, Some(ref found)) => {
                let found = found as &dyn Show;
                write!(f, "unexpected token: {}", found)
            }
            Expected::Token(Some(ref expected), None) => {
                write!(f, "expected token {} but none was found", expected)
            }
            Expected::Token(None, None) => {
                write!(f, "expected any token but none was found")
            }
            Expected::Slice(Some(ref expected), Some(ref found)) => {
                let found = found as &dyn Show;
                write!(f, "expected slice {} but found {}", expected, found)
            }
            Expected::Slice(None, Some(ref found)) => {
                let found = found as &dyn Show;
                write!(f, "unexpected slice: {}", found)
            }
            Expected::Slice(Some(ref expected), None) => {
                write!(f, "expected slice {} but none was found", expected)
            }
            Expected::Slice(None, None) => {
                write!(f, "expected any slice but none was found")
            }
            Expected::Eof(None) => {
                write!(f, "expected EOF but input remains")
            }
            Expected::Eof(Some(ref found)) => {
                let found = found as &dyn Show;
                write!(f, "expected EOF but found {}", found)
            }
            Expected::Other(ref other) => {
                write!(f, "expected {}", other)
            }
        }
    }
}
