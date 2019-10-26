use std::fmt;
use std::borrow::Cow;

use crate::input::Show;

pub enum Expected<Token, Slice> {
    // Token(Option<I::Token>, Option<I::Token>),
    // Slice(Option<I::Slice>, Option<I::Slice>),
    Token(Option<String>, Option<Token>),
    Slice(Option<String>, Option<Slice>),
    Eof(Option<Token>),
    Other(Cow<'static, str>),
}

impl<T: ToOwned, S: ?Sized + ToOwned> Expected<T, &S> {
    pub fn into_owned(self) -> Expected<T::Owned, S::Owned> {
        use Expected::*;

        match self {
            Token(e, v) => Token(e, v.map(|v| v.to_owned())),
            Slice(e, v) => Slice(e, v.map(|v| v.to_owned())),
            Eof(v) => Eof(v.map(|v| v.to_owned())),
            Other(v) => Other(v),
        }
    }
}

impl<T, S> From<String> for Expected<T, S> {
    fn from(string: String) -> Expected<T, S> {
        Expected::Other(string.into())
    }
}

impl<T, S> From<&'static str> for Expected<T, S> {
    fn from(string: &'static str) -> Expected<T, S> {
        Expected::Other(string.into())
    }
}

impl<T: fmt::Debug, S: fmt::Debug> fmt::Debug for Expected<T, S> {
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

impl<T: Clone, S: Clone> Clone for Expected<T, S> {
    fn clone(&self) -> Self {
        match self {
            Expected::Token(e, f) => Expected::Token(e.clone(), f.clone()),
            Expected::Slice(e, f) => Expected::Slice(e.clone(), f.clone()),
            Expected::Eof(f) => Expected::Eof(f.clone()),
            Expected::Other(v) => Expected::Other(v.clone())
        }
    }
}

impl<T: Show, S: Show> fmt::Display for Expected<T, S> {
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
                write!(f, "{}", other)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Expected;

    #[test]
    fn test_into_owned() {
        let expected: Expected<char, &str> = Expected::Slice(None, Some("hi"));
        let _owned: Expected<char, String> = expected.into_owned();
    }
}
