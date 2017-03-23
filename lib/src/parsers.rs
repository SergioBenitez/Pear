use {Input, Length, ParseResult, Expected};
use result::error;
use ParseResult::*;

#[inline(always)]
fn advance_and<I: Input, T>(input: &mut I, num: usize, out: T) -> ParseResult<I, T> {
    input.advance(num);
    Done(out)
}

#[inline]
pub fn eat<I: Input>(input: &mut I, token: I::Token) -> ParseResult<I, I::Token> {
    match input.peek() {
        Some(peeked) if peeked == token => advance_and(input, 1, token),
        t@Some(_) | t@None => error("eat", Expected::Token(Some(token), t)),
    }
}

#[inline]
pub fn eat_if<I: Input, F>(input: &mut I, cond: F) -> ParseResult<I, I::Token>
    where F: Fn(I::Token) -> bool
{
    match input.peek() {
        Some(peeked) if cond(peeked) => advance_and(input, 1, peeked),
        t@Some(_) | t@None => error("eat_if", Expected::Token(None, t)),
    }
}

#[inline]
pub fn eat_slice<I: Input>(input: &mut I, slice: I::Slice) -> ParseResult<I, I::Slice> {
    let len = slice.len();
    match input.peek_slice(slice) {
        Some(peeked) if peeked == slice => advance_and(input, len, slice),
        t@Some(_) | t@None => error("eat_slice", Expected::Slice(Some(slice), t)),
    }
}

#[inline]
pub fn eat_any<I: Input>(input: &mut I) -> ParseResult<I, I::Token> {
    match input.peek() {
        Some(peeked) => advance_and(input, 1, peeked),
        None => error("eat_any", Expected::Token(None, None)),
    }
}

#[inline]
pub fn peek<I: Input>(input: &mut I, token: I::Token) -> ParseResult<I, I::Token> {
    match input.peek() {
        Some(peeked) if peeked == token => Done(token),
        t@Some(_) | t@None => error("eat", Expected::Token(Some(token), t)),
    }
}

#[inline]
pub fn peek_if<I: Input, F>(input: &mut I, cond: F) -> ParseResult<I, I::Token>
    where F: Fn(I::Token) -> bool
{
    match input.peek() {
        Some(peeked) if cond(peeked) => Done(peeked),
        t@Some(_) | t@None => error("peek_id", Expected::Token(None, t)),
    }
}

#[inline]
pub fn peek_slice<I: Input>(input: &mut I, slice: I::Slice) -> ParseResult<I, I::Slice> {
    match input.peek_slice(slice) {
        Some(peeked) if peeked == slice => Done(slice),
        t@Some(_) | t@None => error("peek_slice", Expected::Slice(Some(slice), t)),
    }
}

#[inline]
pub fn skip_while<I: Input, F>(input: &mut I, condition: F) -> ParseResult<I, ()>
    where F: FnMut(I::Token) -> bool
{
    input.skip_many(condition);
    Done(())
}

#[inline]
pub fn take_some_while<I: Input, F>(input: &mut I, condition: F) -> ParseResult<I, I::Many>
    where F: FnMut(I::Token) -> bool
{
    let value = input.take_many(condition);
    if value.len() == 0 {
        return error("take_some_while", Expected::Token(None, None));
    }

    Done(value)
}

#[inline(always)]
pub fn take_while<I: Input, F>(input: &mut I, condition: F) -> ParseResult<I, I::Many>
    where F: FnMut(I::Token) -> bool
{
    Done(input.take_many(condition))
}

#[inline]
pub fn delimited<I: Input, F>(input: &mut I,
                              start: I::Token,
                              mut cond: F,
                              end: I::Token) -> ParseResult<I, I::Many>
    where F: FnMut(I::Token) -> bool
{
    if let Error(mut e) = eat(input, start) {
        e.parser = "delimited";
        return Error(e);
    }

    let output = match take_some_while(input, |c| c != end && cond(c)) {
        Done(output) => output,
        Error(mut e) => {
            e.parser = "delimited";
            return Error(e);
        }
    };

    if let Error(mut e) = eat(input, end) {
        e.parser = "delimited";
        return Error(e);
    }

    Done(output)
}

#[inline(always)]
pub fn eof<I: Input>(input: &mut I) -> ParseResult<I, ()> {
    if input.is_empty() {
        Done(())
    } else {
        error("eof", Expected::EOF)
    }
}
