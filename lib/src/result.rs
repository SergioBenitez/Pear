use crate::error::ParseError;
use crate::input::Input;

pub type Result<R, I> = ::std::result::Result<R, ParseError<I>>;

#[doc(hidden)]
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
//         self.map_err(|e| ParseError::new(name, e.to_string()))
//     }
// }

// This one won't but makes some things uglier to write.
impl<T, I: Input> AsResult<T, I> for Result<T, I> {
    fn as_result(self) -> Result<T, I> {
        self
    }
}
