use crate::input::Input;
use crate::error::{ParseError, ExpectedInput};

pub type Result<R, I, C = ExpectedInput<I>>
    = std::result::Result<R, ParseError<I, C>>;

#[doc(hidden)]
pub trait AsResult<T, I: Input, E> {
    fn as_result(self) -> Result<T, I, E>;
}

impl<T, I: Input, E> AsResult<T, I, E> for T {
    fn as_result(self) -> Result<T, I, E> {
        Ok(self)
    }
}

// // This one will result in inference issues when `Ok(T)` is returned.
// impl<T, I: Input, E: ::std::fmt::Display> AsResult<T, I> for ::std::result::Result<T, E> {
//     fn as_result(self) -> Result<T, I> {
//         let name = unsafe { ::std::intrinsics::type_name::<E>() };
//         self.map_err(|e| ParseError::new(name, e.to_string()))
//     }
// }

// // This one won't but makes some things uglier to write.
// impl<T, I: Input, E2, E1: Into<E2>> AsResult<T, I, E2> for Result<T, I, E1> {
//     fn as_result(self) -> Result<T, I, E2> {
//         match self {
//             Ok(v) => Ok(v),
//             Err(e) => Err(ParseError {
//                 error: e.error.into(),
//                 contexts: e.contexts
//             })
//         }
//     }
// }

// This one won't but makes some things uglier to write.
impl<T, I: Input, E> AsResult<T, I, E> for Result<T, I, E> {
    fn as_result(self) -> Result<T, I, E> {
        self
    }
}

// // This one won't but makes some things uglier to write.
// impl<T, I: Input, E> AsResult<T, I, B> for Result<T, I, A> {
//     fn as_result(self) -> Result<T, I, B> {
//         self
//     }
// }
