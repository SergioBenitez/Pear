use {Result, Input, Length, Expected, ParseErr, switch};

// TODO:
// * provide basic parsers in pear
//   - [f32, f64, i8, i32, ..., bool, etc.]: one for all reasonable built-ins
//   - quoted_string(allowed): '"' allowed* '"'
//   - escaped string, with some way to configure escapes

#[inline(always)]
pub fn error<I: Input, R>(
    input: &mut I,
    parser: &'static str,
    expected: Expected<I::Token, I::InSlice, I::Slice>
) -> Result<R, I> {
    Err(ParseErr { parser, expected, context: input.context() })
}

#[inline(always)]
fn advance_and<I: Input, T>(input: &mut I, num: usize, out: T) -> Result<T, I> {
    input.advance(num);
    Ok(out)
}

#[inline]
pub fn eat<I: Input>(input: &mut I, token: I::Token) -> Result<I::Token, I> {
    match input.peek() {
        Some(peeked) if peeked == token => advance_and(input, 1, token),
        t@Some(_) | t@None => error(input, "eat", Expected::Token(Some(token), t)),
    }
}

#[inline]
pub fn eat_if<I: Input, F>(input: &mut I, cond: F) -> Result<I::Token, I>
    where F: Fn(I::Token) -> bool
{
    match input.peek() {
        Some(peeked) if cond(peeked) => advance_and(input, 1, peeked),
        t@Some(_) | t@None => error(input, "eat_if", Expected::Token(None, t)),
    }
}

#[inline]
pub fn eat_slice<I: Input>(input: &mut I, slice: I::InSlice) -> Result<I::Slice, I> {
    let len = slice.len();
    match input.peek_slice(slice.clone()) {
        Some(peeked) => advance_and(input, len, peeked),
        t@None => error(input, "eat_slice", Expected::Slice(Some(slice), t)),
    }
}

#[inline]
pub fn eat_any<I: Input>(input: &mut I) -> Result<I::Token, I> {
    match input.peek() {
        Some(peeked) => advance_and(input, 1, peeked),
        None => error(input, "eat_any", Expected::Token(None, None)),
    }
}

#[inline]
pub fn peek<I: Input>(input: &mut I, token: I::Token) -> Result<I::Token, I> {
    match input.peek() {
        Some(peeked) if peeked == token => Ok(token),
        t@Some(_) | t@None => error(input, "eat", Expected::Token(Some(token), t)),
    }
}

#[inline]
pub fn peek_if<I: Input, F>(input: &mut I, cond: F) -> Result<I::Token, I>
    where F: Fn(I::Token) -> bool
{
    match input.peek() {
        Some(peeked) if cond(peeked) => Ok(peeked),
        t@Some(_) | t@None => error(input, "peek_id", Expected::Token(None, t)),
    }
}

#[inline]
pub fn peek_slice<I: Input>(input: &mut I, slice: I::InSlice) -> Result<I::Slice, I> {
    match input.peek_slice(slice.clone()) {
        Some(peeked) => Ok(peeked),
        t@None => error(input, "peek_slice", Expected::Slice(Some(slice), t)),
    }
}

#[inline]
pub fn peek_any<I: Input>(input: &mut I) -> Result<I::Token, I> {
    match input.peek() {
        Some(peeked) => Ok(peeked),
        None => error(input, "peek_any", Expected::Token(None, None)),
    }
}

#[inline]
pub fn skip_while<I: Input, F>(input: &mut I, condition: F) -> Result<(), I>
    where F: FnMut(I::Token) -> bool
{
    input.skip_many(condition);
    Ok(())
}

#[inline]
pub fn take_some_while<I: Input, F>(input: &mut I, condition: F) -> Result<I::Many, I>
    where F: FnMut(I::Token) -> bool
{
    let value = input.take_many(condition);
    if value.len() == 0 {
        return error(input, "take_some_while", Expected::Token(None, None));
    }

    Ok(value)
}

#[inline(always)]
pub fn take_while<I: Input, F>(input: &mut I, condition: F) -> Result<I::Many, I>
    where F: FnMut(I::Token) -> bool
{
    Ok(input.take_many(condition))
}

#[inline(always)]
pub fn take_some_while_until<I: Input, F>(
    input: &mut I,
    mut condition: F,
    until: I::Token,
) -> Result<I::Many, I>
    where F: FnMut(I::Token) -> bool
{
    take_some_while(input, |c| condition(c) && c != until)
}

/// Takes at most `num` inputs.
#[inline(always)]
pub fn take_n<I: Input>(input: &mut I, num: usize) -> Result<I::Many, I> {
    let mut i = 0;
    Ok(input.take_many(|_| { let c = i < num; i += 1; c }))
}

