use std::borrow::Cow;
use std::fmt::{self, Display, Debug};

use crate::input::{Input, ParserInfo};

pub enum Expected<I: Input> {
    Token(Option<I::Token>, Option<I::Token>),
    Slice(Option<I::Slice>, Option<I::Slice>),
    Custom(Cow<'static, str>),
    Eof(Option<I::Token>),
}

pub struct ParseContext<I: Input> {
    parser: ParserInfo,
    context: Option<I::Context>,
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

impl<I: Input> Debug for ParseError<I>
    where I::Context: Debug, I::Slice: Debug, I::Token: Debug
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParseError")
            .field("expected", &self.expected)
            .field("context", &self.context)
            .finish()
    }
}

impl<I: Input> Debug for Expected<I> where I::Token: Debug, I::Slice: Debug {
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

impl<I: Input> Debug for ParseContext<I> where I::Context: Debug {
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

// // FIXME: Dedup this.

impl<I: Input> fmt::Display for ParseError<I>
    where Expected<I>: Display, I::Context: Display
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.expected)?;
        for ctxt in &self.context {
            write!(f, "\n + ({}", ctxt.parser.name)?;
            if let Some(ctxt) = &ctxt.context {
                write!(f, " at {})", ctxt)?;
            } else {
                write!(f, ")")?;
            }
        }

        Ok(())
    }
}

impl<I: Input> Display for Expected<I>
    where I::Token: Debug + Display,
          I::Slice: Debug + Display,
          I::Many: Debug + Display
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Expected::Token(Some(ref expected), Some(ref found)) => {
                write!(f, "expected token {:?} but found {:?}", expected, found)
                // write!(f, "expected token {} but found {}", expected, found)
            }
            Expected::Token(None, Some(ref found)) => {
                write!(f, "unexpected token: {:?}", found)
                // write!(f, "unexpected token: {}", found)
            }
            Expected::Token(Some(ref expected), None) => {
                write!(f, "expected token {:?} but none was found", expected)
                // write!(f, "expected token {} but none was found", expected)
            }
            Expected::Token(None, None) => {
                write!(f, "expected any token but none was found")
            }
            Expected::Slice(Some(ref expected), Some(ref found)) => {
                write!(f, "expected slice {:?} but found {:?}", expected, found)
                // write!(f, "expected slice {} but found {}", expected, found)
            }
            Expected::Slice(None, Some(ref found)) => {
                write!(f, "unexpected slice: {:?}", found)
                // write!(f, "unexpected slice: {}", found)
            }
            Expected::Slice(Some(ref expected), None) => {
                write!(f, "expected slice {:?} but none was found", expected)
                // write!(f, "expected slice {} but none was found", expected)
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
            Expected::Eof(Some(ref token)) => {
                write!(f, "expected EOF but found {:?}", token)
                // write!(f, "expected EOF but found {}", token)
            }
        }
    }
}

// impl<I: Input> Display for Expected<I>
//     where I::Token: Debug, I::Slice: Debug, I::Many: Debug
// {
//     default fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match *self {
//             Expected::Token(Some(ref expected), Some(ref found)) => {
//                 write!(f, "expected token {:?} but found {:?}", expected, found)
//             }
//             Expected::Token(None, Some(ref found)) => {
//                 write!(f, "unexpected token: {:?}", found)
//             }
//             Expected::Token(Some(ref expected), None) => {
//                 write!(f, "expected token {:?} but none was found", expected)
//             }
//             Expected::Token(None, None) => {
//                 write!(f, "expected any token but none was found")
//             }
//             Expected::Slice(Some(ref expected), Some(ref found)) => {
//                 write!(f, "expected slice {:?} but found {:?}", expected, found)
//             }
//             Expected::Slice(None, Some(ref found)) => {
//                 write!(f, "unexpected slice: {:?}", found)
//             }
//             Expected::Slice(Some(ref expected), None) => {
//                 write!(f, "expected slice {:?} but none was found", expected)
//             }
//             Expected::Slice(None, None) => {
//                 write!(f, "expected any slice but none was found")
//             }
//             Expected::Custom(ref message) => {
//                 write!(f, "{}", message)
//             }
//             Expected::Eof(None) => {
//                 write!(f, "expected EOF but input remains")
//             }
//             Expected::Eof(Some(ref token)) => {
//                 write!(f, "expected EOF but found {:?}", token)
//             }
//         }
//     }
// }
