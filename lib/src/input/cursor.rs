use std::fmt::Debug;

use crate::input::{Input, Show, Rewind, ParserInfo, Length};

#[derive(Debug)]
pub struct Cursor<T> {
    pub start: T,
    pub items: T,
}

impl<T: Copy> From<T> for Cursor<T> {
    fn from(items: T) -> Self {
        Cursor { start: items, items }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Extent<T> {
    pub start: usize,
    pub end: usize,
    pub values: T,
}

impl<T: Length> From<T> for Extent<T> {
    fn from(values: T) -> Self {
        Extent { start: 0, end: values.len(), values }
    }
}

impl<T> std::ops::Deref for Extent<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl<T: PartialEq> PartialEq<T> for Extent<T> {
    fn eq(&self, other: &T) -> bool {
        &self.values == other
    }
}

impl PartialEq<Extent<&str>> for &str {
    fn eq(&self, other: &Extent<&str>) -> bool {
        other == self
    }
}

impl<T: PartialEq> PartialEq<Extent<&[T]>> for &[T] {
    fn eq(&self, other: &Extent<&[T]>) -> bool {
        other == self
    }
}

impl<T> Length for Extent<T> {
    fn len(&self) -> usize {
        self.end - self.start
    }
}

impl<T: Show> Show for Extent<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{} {}", self.start, self.end, &self.values as &dyn Show)
    }
}

pub trait Indexable: Sized {
    type One;
    type Iter: Iterator<Item = Self::One>;

    fn head(&self) -> Option<Self::One>;
    fn tail(&self) -> Option<Self>;
    fn slice<R: std::ops::RangeBounds<usize>>(&self, range: R) -> Option<Self>;
    fn iter(&self) -> Self::Iter;
}

use std::ops::{Bound, RangeBounds, RangeInclusive};

fn abs<R: RangeBounds<usize>>(range: R, start: usize, end: usize) -> RangeInclusive<usize> {
    let start = match range.start_bound() {
        Bound::Unbounded => start,
        Bound::Included(&n) => n,
        Bound::Excluded(&n) => n.saturating_add(1),
    };

    let end = match range.end_bound() {
        Bound::Unbounded => end.saturating_sub(1),
        Bound::Included(&n) => n,
        Bound::Excluded(&n) => n.saturating_sub(1),
    };

    RangeInclusive::new(start, end)
}

impl<'a> Indexable for &'a str {
    type One = char;
    type Iter = std::str::Chars<'a>;

    fn head(&self) -> Option<Self::One> {
        self.chars().next()
    }

    fn tail(&self) -> Option<Self> {
        self.get(self.head()?.len_utf8()..)
    }

    fn slice<R: std::ops::RangeBounds<usize>>(&self, range: R) -> Option<Self> {
        self.get(abs(range, 0, self.len()))
    }

    fn iter(&self) -> Self::Iter {
        self.chars()
    }
}

impl<'a, T: Clone> Indexable for &'a [T] {
    type One = T;
    type Iter = std::iter::Cloned<std::slice::Iter<'a, T>>;

    fn head(&self) -> Option<Self::One> {
        self.get(0).cloned()
    }

    fn tail(&self) -> Option<Self> {
        self.get(1..)
    }

    fn slice<R: std::ops::RangeBounds<usize>>(&self, range: R) -> Option<Self> {
        self.get(abs(range, 0, self.len()))
    }

    fn iter(&self) -> Self::Iter {
        (self as &'a [T]).iter().cloned()
    }
}

impl<T: Length> Cursor<T> {
    fn offset(&self) -> usize {
        self.start.len() - self.items.len()
    }
}

impl<T: Indexable + Show + Length + PartialEq> Input for Cursor<T>
    where T::One: Show + PartialEq
{
    type Token = T::One;
    type Slice = Extent<T>;
    type Many = Extent<T>;

    type Marker = usize;
    type Context = Extent<T>;

    /// Returns a copy of the current token, if there is one.
    fn token(&mut self) -> Option<Self::Token> {
        self.items.head()
    }

    /// Returns a copy of the current slice of size `n`, if there is one.
    fn slice(&mut self, n: usize) -> Option<Self::Slice> {
        Some(Extent {
            start: self.offset(),
            end: self.offset() + n,
            values: self.items.slice(..n)?
        })
    }

    /// Checks if the current token fulfills `cond`.
    fn peek<F>(&mut self, mut cond: F) -> bool
        where F: FnMut(&Self::Token) -> bool
    {
        self.token().map(|t| cond(&t)).unwrap_or(false)
    }

    /// Checks if the current slice of size `n` (if any) fulfills `cond`.
    fn peek_slice<F>(&mut self, n: usize, mut cond: F) -> bool
        where F: FnMut(&Self::Slice) -> bool
    {
        self.slice(n).map(|s| cond(&s)).unwrap_or(false)
    }

    /// Checks if the current token fulfills `cond`. If so, the token is
    /// consumed and returned. Otherwise, returns `None`.
    fn eat<F>(&mut self, mut cond: F) -> Option<Self::Token>
        where F: FnMut(&Self::Token) -> bool
    {
        let token = self.token()?;
        if cond(&token) {
            self.items = self.items.tail().unwrap();
            Some(token)
        } else {
            None
        }
    }

    /// Checks if the current slice of size `n` (if any) fulfills `cond`. If so,
    /// the slice is consumed and returned. Otherwise, returns `None`.
    fn eat_slice<F>(&mut self, n: usize, mut cond: F) -> Option<Self::Slice>
        where F: FnMut(&Self::Slice) -> bool
    {
        let slice = self.slice(n)?;
        if cond(&slice) {
            self.items = self.items.slice(n..).unwrap();
            Some(slice)
        } else {
            None
        }
    }

    /// Takes tokens while `cond` returns true, collecting them into a
    /// `Self::Many` and returning it.
    fn take<F>(&mut self, cond: F) -> Self::Many
        where F: FnMut(&Self::Token) -> bool
    {
        let start = self.offset();
        let matches = self.items.iter()
            .take_while(cond)
            .count();

        let values = self.items.slice(..matches).unwrap();
        self.items = self.items.slice(matches..).unwrap();
        Extent { start, end: self.offset(), values }
    }

    /// Skips tokens while `cond` returns true. Returns the number of skipped
    /// tokens.
    fn skip<F>(&mut self, cond: F) -> usize
        where F: FnMut(&Self::Token) -> bool
    {
        self.take(cond).len()
    }

    /// Returns `true` if there are at least `n` tokens remaining.
    fn has(&mut self, n: usize) -> bool {
        self.items.len() >= n
    }

    fn mark(&mut self, _: &ParserInfo) -> Self::Marker {
        self.offset()
    }

    /// Optionally returns a context to identify the current input position. By
    /// default, this method returns `None`, indicating that no context could be
    /// resolved.
    fn context(&mut self, mark: &Self::Marker) -> Option<Self::Context> {
        let end = self.offset();
        let values = self.start.slice(*mark..end).unwrap();
        Some(Extent { start: *mark, end, values })
    }
}

impl<T: Indexable + Show + Length + PartialEq> Rewind for Cursor<T>
    where T::One: Show + PartialEq
{
    fn rewind_to(&mut self, marker: &Self::Marker) {
        self.items = self.start.slice(*marker..).unwrap();
    }
}
