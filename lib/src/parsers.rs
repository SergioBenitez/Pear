use {Input, ParseResult};
use ParseResult::*;

use num_traits::{Zero, One};

#[inline]
pub fn eat<I: Input>(input: I, token: I::Token) -> ParseResult<I, I::Token> {
    match input.get(I::Index::zero()) {
        Some(t) if t == token => Done(input.slice_from(I::Index::one()), token),
        Some(_) => Error("eat: Found other token."),
        None => Error("eat: No tokens to eat.")
    }
}
