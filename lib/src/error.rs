use std::fmt;
use std::borrow::Cow;

use crate::input::{Input, Show, ParserInfo};

pub enum Expected<I: Input> {
    // Token(Option<I::Token>, Option<I::Token>),
    // Slice(Option<I::Slice>, Option<I::Slice>),
    Token(Option<String>, Option<I::Token>),
    Slice(Option<String>, Option<I::Slice>),
    Custom(Cow<'static, str>),
    Eof(Option<I::Token>),
}

pub struct ParseContext<I: Input> {
    pub parser: ParserInfo,
    pub context: Option<I::Context>,
}

pub struct ParseError<I: Input> {
    pub expected: Expected<I>,
    pub context: Vec<ParseContext<I>>,
}

impl<I: Input> ParseError<I> {
    pub fn custom<T: Into<Cow<'static, str>>>(message: T) -> ParseError<I> {
        ParseError {
            expected: Expected::Custom(message.into()),
            context: vec![]
        }
    }

    #[inline(always)]
    pub fn expected(expected: Expected<I>) -> ParseError<I> {
        ParseError { expected, context: vec![] }
    }

    pub fn push_context(&mut self, context: Option<I::Context>, parser: ParserInfo) {
        self.context.push(ParseContext { context, parser })
    }
}

impl<I: Input> fmt::Debug for ParseError<I>
    where I::Context: fmt::Debug, I::Slice: fmt::Debug, I::Token: fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParseError")
            .field("expected", &self.expected)
            .field("context", &self.context)
            .finish()
    }
}

impl<I: Input> fmt::Debug for Expected<I> where I::Token: fmt::Debug, I::Slice: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expected::Token(e, v) => {
                f.debug_tuple("Expected::Token").field(&e).field(&v).finish()
            }
            Expected::Slice(e, v) => {
                f.debug_tuple("Expected::Slice").field(&e).field(&v).finish()
            }
            Expected::Custom(s) => {
                f.debug_tuple("Expected::Custom").field(&s).finish()
            }
            Expected::Eof(v) => {
                f.debug_tuple("Expected::Eof").field(&v).finish()
            }
        }
    }
}

impl<I: Input> fmt::Debug for ParseContext<I> where I::Context: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParseContext")
            .field("parser", &self.parser)
            .field("context", &self.context)
            .finish()
    }
}

impl<I: Input> Clone for ParseError<I>
    where I::Token: Clone, I::Slice: Clone, I::Context: Clone
{
    fn clone(&self) -> Self {
        ParseError {
            expected: self.expected.clone(),
            context: self.context.clone(),
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
            Expected::Custom(s) => Expected::Custom(s.clone()),
            Expected::Eof(f) => Expected::Eof(f.clone()),
        }
    }
}

impl<I: Input> Clone for ParseContext<I> where I::Context: Clone {
    fn clone(&self) -> Self {
        ParseContext {
            context: self.context.clone(),
            parser: self.parser.clone()
        }
    }
}

impl<I: Input> fmt::Display for ParseError<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.expected)?;
        for ctxt in &self.context {
            write!(f, "\n + {}", ctxt.parser.name)?;
            if let Some(ctxt) = &ctxt.context {
                write!(f, " at {})", ctxt as &dyn Show)?;
            }
        }

        Ok(())
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
            Expected::Custom(ref message) => {
                write!(f, "{}", message)
            }
            Expected::Eof(None) => {
                write!(f, "expected EOF but input remains")
            }
            Expected::Eof(Some(ref found)) => {
                let found = found as &dyn Show;
                write!(f, "expected EOF but found {}", found)
            }
        }
    }
}
