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

#[derive(PartialEq)]
pub enum Expected<I: Input> {
    Token(Option<I::Token>, Option<I::Token>),
    Slice(Option<I::Slice>, Option<I::Slice>),
    Custom(Cow<'static, str>),
    Eof(Option<I::Token>),
}

impl<I: Input> Debug for Expected<I>
    where I::Token: Debug, I::Slice: Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expected::Token(e, v) => f.debug_tuple("Expected::Token").field(&e).field(&v).finish(),
            Expected::Slice(e, v) => f.debug_tuple("Expected::Slice").field(&e).field(&v).finish(),
            Expected::Custom(s) => f.debug_tuple("Expected::Custom").field(&s).finish(),
            Expected::Eof(v) => f.debug_tuple("Expected::Eof").field(&v).finish(),
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

// impl<T, I, S> Expected<T, I, S> {
//     pub fn map<FT, FI, FS, OT, OI, OS>(self, f_t: FT, f_i: FI, f_s: FS) -> Expected<OT, OI, OS>
//         where FT: Copy + Fn(T) -> OT, FI: Fn(I) -> OI, FS: Fn(S) -> OS
//     {
//         match self {
//             Expected::Token(a, b) => Expected::Token(a.map(f_t), b.map(f_t)),
//             Expected::Slice(a, b) => Expected::Slice(a.map(f_i), b.map(f_s)),
//             Expected::Custom(msg) => Expected::Custom(msg),
//             Expected::Eof(t) => Expected::Eof(t.map(f_t))
//         }
//     }
// }

// FIXME: Dedup this.

impl<I: Input> Display for Expected<I>
    where I::Token: Debug, I::Slice: Debug, I::Many: Debug
{
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
            Expected::Eof(None) => {
                write!(f, "expected EOF but input remains")
            }
            Expected::Eof(Some(ref token)) => {
                write!(f, "expected EOF but found {:?}", token)
            }
        }
    }
}

impl<I: Input> Display for Expected<I>
    where I::Token: Debug + Display,
          I::Slice: Debug + Display,
          I::Many: Debug + Display
{
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
            Expected::Eof(None) => {
                write!(f, "expected EOF but input remains")
            }
            Expected::Eof(Some(ref token)) => {
                write!(f, "expected EOF but found {}", token)
            }
        }
    }
}

#[derive(PartialEq)]
pub struct ParseErr<I: Input> {
    pub parser: &'static str,
    pub expected: Expected<I>,
    pub context: Option<I::Context>,
}

impl<I: Input> Clone for ParseErr<I>
    where I::Token: Clone, I::Slice: Clone, I::Context: Clone
{
    fn clone(&self) -> Self {
        ParseErr {
            parser: self.parser,
            expected: self.expected.clone(),
            context: self.context.clone(),
        }
    }
}

impl<I: Input> Debug for ParseErr<I>
    where Expected<I>: Debug, I::Context: Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ParseErr")
            .field("parser", &self.parser)
            .field("expected", &self.expected)
            .field("context", &self.context)
            .finish()
    }
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
    where I::Token: Debug + Display, I::Slice: Display + Debug, I::Many: Display + Debug
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
    where Expected<I>: Display, I::Context: Display
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

// use std::borrow::Cow;
// use std::fmt::{self, Display, Debug};

// use Input;

// pub trait AsResult<T, I: Input, E, F> {
//     fn as_result(self) -> Result<T, I, E, F>;
// }

// impl<T, I: Input, E, F> AsResult<T, I, E, F> for T {
//     fn as_result(self) -> Result<T, I, E, F> {
//         Ok(self)
//     }
// }

// // // This one will result in inference issues when `Ok(T)` is returned.
// // impl<T, I: Input, E: ::std::fmt::Display> AsResult<T, I> for ::std::result::Result<T, E> {
// //     fn as_result(self) -> Result<T, I> {
// //         let name = unsafe { ::std::intrinsics::type_name::<E>() };
// //         self.map_err(|e| ParseErr::new(name, e.to_string()))
// //     }
// // }

// // This one won't but makes some things uglier to write.
// impl<T, I: Input, E, F> AsResult<T, I, E, F> for Result<T, I, E, F> {
//     fn as_result(self) -> Result<T, I, E, F> {
//         self
//     }
// }

