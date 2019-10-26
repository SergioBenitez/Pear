use crate::error::{ParseError, Expected};
use crate::input::{Input, Length, Token, Slice, Show, Result};
use crate::macros::parser;

// // TODO:
// // * provide basic parsers in pear
// //   - [f32, f64, i8, i32, ..., bool, etc.]: one for all reasonable built-ins
// //   - quoted_string(allowed): '"' allowed* '"'
// //   - escaped string, with some way to configure escapes

#[inline(always)]
fn expected_token<T, A, I>(
    input: &mut I,
    token: Option<T>
) -> Result<A, I>
    where T: Token<I>, I: Input
{
    // FIXME(show)
    // let expected = Expected::Token(token.map(|t| t.into_token()), input.token());
    // Err(ParseError::expected(expected))

    let string = token.map(|t| (&t as &dyn Show).to_string());
    let expected = Expected::Token(string, input.token());
    Err(ParseError::new(expected))
}

#[inline(always)]
fn expected_slice<S, A, I>(
    input: &mut I,
    slice: S
) -> Result<A, I>
    where S: Slice<I>, I: Input
{
    // FIXME(show)
    // let len = slice.len();
    // let expected = Expected::Slice(Some(slice.into_slice()), input.slice(len));
    // Err(ParseError::expected(expected))

    let string = (&slice as &dyn Show).to_string();
    let expected = Expected::Slice(Some(string), input.slice(slice.len()));
    Err(ParseError::new(expected))
}

/// Eats the current token if it is `token`.
#[parser(raw)]
pub fn eat<I, T>(input: &mut I, token: T) -> Result<I::Token, I>
    where I: Input, T: Token<I>
{
    match input.eat(|t| &token == t) {
        Some(token) => Ok(token),
        None => expected_token(input, Some(token))
    }
}

/// Eats the token `token` if `cond` holds on the current token.
#[parser(raw)]
pub fn eat_if<I, F>(input: &mut I, cond: F) -> Result<I::Token, I>
    where I: Input, F: FnMut(&I::Token) -> bool
{
    match input.eat(cond) {
        Some(token) => Ok(token),
        None => expected_token::<I::Token, _, _>(input, None)
    }
}

/// Eats the current token unconditionally.
#[parser(raw)]
pub fn eat_any<I: Input>(input: &mut I) -> Result<I::Token, I> {
    match input.eat(|_| true) {
        Some(token) => Ok(token),
        None => Err(ParseError::new(Expected::Token(None, None)))
    }
}

/// Eats the current slice if it is `slice`.
#[parser(raw)]
pub fn eat_slice<I, S>(input: &mut I, slice: S) -> Result<I::Slice, I>
    where I: Input, S: Slice<I>
{
    match input.eat_slice(slice.len(), |s| &slice == s) {
        Some(slice) => Ok(slice),
        None => expected_slice(input, slice)
    }
}

/// Succeeds if the current token is `token`.
#[parser(raw)]
pub fn peek<I, T>(input: &mut I, token: T) -> Result<(), I>
    where I: Input, T: Token<I>
{
    match input.peek(|t| &token == t) {
        true => Ok(()),
        false => expected_token(input, Some(token))
    }
}

/// Succeeds if `cond` holds for the current token.
#[parser(raw)]
pub fn peek_if_copy<I, F>(input: &mut I, cond: F) -> Result<I::Token, I>
    where I: Input, F: FnMut(&I::Token) -> bool
{
    match input.peek(cond) {
        true => Ok(input.token().unwrap()),
        false => expected_token::<I::Token, _, _>(input, None)
    }
}

/// Succeeds if `cond` holds for the current token.
#[parser(raw)]
pub fn peek_if<I, F>(input: &mut I, cond: F) -> Result<(), I>
    where I: Input, F: FnMut(&I::Token) -> bool
{
    match input.peek(cond) {
        true => Ok(()),
        false => expected_token::<I::Token, _, _>(input, None)
    }
}

/// Succeeds if the current slice is `slice`.
#[parser(raw)]
pub fn peek_slice<I, S>(input: &mut I, slice: S) -> Result<(), I>
    where I: Input, S: Slice<I>
{
    match input.peek_slice(slice.len(), |s| &slice == s) {
        true => Ok(()),
        false => expected_slice(input, slice)
    }
}

/// Returns the current token.
#[parser(raw)]
pub fn peek_any<I: Input>(input: &mut I) -> Result<I::Token, I> {
    match input.token() {
        Some(peeked) => Ok(peeked),
        None => Err(ParseError::new(Expected::Token(None, None)))
    }
}

/// Skips tokens while `cond` matches.
#[parser(raw)]
pub fn skip_while<I, F>(input: &mut I, cond: F) -> Result<usize, I>
    where I: Input, F: FnMut(&I::Token) -> bool
{
    Ok(input.skip(cond))
}

/// Consumes tokens while `cond` matches and returns them. Succeeds even if no
/// tokens match.
#[parser(raw)]
pub fn take_while<I, F>(input: &mut I, cond: F) -> Result<I::Many, I>
    where I: Input, F: FnMut(&I::Token) -> bool
{
    Ok(input.take(cond))
}