/// Takes at most `num` inputs as long as `condition` holds.
#[inline(always)]
pub fn take_n_while<I: Input, F>(input: &mut I, num: usize, mut condition: F) -> Result<I::Many, I>
    where F: FnMut(I::Token) -> bool
{
    let mut i = 0;
    Ok(input.take_many(|c| { condition(c) && { let ok = i < num; i += 1; ok } }))
}

/// Take exactly `num` inputs, ensuring `condition` holds.
#[inline(always)]
pub fn take_n_if<I: Input, F>(input: &mut I, num: usize, mut condition: F) -> Result<I::Many, I>
    where F: FnMut(I::Token) -> bool
{
    let mut i = 0;
    let v = input.take_many(|c| { condition(c) && { let ok = i < num; i += 1; ok } });
    if v.len() != num {
        return error(input, "take_n", Expected::Token(None, None));
    }

    Ok(v)
}

#[inline]
pub fn delimited<I: Input, F>(
    input: &mut I,
    start: I::Token,
    mut cond: F,
    end: I::Token
) -> Result<I::Many, I>
    where F: FnMut(I::Token) -> bool
{
    if let Err(mut e) = eat(input, start) {
        e.parser = "delimited";
        return Err(e);
    }

    let output = match take_some_while(input, |c| c != end && cond(c)) {
        Ok(output) => output,
        Err(mut e) => {
            e.parser = "delimited";
            return Err(e);
        }
    };

    if let Err(mut e) = eat(input, end) {
        e.parser = "delimited";
        return Err(e);
    }

    Ok(output)
}

// Like delimited, but keeps the start and end tokens.
#[inline]
pub fn enclosed<I: Input, F>(
    input: &mut I,
    start: I::Token,
    mut cond: F,
    end: I::Token
) -> Result<I::Many, I>
    where F: FnMut(I::Token) -> bool
{
    let mut phase = 0;
    let output = take_some_while(input, |c| {
        match phase {
            0 => {
                phase = 1;
                c == start
            }
            1 => if cond(c) {
                true
            } else if c == end {
                phase = 2;
                true
            } else {
                false
            }
            _ => false
        }
    });

    match phase {
        0 => error(input, "enclosed", Expected::Token(Some(start), None)),
        1 => error(input, "enclosed", Expected::Token(Some(end), None)),
        _ => output
    }
}

#[inline(always)]
pub fn eof<I: Input>(input: &mut I) -> Result<(), I> {
    if input.is_empty() {
        Ok(())
    } else {
        let next = input.peek();
        error(input, "eof", Expected::EOF(next))
    }
}

pub trait Collection {
    type Item;
    fn new() -> Self;
    fn add(&mut self, item: Self::Item);
}

impl<T> Collection for Vec<T> {
    type Item = T;

    fn new() -> Self {
        vec![]
    }

    fn add(&mut self, item: Self::Item) {
        self.push(item);
    }
}

use std::hash::Hash;
use std::collections::HashMap;
use std::collections::BTreeMap;

impl<K: Eq + Hash, V> Collection for HashMap<K, V> {
    type Item = (K, V);

    fn new() -> Self {
        HashMap::new()
    }

    fn add(&mut self, item: Self::Item) {
        let (k, v) = item;
        self.insert(k, v);
    }
}

impl<K: Ord, V> Collection for BTreeMap<K, V> {
    type Item = (K, V);

    fn new() -> Self {
        BTreeMap::new()
    }

    fn add(&mut self, item: Self::Item) {
        let (k, v) = item;
        self.insert(k, v);
    }
}

pub fn collection<C: Collection<Item=O>, I: Input, O, F>(
    input: &mut I,
    start: I::Token,
    mut item: F,
    seperator: I::Token,
    end: I::Token,
) -> Result<C, I>
    where F: FnMut(&mut I) -> Result<O, I>,
{
    let mut collection = (eat(input, start)?, C::new()).1;
    loop {
        switch! { [collection; input]
            eat(end) => break,
            eat(seperator) => continue,
            _ => collection.add(item()?)
        }
    }

    Ok(collection)
}

#[inline]
pub fn series<C: Collection<Item=O>, I: Input, O, F, W>(
    input: &mut I,
    prefix_needed: bool,
    seperator: I::Token,
    whitespace: W,
    mut item: F,
) -> Result<C, I>
    where F: FnMut(&mut I) -> Result<O, I>,
          W: FnMut(I::Token) -> bool + Copy
{
    let mut collection = C::new();

    if prefix_needed {
        if ::combinators::surrounded(input, |i| eat(i, seperator), whitespace).is_err() {
            return Ok(collection);
        }
    }

    loop {
        skip_while(input, whitespace)?;
        collection.add(item(input)?);
        skip_while(input, whitespace)?;

        if !eat(input, seperator).is_ok() {
            break
        }
    }

    skip_while(input, whitespace)?;
    Ok(collection)
}
