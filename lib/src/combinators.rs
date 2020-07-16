use std::default::Default;

use crate::input::{Input, Rewind, Token, Result};
use crate::macros::parser;
use crate::parsers::*;

pub trait Collection<A>: Default + Extend<A> {
    #[inline(always)]
    fn push(&mut self, item: A) {
        self.extend(Some(item))
    }
}

impl<A, T: Default + Extend<A>> Collection<A> for T {  }

/// Parses `p` until `p` fails, returning the last successful `p`.
#[parser(raw)]
pub fn last_of_many<I, O, P>(input: &mut I, mut p: P) -> Result<O, I>
    where I: Input, P: FnMut(&mut I) -> Result<O, I>
{
    loop {
        let output = p(input)?;
        if let Ok(_) = eof(input) {
            return Ok(output);
        }
    }
}

/// Skips all tokens that match `f` before and after a `p`, returning `p`.
#[parser(raw)]
pub fn surrounded<I, O, F, P>(input: &mut I, mut p: P, mut f: F) -> Result<O, I>
    where I: Input,
          F: FnMut(&I::Token) -> bool,
          P: FnMut(&mut I) -> Result<O, I>
{
    skip_while(input, &mut f)?;
    let output = p(input)?;
    skip_while(input, &mut f)?;
    Ok(output)
}

/// Parses as many `p` as possible until EOF is reached, collecting them into a
/// `C`. Fails if `p` every fails. `C` may be empty.
#[parser(raw)]
pub fn collect<C, I, O, P>(input: &mut I, mut p: P) -> Result<C, I>
    where C: Collection<O>, I: Input, P: FnMut(&mut I) -> Result<O, I>
{
    let mut collection = C::default();
    loop {
        if eof(input).is_ok() {
            return Ok(collection);
        }

        collection.push(p(input)?);
    }
}

/// Parses as many `p` as possible until EOF is reached, collecting them into a
/// `C`. Fails if `p` ever fails. `C` is not allowed to be empty.
#[parser(raw)]
pub fn collect_some<C, I, O, P>(input: &mut I, mut p: P) -> Result<C, I>
    where C: Collection<O>, I: Input, P: FnMut(&mut I) -> Result<O, I>
{
    let mut collection = C::default();
    loop {
        collection.push(p(input)?);
        if eof(input).is_ok() {
            return Ok(collection);
        }
    }
}

/// Parses as many `p` as possible until EOF is reached or `p` fails, collecting
/// them into a `C`. `C` may be empty.
#[parser(raw)]
pub fn try_collect<C, I, O, P>(input: &mut I, mut p: P) -> Result<C, I>
    where C: Collection<O>, I: Input + Rewind, P: FnMut(&mut I) -> Result<O, I>
{
    let mut collection = C::default();
    loop {
        if eof(input).is_ok() {
            return Ok(collection);
        }

        // FIXME: We should be able to call `parse_marker!` here.
        let start = input.mark(&crate::input::ParserInfo {
            name: "try_collect",
            raw: true
        });

        match p(input) {
            Ok(val) => collection.push(val),
            Err(_) => {
                input.rewind_to(&start);
                break;
            }
        }
    }

    Ok(collection)
}

/// Parses many `separator` delimited `p`s, the entire collection of which must
/// start with `start` and end with `end`. `item` Gramatically, this is:
///
/// START (item SEPERATOR)* END
#[parser(raw)]
pub fn delimited_collect<C, I, T, S, O, P>(
    input: &mut I,
    start: T,
    mut item: P,
    seperator: S,
    end: T,
) -> Result<C, I>
    where C: Collection<O>,
          I: Input,
          T: Token<I> + Clone,
          S: Into<Option<T>>,
          P: FnMut(&mut I) -> Result<O, I>,
{
    eat(input, start)?;

    let seperator = seperator.into();
    let mut collection = C::default();
    loop {
        if eat(input, end.clone()).is_ok() {
            break;
        }

        collection.push(item(input)?);

        if let Some(seperator) = seperator.clone() {
            if eat(input, seperator).is_err(){
                eat(input, end.clone())?;
                break;
            }
        }
    }

    Ok(collection)
}

/// Parses many `separator` delimited `p`s. Gramatically, this is:
///
/// item (SEPERATOR item)*
#[parser(raw)]
pub fn series<C, I, S, O, P>(
    input: &mut I,
    mut item: P,
    seperator: S,
) -> Result<C, I>
    where C: Collection<O>,
          I: Input,
          S: Token<I> + Clone,
          P: FnMut(&mut I) -> Result<O, I>,
{
    let mut collection = C::default();
    loop {
        collection.push(item(input)?);
        if eat(input, seperator.clone()).is_err() {
            break;
        }
    }

    Ok(collection)
}

/// Parses many `separator` delimited `p`s with an optional trailing separator.
/// Gramatically, this is:
///
/// item (SEPERATOR item)* SEPERATOR?
#[parser(raw)]
pub fn trailing_series<C, I, S, O, P>(
    input: &mut I,
    mut item: P,
    seperator: S,
) -> Result<C, I>
    where C: Collection<O>,
          I: Input,
          S: Token<I> + Clone,
          P: FnMut(&mut I) -> Result<O, I>,
{
    let mut collection = C::default();
    let mut have_some = false;
    loop {
        match item(input) {
            Ok(item) => collection.push(item),
            Err(e) => if have_some {
                break
            } else {
                return Err(e)
            }
        }

        if eat(input, seperator.clone()).is_err() {
            break;
        }

        have_some = true;
    }

    Ok(collection)
}

/// Parses many `separator` delimited `p`s that are collectively prefixed with
/// `prefix`. Gramatically, this is:
///
/// PREFIX (item SEPERATOR)*
#[parser(raw)]
pub fn prefixed_series<C, I, T, O, P>(
    input: &mut I,
    prefix: T,
    item: P,
    seperator: T,
) -> Result<C, I>
    where C: Collection<O>,
          I: Input,
          T: Token<I> + Clone,
          P: FnMut(&mut I) -> Result<O, I>,
{
    if eat(input, prefix).is_err() {
        return Ok(C::default());
    }

    series(input, item, seperator)
}
