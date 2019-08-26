pub use crate::input::{Input, Token, Slice, ParserInfo};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Span<'a> {
    /// Start line/column/offset.
    pub start: (usize, usize, usize),
    /// End line/column/offset.
    pub end: (usize, usize, usize),
    /// Snippet between start and end.
    pub snippet: Option<&'a str>
}

impl<'a> std::fmt::Display for Span<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (a, b, _) = self.start;
        let (c, d, _) = self.end;

        if self.start == self.end {
            write!(f, "{}:{}", a, b)?;
        } else {
            write!(f, "{}:{} to {}:{}", a, b, c, d)?;
        }

        if let Some(snippet) = self.snippet {
            write!(f, " {:?}", snippet)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Text<'a> {
    current: &'a str,
    start: &'a str,
}

impl<'a> From<&'a str> for Text<'a> {
    #[inline(always)]
    fn from(start: &'a str) -> Text<'a> {
        Text { start: start, current: start }
    }
}

impl<'a, 'b: 'a> Slice<Text<'a>> for &'b str {
    default fn eq_slice(&self, other: &&str) -> bool { self == other }
    default fn into_slice(self) -> &'a str { self }
}

impl Text<'_> {
    /// Rewind the input `n` bytes. If `n` is greater than the number of bytes
    /// that have been consumed, rewinds to the beginning of the input.
    pub fn rewind(&mut self, n: usize) {
        let consumed = self.start.len() - self.current.len();
        let to_rewind = std::cmp::min(consumed, n);
        self.current = &self.start[(consumed - to_rewind)..];
    }
}

impl<'a> Input for Text<'a> {
    type Token = char;
    type Slice = &'a str;
    type Many = Self::Slice;

    type Marker = usize;
    type Context = Span<'a>;

    /// Returns a copy of the current token, if there is one.
    fn token(&mut self) -> Option<Self::Token> {
        self.current.token()
    }

    /// Returns a copy of the current slice of size `n`, if there is one.
    fn slice(&mut self, n: usize) -> Option<Self::Slice> {
        self.current.slice(n)
    }

    /// Checks if the current token fulfills `cond`.
    fn peek<F>(&mut self, cond: F) -> bool
        where F: FnMut(&Self::Token) -> bool
    {
        self.current.peek(cond)
    }

    /// Checks if the current slice of size `n` (if any) fulfills `cond`.
    fn peek_slice<F>(&mut self, n: usize, cond: F) -> bool
        where F: FnMut(&Self::Slice) -> bool
    {
        self.current.peek_slice(n, cond)
    }

    /// Checks if the current token fulfills `cond`. If so, the token is
    /// consumed and returned. Otherwise, returns `None`.
    fn eat<F>(&mut self, cond: F) -> Option<Self::Token>
        where F: FnMut(&Self::Token) -> bool
    {
        self.current.eat(cond)
    }

    /// Checks if the current slice of size `n` (if any) fulfills `cond`. If so,
    /// the slice is consumed and returned. Otherwise, returns `None`.
    fn eat_slice<F>(&mut self, n: usize, cond: F) -> Option<Self::Slice>
        where F: FnMut(&Self::Slice) -> bool
    {
        self.current.eat_slice(n, cond)
    }

    /// Takes tokens while `cond` returns true, collecting them into a
    /// `Self::Many` and returning it.
    fn take<F>(&mut self, cond: F) -> Self::Many
        where F: FnMut(&Self::Token) -> bool
    {
        self.current.take(cond)
    }

    /// Skips tokens while `cond` returns true. Returns the number of skipped
    /// tokens.
    fn skip<F>(&mut self, cond: F) -> usize
        where F: FnMut(&Self::Token) -> bool
    {
        self.current.skip(cond)
    }

    /// Returns `true` if there are no more tokens.
    fn is_eof(&mut self) -> bool {
        self.current.is_eof()
    }

    fn mark(&mut self, _: &ParserInfo) -> Self::Marker {
        self.start.len() - self.current.len()
    }

    fn context(&mut self, mark: &Self::Marker) -> Option<Self::Context> {
        let bytes_read = self.start.len() - self.current.len();
        let pos = if bytes_read == 0 {
            Span { start: (1, 1, 0), end: (1, 1, 0), snippet: None }
        } else {
            let start_offset = *mark;
            let end_offset = bytes_read;

            let to_start_str = &self.start[..start_offset];
            let (start_line, start_col) = line_col(to_start_str);
            let start = (start_line, start_col, start_offset);

            let to_current_str = &self.start[..bytes_read];
            let (end_line, end_col) = line_col(to_current_str);
            let end = (end_line, end_col, bytes_read);

            let snippet = Some(&self.start[start_offset..end_offset]);
            Span { start, end, snippet }
        };

        Some(pos)
    }
}

fn line_col(string: &str) -> (usize, usize) {
    if string.is_empty() {
        return (1, 1);
    }

    let (line_count, last_line) = string.lines().enumerate().last().unwrap();
    if string.ends_with('\n') {
        (line_count + 2, 1)
    } else {
        (line_count + 1, last_line.len() + 1)
    }
}