// #[derive(Debug, Clone, PartialEq)]
// pub enum Expected<I: Input> {
//     Token(Option<Box<PartialEq<I::Token>>>, Option<I::Token>),
//     Slice(Option<Box<PartialEq<I::Slice>>>, Option<I::Slice>),
//     Custom(Cow<'static, str>),
//     EOF(Option<I::Token>)
// }

// // impl<T, I, S> Expected<T, I, S> {
// //     pub fn map<FT, FI, FS, OT, OI, OS>(self, f_t: FT, f_i: FI, f_s: FS) -> Expected<OT, OI, OS>
// //         where FT: Copy + Fn(T) -> OT, FI: Fn(I) -> OI, FS: Fn(S) -> OS
// //     {
// //         match self {
// //             Expected::Token(a, b) => Expected::Token(a.map(f_t), b.map(f_t)),
// //             Expected::Slice(a, b) => Expected::Slice(a.map(f_i), b.map(f_s)),
// //             Expected::Custom(msg) => Expected::Custom(msg),
// //             Expected::Eof(t) => Expected::Eof(t.map(f_t))
// //         }
// //     }
// // }

// // FIXME: Dedup this.

// impl<E: Debug, F: Debug> Display for Expected<E, F> {
//     default fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

// impl<E: Display + Debug, F: Display + Debug> Display for Expected<E, F> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match *self {
//             Expected::Token(Some(ref expected), Some(ref found)) => {
//                 write!(f, "expected token {} but found {}", expected, found)
//             }
//             Expected::Token(None, Some(ref found)) => {
//                 write!(f, "unexpected token: {}", found)
//             }
//             Expected::Token(Some(ref expected), None) => {
//                 write!(f, "expected token {} but none was found", expected)
//             }
//             Expected::Token(None, None) => {
//                 write!(f, "expected any token but none was found")
//             }
//             Expected::Slice(Some(ref expected), Some(ref found)) => {
//                 write!(f, "expected slice {} but found {}", expected, found)
//             }
//             Expected::Slice(None, Some(ref found)) => {
//                 write!(f, "unexpected slice: {}", found)
//             }
//             Expected::Slice(Some(ref expected), None) => {
//                 write!(f, "expected slice {} but none was found", expected)
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
//                 write!(f, "expected EOF but found {}", token)
//             }
//         }
//     }
// }


// #[derive(Debug, Clone, PartialEq)]
// pub struct ParseErr<I: Input, E, F> {
//     pub parser: &'static str,
//     pub expected: Expected<E, F>,
//     pub context: Option<I::Context>,
// }

// impl<I: Input, E, F> ParseErr<I, E, F> {
//     #[inline(always)]
//     pub fn new<T>(parser: &'static str, message: T) -> ParseErr<I, E, F>
//         where T: Into<Cow<'static, str>>
//     {
//         ParseErr {
//             parser,
//             expected: Expected::Custom(message.into()),
//             context: None
//         }
//     }

//     #[inline(always)]
//     pub fn from_context<T>(input: &mut I, parser: &'static str, message: T) -> ParseErr<I, E, F>
//         where T: Into<Cow<'static, str>>
//     {
//         ParseErr {
//             parser,
//             expected: Expected::Custom(message.into()),
//             context: input.context()
//         }
//     }
// }

// impl<I: Input, E, F> fmt::Display for ParseErr<I, E, F>
//     where E: Debug + Display, F: Display + Debug,
// {
//     #[inline]
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", self.expected)?;
//         if let Some(ref context) = self.context {
//             write!(f, " ({} at {})", self.parser, context)?;
//         } else {
//             write!(f, " ({})", self.parser)?;
//         }

//         Ok(())
//     }
// }

// impl<I: Input, E, F> fmt::Display for ParseErr<I, E, F>
//     where E: Debug, F: Debug,
// {
//     #[inline]
//     default fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", self.expected)?;
//         if let Some(ref context) = self.context {
//             write!(f, " ({} at {})", self.parser, context)?;
//         } else {
//             write!(f, " ({})", self.parser)?;
//         }

//         Ok(())
//     }
// }


// pub type Result<T, I, E, F> = ::std::result::Result<T, ParseErr<I, E, F>>;
