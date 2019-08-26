mod input;
mod length;
mod string;
mod text;
mod text_file;

pub use input::*;
pub use text::{Text, Span};
pub use length::Length;

// impl<'a> Input for &'a [u8] {
//     type Token = u8;
//     type Slice = &'a [u8];
//     type Many = Self::Slice;

//     // type Marker = String;
//     type Context = String;

//     /// Returns a copy of the current token, if there is one.
//     fn token(&mut self) -> Option<Self::Token> {
//         self.get(0).map(|&c| c)
//     }

//     /// Returns a copy of the current slice of size `n`, if there is one.
//     fn slice(&mut self, n: usize) -> Option<Self::Slice> {
//         if self.len() < n {
//             None
//         } else {
//             Some(&self[..n])
//         }
//     }

//     /// Checks if the current token fulfills `cond`.
//     fn peek<F>(&mut self, mut cond: F) -> bool
//         where F: FnMut(&Self::Token) -> bool
//     {
//         self.token().map(|t| cond(&t)).unwrap_or(false)
//     }

//     /// Checks if the current slice of size `n` (if any) fulfills `cond`.
//     fn peek_slice<F>(&mut self, n: usize, mut cond: F) -> bool
//         where F: FnMut(&Self::Slice) -> bool
//     {
//         self.slice(n).map(|s| cond(&s)).unwrap_or(false)
//     }

//     /// Checks if the current token fulfills `cond`. If so, the token is
//     /// consumed and returned. Otherwise, returns `None`.
//     fn eat<F>(&mut self, mut cond: F) -> Option<Self::Token>
//         where F: FnMut(&Self::Token) -> bool
//     {
//         if let Some(token) = self.token() {
//             if cond(&token) {
//                 *self = &self[1..];
//                 return Some(token)
//             }
//         }

//         None
//     }

//     /// Checks if the current slice of size `n` (if any) fulfills `cond`. If so,
//     /// the slice is consumed and returned. Otherwise, returns `None`.
//     fn eat_slice<F>(&mut self, n: usize, mut cond: F) -> Option<Self::Slice>
//         where F: FnMut(&Self::Slice) -> bool
//     {
//         if let Some(slice) = self.slice(n) {
//             if cond(&slice) {
//                 *self = &self[slice.len()..];
//                 return Some(slice)
//             }
//         }

//         None
//     }

//     /// Takes tokens while `cond` returns true, collecting them into a
//     /// `Self::Many` and returning it.
//     fn take<F>(&mut self, mut cond: F) -> Self::Many
//         where F: FnMut(&Self::Token) -> bool
//     {
//         let mut consumed = 0;
//         for c in self.iter() {
//             if !cond(c) { break; }
//             consumed += 1;
//         }

//         let value = &self[..consumed];
//         *self = &self[consumed..];
//         value
//     }

//     /// Skips tokens while `cond` returns true. Returns the number of skipped
//     /// tokens.
//     fn skip<F>(&mut self, mut cond: F) -> usize
//         where F: FnMut(&Self::Token) -> bool
//     {
//         let mut skipped = 0;
//         match self.take(|c| {  skipped += 1; cond(c) }) {
//             &[] => 0,
//             _ => skipped - 1
//         }
//     }

//     /// Returns `true` if there are no more tokens.
//     fn is_eof(&mut self) -> bool {
//         self.is_empty()
//     }

//     #[inline(always)]
//     fn context(&mut self) -> Option<Self::Context> {
//         let n = ::std::cmp::min(self.len(), 5);
//         if n == 0 {
//             return None;
//         }

//         let bytes = &self[..n];
//         if let Ok(string) = ::std::str::from_utf8(bytes) {
//             Some(format!("{:?}", string))
//         } else {
//             Some(format!("{:?}", bytes))
//         }
//     }
// }
