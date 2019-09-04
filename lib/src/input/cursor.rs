use std::fmt::Debug;

use crate::input::{Input, Show, Rewind, ParserInfo};

pub struct Cursor<'a, T> {
    pub start: &'a [T],
    pub items: &'a [T],
}

impl<'a, T> From<&'a [T]> for Cursor<'a, T> {
    fn from(items: &'a [T]) -> Self {
        Cursor { start: items, items }
    }
}

impl<'a, T: PartialEq + Show> Rewind for Cursor<'a, T> {
    fn rewind_to(&mut self, marker: &Self::Marker) {
        self.items = &self.start[*marker..];
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Extent<'a, T> {
    pub start: usize,
    pub end: usize,
    pub values: &'a [T],
}

impl<T: Show> Show for Extent<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{} {}", self.start, self.end, &self.values as &dyn Show)
    }
}

// ident_impl_token!([T: PartialEq + Show] Cursor<'_, T>);
// ident_impl_slice!([T: PartialEq + Show] Cursor<'_, T>);

impl<'a, T: PartialEq + Show> Input for Cursor<'a, T> {
    type Token = &'a T;
    type Slice = &'a [T];
    type Many = &'a [T];

    type Marker = usize;
    type Context = Extent<'a, T>;

    /// Returns a copy of the current token, if there is one.
    fn token(&mut self) -> Option<Self::Token> {
        self.items.get(0)
    }

    /// Returns a copy of the current slice of size `n`, if there is one.
    fn slice(&mut self, n: usize) -> Option<Self::Slice> {
        self.items.get(..n)
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
            self.items = &self.items[1..];
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
            self.items = &self.items[slice.len()..];
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
        let matches = self.items.iter()
            .skip_while(cond)
            .count();

        let value = &self.items[..matches];
        self.items = &self.items[matches..];
        value
    }

    /// Skips tokens while `cond` returns true. Returns the number of skipped
    /// tokens.
    fn skip<F>(&mut self, cond: F) -> usize
        where F: FnMut(&Self::Token) -> bool
    {
        self.take(cond).len()
    }

    /// Returns `true` if there are no more tokens.
    fn is_eof(&mut self) -> bool {
        self.items.is_empty()
    }

    fn mark(&mut self, _: &ParserInfo) -> Self::Marker {
        self.start.len() - self.items.len()
    }

    /// Optionally returns a context to identify the current input position. By
    /// default, this method returns `None`, indicating that no context could be
    /// resolved.
    fn context(&mut self, mark: &Self::Marker) -> Option<Self::Context> {
        let end = self.start.len() - self.items.len();
        let values = &self.start[*mark..end];
        Some(Extent { start: *mark, end, values })
    }
}