/// Consumes tokens while `cond` matches and returns them. Succeeds only if at
/// least one token matched `cond`.
#[parser(raw)]
pub fn take_some_while<I, F>(input: &mut I, cond: F) -> Result<I::Many, I>
    where I: Input, F: FnMut(&I::Token) -> bool
{
    let value = input.take(cond);
    if value.len() == 0 {
        return Err(ParseError::new(Expected::Token(None, None)));
    }

    Ok(value)
}

/// Consumes tokens while `cond` matches and the token is not `until`. Succeeds
/// even if no tokens match.
#[parser(raw)]
pub fn take_while_until<I, T, F>(
    input: &mut I,
    mut cond: F,
    until: T,
) -> Result<I::Many, I>
    where I: Input,
          T: Token<I>,
          F: FnMut(&I::Token) -> bool
{
    take_while(input, |t| cond(t) && (&until != t))
}

/// Consumes tokens while `cond` matches and the token is not `until`. Succeeds
/// only if at least one token matched `cond`.
#[parser(raw)]
pub fn take_some_while_until<I, T, F>(
    input: &mut I,
    mut cond: F,
    until: T,
) -> Result<I::Many, I>
    where I: Input,
          T: Token<I>,
          F: FnMut(&I::Token) -> bool
{
    take_some_while(input, |t| cond(t) && (&until != t))
}

/// Takes at most `n` tokens.
#[parser(raw)]
pub fn take_n<I: Input>(input: &mut I, n: usize) -> Result<I::Many, I> {
    let mut i = 0;
    Ok(input.take(|_| { let c = i < n; i += 1; c }))
}

/// Takes at most `n` tokens as long as `cond` holds.
#[parser(raw)]
pub fn take_n_while<I, F>(input: &mut I, n: usize, mut cond: F) -> Result<I::Many, I>
    where I: Input, F: FnMut(&I::Token) -> bool
{
    let mut i = 0;
    Ok(input.take(|c| { cond(c) && { let ok = i < n; i += 1; ok } }))
}

/// Take exactly `n` tokens, ensuring `cond` holds on all `n`.
#[parser(raw)]
pub fn take_n_if<I, F>(input: &mut I, n: usize, mut cond: F) -> Result<I::Many, I>
    where I: Input, F: FnMut(&I::Token) -> bool
{
    let mut i = 0;
    let v = input.take(|c| { cond(c) && { let ok = i < n; i += 1; ok } });
    if v.len() != n {
        return Err(ParseError::new(Expected::Token(None, None)));
    }

    Ok(v)
}

/// Parse a token stream that starts with `start` and ends with `end`, returning
/// all of the tokens in between. The tokens in between must match `cond`.
/// Succeeds even if there are no tokens between `start` and `end`.
#[parser(raw)]
pub fn delimited<I, T, F>(
    input: &mut I,
    start: T,
    mut cond: F,
    end: T,
) -> Result<I::Many, I>
    where I: Input,
          T: Token<I>,
          F: FnMut(&I::Token) -> bool
{
    eat(input, start)?;
    let output = input.take(|t| cond(t) && (&end != t));
    eat(input, end)?;
    Ok(output)
}

/// Parse a token stream that starts with `start` and ends with `end`, returning
/// all of the tokens in between. The tokens in between must match `cond`. There
/// must be at least one token between `start` and `end`.
#[parser(raw)]
pub fn delimited_some<I, T, F>(
    input: &mut I,
    start: T,
    mut cond: F,
    end: T,
) -> Result<I::Many, I>
    where I: Input,
          T: Token<I>,
          F: FnMut(&I::Token) -> bool
{
    eat(input, start)?;
    let output = take_some_while(input, |t| cond(t) && (&end != t))?;
    eat(input, end)?;
    Ok(output)
}

/// Succeeds only if the input has reached EOF.
#[parser(raw)]
pub fn eof<I: Input>(input: &mut I) -> Result<(), I> {
    if input.is_eof() {
        Ok(())
    } else {
        let next = input.token();
        Err(ParseError::new(Expected::Eof(next)))
    }
}

// // Like delimited, but keeps the start and end tokens.
// #[parser(raw)]
// pub fn enclosed<I: Input, F>(
//     input: &mut I,
//     start: I::Token,
//     mut cond: F,
//     end: I::Token
// ) -> Result<I::Many, I>
//     where F: FnMut(&I::Token) -> bool
// {
//     let mut phase = 0;
//     let output = take_some_while(input, |c| {
//         match phase {
//             0 => {
//                 phase = 1;
//                 c == &start
//             }
//             1 => if cond(c) {
//                 true
//             } else if c == &end {
//                 phase = 2;
//                 true
//             } else {
//                 false
//             }
//             _ => false
//         }
//     });

//     match phase {
//         0 => error(input, "enclosed", Expected::Token(Some(start), None)),
//         1 => error(input, "enclosed", Expected::Token(Some(end), None)),
//         _ => output
//     }
// }

