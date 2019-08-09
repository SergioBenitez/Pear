use std::fmt::Display;

use crate::input::Length;

pub trait Token<I: Input> {
    fn eq_token(&self, other: &I::Token) -> bool;
    fn into_token(self) -> I::Token;
}

impl<I: Input> Token<I> for I::Token {
    default fn eq_token(&self, other: &I::Token) -> bool { self == other }
    default fn into_token(self) -> I::Token { self }
}

pub trait Slice<I: Input>: Length {
    fn eq_slice(&self, other: &I::Slice) -> bool;
    fn into_slice(self) -> I::Slice;
}

impl<I: Input> Slice<I> for I::Slice {
    default fn eq_slice(&self, other: &I::Slice) -> bool { self == other }
    default fn into_slice(self) -> I::Slice { self }
}

#[derive(Debug, Copy, Clone)]
pub struct ParserInfo {
    pub name: &'static str,
    pub raw: bool,
}

pub trait Input: Sized {
    type Token: PartialEq + Token<Self>;
    type Slice: PartialEq + Length + Slice<Self>;
    type Many: Length;

    type Marker;
    type Context: Display;

    /// Returns a copy of the current token, if there is one.
    fn token(&mut self) -> Option<Self::Token>;

    /// Returns a copy of the current slice of size `n`, if there is one.
    fn slice(&mut self, n: usize) -> Option<Self::Slice>;

    /// Checks if the current token fulfills `cond`.
    fn peek<F>(&mut self, cond: F) -> bool
        where F: FnMut(&Self::Token) -> bool;

    /// Checks if the current slice of size `n` (if any) fulfills `cond`.
    fn peek_slice<F>(&mut self, n: usize, cond: F) -> bool
        where F: FnMut(&Self::Slice) -> bool;

    /// Checks if the current token fulfills `cond`. If so, the token is
    /// consumed and returned. Otherwise, returns `None`.
    fn eat<F>(&mut self, cond: F) -> Option<Self::Token>
        where F: FnMut(&Self::Token) -> bool;

    /// Checks if the current slice of size `n` (if any) fulfills `cond`. If so,
    /// the slice is consumed and returned. Otherwise, returns `None`.
    fn eat_slice<F>(&mut self, n: usize, cond: F) -> Option<Self::Slice>
        where F: FnMut(&Self::Slice) -> bool;

    /// Takes tokens while `cond` returns true, collecting them into a
    /// `Self::Many` and returning it.
    fn take<F>(&mut self, cond: F) -> Self::Many
        where F: FnMut(&Self::Token) -> bool;

    /// Skips tokens while `cond` returns true. Returns the number of skipped
    /// tokens.
    fn skip<F>(&mut self, cond: F) -> usize
        where F: FnMut(&Self::Token) -> bool;

    /// Returns `true` if there are no more tokens.
    fn is_eof(&mut self) -> bool;

    fn mark(&mut self, _info: &ParserInfo) -> Option<Self::Marker> {
        None
    }

    /// Optionally returns a context to identify the current input position. By
    /// default, this method returns `None`, indicating that no context could be
    /// resolved.
    fn context(&mut self, _mark: Option<&Self::Marker>) -> Option<Self::Context> {
        None
    }

    fn unmark(&mut self, _info: &ParserInfo, _success: bool, _mark: Option<Self::Marker>) { }
}

