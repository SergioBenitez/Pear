use std::borrow::Cow;
use std::fmt;

use Input;

pub trait AsResult<T, I: Input> {
    fn as_result(self) -> Result<T, I>;
}

impl<T, I: Input> AsResult<T, I> for T {
    fn as_result(self) -> Result<T, I> {
        Ok(self)
    }
}

// // This one will result in inference issues when `Ok(T)` is returned.
// impl<T, I: Input, E: ::std::fmt::Display> AsResult<T, I> for ::std::result::Result<T, E> {
//     fn as_result(self) -> Result<T, I> {
//         let name = unsafe { ::std::intrinsics::type_name::<E>() };
//         self.map_err(|e| ParseErr::new(name, e.to_string()))
//     }
// }

// This one won't but makes some things uglier to write.
impl<T, I: Input> AsResult<T, I> for Result<T, I> {
    fn as_result(self) -> Result<T, I> {
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expected<I: Input> {
    Token(Option<I::Token>, Option<I::Token>),
    Slice(Option<I::InSlice>, Option<I::Slice>),
    Custom(Cow<'static, str>),
    EOF(Option<I::Token>)
}

impl<I: Input> fmt::Display for Expected<I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Expected::Token(Some(expected), Some(found)) => {
                write!(f, "expected token {:?} but found {:?}", expected, found)
            }
            Expected::Token(None, Some(found)) => {
                write!(f, "the token {:?} was not expected", found)
            }
            Expected::Token(Some(expected), None) => {
                write!(f, "expected the token {:?} but none was found", expected)
            }
            Expected::Token(None, None) => {
                write!(f, "expected any token but none was found")
            }
            Expected::Slice(Some(ref expected), Some(ref found)) => {
                write!(f, "expected slice {:?} but found {:?}", expected, found)
            }
            Expected::Slice(None, Some(ref found)) => {
                write!(f, "the slice {:?} was not expected", found)
            }
            Expected::Slice(Some(ref expected), None) => {
                write!(f, "expected the slice {:?} but none was found", expected)
            }
            Expected::Slice(None, None) => {
                write!(f, "expected any slice but none was found")
            }
            Expected::Custom(ref message) => {
                write!(f, "{}", message)
            }
            Expected::EOF(None) => {
                write!(f, "expected EOF but input remains")
            }
            Expected::EOF(Some(token)) => {
                write!(f, "expected EOF but found {:?}", token)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseErr<I: Input> {
    pub parser: &'static str,
    pub expected: Expected<I>,
    pub context: Option<I::Context>,
}

impl<I: Input> ParseErr<I> {
    #[inline(always)]
    pub fn new<T>(parser: &'static str, message: T) -> ParseErr<I>
        where T: Into<Cow<'static, str>>
    {
        ParseErr {
            parser,
            expected: Expected::Custom(message.into()),
            context: None
        }
    }

    #[inline(always)]
    pub fn from_context<T>(input: &mut I, parser: &'static str, message: T) -> ParseErr<I>
        where T: Into<Cow<'static, str>>
    {
        ParseErr {
            parser,
            expected: Expected::Custom(message.into()),
            context: input.context()
        }
    }
}

impl<I: Input> fmt::Display for ParseErr<I> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.expected)?;
        if let Some(ref context) = self.context {
            write!(f, " ({} at {})", self.parser, context)?;
        } else {
            write!(f, " ({})", self.parser)?;
        }

        Ok(())
    }
}

pub type Result<R, I> = ::std::result::Result<R, ParseErr<I>>;
