use std::hash::Hash;
use std::collections::{HashMap, BTreeMap};

use {Result, Input, Token};
use parsers::*;

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

/// Parses `p` until `p` fails, returning the last successful `p`.
#[raw_parser]
pub fn last_of_many<I, O, P>(input: &mut I, p: P) -> Result<O, I>
    where I: Input, P: Fn(&mut I) -> Result<O, I>
{
    loop {
        let output = p(input)?;
        if let Ok(_) = eof(input) {
            return Ok(output);
        }
    }
}

/// Skips all tokens that match `f` before and after a `p`, returning `p`.
#[raw_parser]
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
/// `C`. `C` may be empty.
#[raw_parser]
pub fn collect<C, I, O, P>(input: &mut I, mut p: P) -> Result<C, I>
    where C: Collection<Item=O>, I: Input, P: FnMut(&mut I) -> Result<O, I>
{
    let mut collection = C::new();
    loop {
        if eof(input).is_ok() {
            return Ok(collection);
        }

        collection.add(p(input)?);
    }
}

/// Parses as many `p` as possible until EOF is reached, collecting them into a
/// `C`. `C` is not allowed to be empty.
#[raw_parser]
pub fn collect_some<C, I, O, P>(input: &mut I, mut p: P) -> Result<C, I>
    where C: Collection<Item=O>, I: Input, P: FnMut(&mut I) -> Result<O, I>
{
    let mut collection = C::new();
    loop {
        collection.add(p(input)?);
        if eof(input).is_ok() {
            return Ok(collection);
        }
    }
}

/// Parses many `separator` delimited `p`s, the entire collection of which must
/// start with `start` and end with `end`. `item` Gramatically, this is:
///
/// START (item SEPERATOR)* END
#[raw_parser]
pub fn delimited_collect<C, I, T, S, O, P>(
    input: &mut I,
    start: T,
    mut item: P,
    seperator: S,
    end: T,
) -> Result<C, I>
    where C: Collection<Item=O>,
          I: Input,
          T: Token<I> + Clone,
          S: Into<Option<T>>,
          P: FnMut(&mut I) -> Result<O, I>,
{
    eat(input, start)?;

    let seperator = seperator.into();
    let mut collection = C::new();
    loop {
        if eat(input, end.clone()).is_ok() {
            break;
        }

        collection.add(item(input)?);

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
/// (item SEPERATOR)+
#[raw_parser]
pub fn series<C, I, S, O, P>(
    input: &mut I,
    mut item: P,
    seperator: S,
) -> Result<C, I>
    where C: Collection<Item=O>,
          I: Input,
          S: Token<I> + Clone,
          P: FnMut(&mut I) -> Result<O, I>,
{
    let mut collection = C::new();
    loop {
        collection.add(item(input)?);
        if eat(input, seperator.clone()).is_err() {
            break;
        }
    }

    Ok(collection)
}

/// Parses many `separator` delimited `p`s that are collectively prefixed with
/// `prefix`. Gramatically, this is:
///
/// PREFIX (item SEPERATOR)*
#[raw_parser]
pub fn prefixed_series<C, I, T, O, P>(
    input: &mut I,
    prefix: T,
    item: P,
    seperator: T,
) -> Result<C, I>
    where C: Collection<Item=O>,
          I: Input,
          T: Token<I> + Clone,
          P: FnMut(&mut I) -> Result<O, I>,
{
    if eat(input, prefix).is_err() {
        return Ok(C::new());
    }

    series(input, item, seperator)
}
