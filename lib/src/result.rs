use std::borrow::Cow;
use std::fmt::{self, Display, Debug};

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
pub enum Expected<T, I, S> {
    Token(Option<T>, Option<T>),
    Slice(Option<I>, Option<S>),
    Custom(Cow<'static, str>),
    EOF(Option<T>)
}

impl<T, I, S> Expected<T, I, S> {
    pub fn map<FT, FI, FS, OT, OI, OS>(self, f_t: FT, f_i: FI, f_s: FS) -> Expected<OT, OI, OS>
        where FT: Copy + Fn(T) -> OT, FI: Fn(I) -> OI, FS: Fn(S) -> OS
    {
        match self {
            Expected::Token(a, b) => Expected::Token(a.map(f_t), b.map(f_t)),
            Expected::Slice(a, b) => Expected::Slice(a.map(f_i), b.map(f_s)),
            Expected::Custom(msg) => Expected::Custom(msg),
            Expected::EOF(t) => Expected::EOF(t.map(f_t))
        }
    }
}

// FIXME: Dedup this.

impl<T: Debug, I: Debug, S: Debug> Display for Expected<T, I, S> {
    default fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Expected::Token(Some(ref expected), Some(ref found)) => {
                write!(f, "expected token {:?} but found {:?}", expected, found)
            }
            Expected::Token(None, Some(ref found)) => {
                write!(f, "unexpected token: {:?}", found)
            }
            Expected::Token(Some(ref expected), None) => {
                write!(f, "expected token {:?} but none was found", expected)
            }
            Expected::Token(None, None) => {
                write!(f, "expected any token but none was found")
            }
            Expected::Slice(Some(ref expected), Some(ref found)) => {
                write!(f, "expected slice {:?} but found {:?}", expected, found)
            }
            Expected::Slice(None, Some(ref found)) => {
                write!(f, "unexpected slice: {:?}", found)
            }
            Expected::Slice(Some(ref expected), None) => {
                write!(f, "expected slice {:?} but none was found", expected)
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
            Expected::EOF(Some(ref token)) => {
                write!(f, "expected EOF but found {:?}", token)
            }
        }
    }
}

impl<T: Display + Debug, I: Display + Debug, S: Display + Debug> Display for Expected<T, I, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Expected::Token(Some(ref expected), Some(ref found)) => {
                write!(f, "expected token {} but found {}", expected, found)
            }
            Expected::Token(None, Some(ref found)) => {
                write!(f, "unexpected token: {}", found)
            }
            Expected::Token(Some(ref expected), None) => {
                write!(f, "expected token {} but none was found", expected)
            }
            Expected::Token(None, None) => {
                write!(f, "expected any token but none was found")
            }
            Expected::Slice(Some(ref expected), Some(ref found)) => {
                write!(f, "expected slice {} but found {}", expected, found)
            }
            Expected::Slice(None, Some(ref found)) => {
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
            Expected::EOF(None) => {
                write!(f, "expected EOF but input remains")
            }
            Expected::EOF(Some(ref token)) => {
                write!(f, "expected EOF but found {}", token)
            }
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct ParseErr<I: Input> {
    pub parser: &'static str,
    pub expected: Expected<I::Token, I::InSlice, I::Slice>,
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

impl<I: Input> fmt::Display for ParseErr<I>
    where I::Token: Debug + Display,
          I::Slice: Display + Debug,
          I::InSlice: Display + Debug
{
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

impl<I: Input> fmt::Display for ParseErr<I>
    where I::Token: Debug, I::Slice: Debug, I::InSlice: Debug
{
    #[inline]
    default fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
