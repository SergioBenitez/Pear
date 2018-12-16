use std::fmt::{self, Display};

pub trait Length {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool { self.len() == 0 }
}

impl Length for str {
    #[inline(always)]
    fn len(&self) -> usize {
        str::len(self)
    }
}

impl<'a, T> Length for &'a [T] {
    #[inline(always)]
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

macro_rules! impl_length_for_sized_slice {
    ($($size:expr),*) => ($(
        impl<'a, T> Length for &'a [T; $size] {
            #[inline(always)] fn len(&self) -> usize { $size }
        }
    )*)
}

impl_length_for_sized_slice! {
     0,  1,  2,  3,  4,  5,  6,  7,  8,  9,
    10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
    20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
    30, 31, 32
}

impl<T> Length for [T] {
    #[inline(always)]
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T> Length for Vec<T> {
    #[inline(always)]
    fn len(&self) -> usize {
        <Vec<T>>::len(self)
    }
}

impl<'a> Length for &'a str {
    #[inline(always)]
    fn len(&self) -> usize {
        str::len(self)
    }
}

impl Length for String {
    #[inline(always)]
    fn len(&self) -> usize {
        String::len(self)
    }
}

pub trait Token<I: Input> {
    fn eq_token(&self, other: &I::Token) -> bool;
    fn into_token(self) -> I::Token;
}

pub trait Slice<I: Input>: Length {
    fn eq_slice(&self, other: &I::Slice) -> bool;
    fn into_slice(self) -> I::Slice;
}

impl<I: Input> Token<I> for I::Token {
    default fn eq_token(&self, other: &I::Token) -> bool { self == other }
    default fn into_token(self) -> I::Token { self }
}

impl<I: Input> Slice<I> for I::Slice {
    default fn eq_slice(&self, other: &I::Slice) -> bool { self == other }
    default fn into_slice(self) -> I::Slice { self }
}

pub trait Input: Sized {
    type Token: PartialEq;
    type Slice: PartialEq + Length;
    type Many: Length;
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

    #[inline(always)]
    fn context(&mut self) -> Option<Self::Context> {
        None
    }
}

impl<'a, 'b: 'a> Slice<&'a str> for &'b str {
    default fn eq_slice(&self, other: &&str) -> bool { self == other }
    default fn into_slice(self) -> &'a str { self }
}

impl<'a> Input for &'a str {
    type Token = char;
    type Slice = &'a str;
    type Many = Self::Slice;
    type Context = String;

    /// Returns a copy of the current token, if there is one.
    fn token(&mut self) -> Option<Self::Token> {
        self.chars().next()
    }

    /// Returns a copy of the current slice of size `n`, if there is one.
    fn slice(&mut self, n: usize) -> Option<Self::Slice> {
        if self.len() < n {
            None
        } else {
            Some(&self[..n])
        }
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
        if let Some(token) = self.token() {
            if cond(&token) {
                *self = &self[token.len_utf8()..];
                return Some(token)
            }
        }

        None
    }

    /// Checks if the current slice of size `n` (if any) fulfills `cond`. If so,
    /// the slice is consumed and returned. Otherwise, returns `None`.
    fn eat_slice<F>(&mut self, n: usize, mut cond: F) -> Option<Self::Slice>
        where F: FnMut(&Self::Slice) -> bool
    {
        if let Some(slice) = self.slice(n) {
            if cond(&slice) {
                *self = &self[slice.len()..];
                return Some(slice)
            }
        }

        None
    }

    /// Takes tokens while `cond` returns true, collecting them into a
    /// `Self::Many` and returning it.
    fn take<F>(&mut self, mut cond: F) -> Self::Many
        where F: FnMut(&Self::Token) -> bool
    {
        let mut consumed = 0;
        for c in self.chars() {
            if !cond(&c) { break; }
            consumed += c.len_utf8();
        }

        let value = &self[..consumed];
        *self = &self[consumed..];
        value
    }

    /// Skips tokens while `cond` returns true. Returns the number of skipped
    /// tokens.
    fn skip<F>(&mut self, mut cond: F) -> usize
        where F: FnMut(&Self::Token) -> bool
    {
        let mut skipped = 0;
        match self.take(|c| {  skipped += 1; cond(c) }) {
            "" => 0,
            _ => skipped - 1
        }
    }

    /// Returns `true` if there are no more tokens.
    fn is_eof(&mut self) -> bool {
        self.is_empty()
    }

    #[inline(always)]
    fn context(&mut self) -> Option<Self::Context> {
        match ::std::cmp::min(self.len(), 5) {
            0 => None,
            n => Some(format!("{:?}", &self[..n]))
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Position<'a> {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
    pub snippet: Option<&'a str>
}

impl<'a> Display for Position<'a> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const LIMIT: usize = 7;

        if is_pear_debug!() {
            write!(f, "{}:{}", self.line, self.column)?;

            if let Some(snippet) = self.snippet {
                if snippet.len() > LIMIT {
                    write!(f, " ({:?}..)", &snippet[..LIMIT])?;
                } else if !snippet.is_empty() {
                    write!(f, " ({:?})", snippet)?;
                }
            }

            Ok(())
        } else {
            write!(f, "line: {}, column: {}", self.line, self.column)
        }
    }
}

#[derive(Debug)]
pub struct Text<'a> {
    current: &'a str,
    start: &'a str,
}

impl<'a> From<&'a str> for Text<'a> {
    #[inline(always)]
    fn from(start: &'a str) -> Text {
        Text { start: start, current: start }
    }
}

impl<'a, 'b: 'a> Slice<Text<'a>> for &'b str {
    default fn eq_slice(&self, other: &&str) -> bool { self == other }
    default fn into_slice(self) -> &'a str { self }
}

impl<'a> Input for Text<'a> {
    type Token = char;
    type Slice = &'a str;
    type Many = Self::Slice;
    type Context = Position<'a>;

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

    #[inline(always)]
    fn context(&mut self) -> Option<Self::Context> {
        let bytes_read = self.start.len() - self.current.len();
        let snippet = Some(&self.start[bytes_read..]);
        let pos = if bytes_read == 0 {
            Position { line: 1, column: 0, offset: 0, snippet }
        } else {
            let string_read = &self.start[..bytes_read];
            let (count, last_line) = string_read.lines().enumerate().last().unwrap();
            Position { line: count + 1, column: last_line.len(), offset: bytes_read, snippet }
        };

        Some(pos)
    }
}

// // use std::fs::File;
// // use std::io::{self, Read, BufReader};

// // use std::cmp::min;
// // use std::marker::PhantomData;

// // // Ideally, this would hold a `String` inside. But we need a lifetime parameter
// // // here so we can return an &'a str from `peek_slice`. The alternative is to
// // // give a lifetime to the `Input` trait and use it in the `peek_slice` method.
// // // But that lifetime will pollute everything. Finally, the _correct_ thing is
// // // for Rust to let us reference the lifetime of `self` in an associated type.
// // // That requires something like https://github.com/rust-lang/rfcs/pull/1598.
// // #[derive(Debug)]
// // pub struct StringFile<'s> {
// //     buffer: Vec<u8>,
// //     consumed: usize,
// //     pos: usize,
// //     reader: BufReader<File>,
// //     _string: PhantomData<&'s str>
// // }

// // impl<'s> StringFile<'s> {
// //     #[inline(always)]
// //     pub fn open(path: &str) -> io::Result<StringFile<'s>> {
// //         Ok(StringFile::new(File::open(path)?, 1024))
// //     }

// //     #[inline(always)]
// //     pub fn open_with_cap(path: &str, cap: usize) -> io::Result<StringFile<'s>> {
// //         Ok(StringFile::new(File::open(path)?, cap))
// //     }

// //     #[inline(always)]
// //     pub fn new(file: File, cap: usize) -> StringFile<'s> {
// //         StringFile {
// //             buffer: vec![0; cap],
// //             consumed: 0,
// //             pos: 0,
// //             reader: BufReader::new(file),
// //             _string: PhantomData
// //         }
// //     }

// //     #[inline(always)]
// //     pub fn available(&self) -> usize {
// //         self.pos - self.consumed
// //     }

// //     fn read_into_peek(&mut self, num: usize) -> io::Result<usize> {
// //         if self.available() >= num {
// //             return Ok(num);
// //         }

// //         let needed = num - self.available();
// //         let to_read = min(self.buffer.len() - self.pos, needed);
// //         let (i, j) = (self.pos, self.pos + to_read);
// //         let read = self.reader.read(&mut self.buffer[i..j])?;

// //         self.pos += read;
// //         Ok(self.available())
// //     }

// //     // Panics if at least `num` aren't available.
// //     #[inline(always)]
// //     fn peek_bytes(&self, num: usize) -> &[u8] {
// //         &self.buffer[self.consumed..(self.consumed + num)]
// //     }

// //     fn consume(&mut self, num: usize) {
// //         if self.pos < num {
// //             let left = (num - self.pos) as u64;
// //             self.consumed = 0;
// //             self.pos = 0;
// //             // TOOD: Probably don't ignore this?
// //             let _ = io::copy(&mut self.reader.by_ref().take(left), &mut io::sink());
// //         } else {
// //             self.consumed += num;
// //         }
// //     }

// //     #[inline]
// //     fn peek_char(&mut self) -> Option<char> {
// //         let available = match self.read_into_peek(4) {
// //             Ok(n) => n,
// //             Err(_) => return None
// //         };

// //         let bytes = self.peek_bytes(available);
// //         let string = match ::std::str::from_utf8(bytes) {
// //             Ok(string) => string,
// //             Err(e) => match ::std::str::from_utf8(&bytes[..e.valid_up_to()]) {
// //                 Ok(string) => string,
// //                 Err(_) => return None
// //             }
// //         };

// //         string.chars().next()
// //     }
// // }

// // impl<'s> Input for StringFile<'s> {
// //     type Token = char;
// //     type InSlice = &'s str;
// //     type Slice = &'s str;
// //     type Many = String;
// //     type Context = &'s str;

// //     // If we took Self::Token here, we'd know the length of the character.
// //     #[inline(always)]
// //     fn peek(&mut self) -> Option<Self::Token> {
// //         self.peek_char()
// //     }

// //     fn take_many<F: FnMut(&Self::Token) -> bool>(&mut self, mut cond: F) -> Self::Many {
// //         let mut result = String::new();
// //         while let Some(c) = self.peek_char() {
// //             if cond(&c) {
// //                 result.push(c);
// //                 self.consume(c.len_utf8());
// //             } else {
// //                 break;
// //             }
// //         }

// //         result
// //     }

// //     fn skip_many<F: FnMut(&Self::Token) -> bool>(&mut self, mut cond: F) -> usize {
// //         let mut taken = 0;
// //         while let Some(c) = self.peek_char() {
// //             if cond(&c) {
// //                 self.consume(c.len_utf8());
// //                 taken += 1;
// //             } else {
// //                 return taken;
// //             }
// //         }

// //         taken
// //     }

// //     fn peek_slice(&mut self, slice: Self::InSlice) -> Option<Self::Slice> {
// //         let available = match self.read_into_peek(slice.len()) {
// //             Ok(n) => n,
// //             Err(_) => return None
// //         };

// //         let bytes = self.peek_bytes(available);
// //         let string = match ::std::str::from_utf8(bytes) {
// //             Ok(string) => string,
// //             Err(e) => match ::std::str::from_utf8(&bytes[..e.valid_up_to()]) {
// //                 Ok(string) => string,
// //                 Err(_) => return None
// //             }
// //         };

// //         match string == slice {
// //             true => Some(slice),
// //             false => None
// //         }
// //     }

// //     #[inline(always)]
// //     fn advance(&mut self, count: usize) {
// //         self.consume(count);
// //     }

// //     #[inline(always)]
// //     fn is_empty(&mut self) -> bool {
// //         match self.read_into_peek(1) {
// //             Ok(0) | Err(_) => true,
// //             Ok(_) => false,
// //         }
// //     }
// // }

impl<'a> Input for &'a [u8] {
    type Token = u8;
    type Slice = &'a [u8];
    type Many = Self::Slice;
    type Context = String;

    /// Returns a copy of the current token, if there is one.
    fn token(&mut self) -> Option<Self::Token> {
        self.get(0).map(|&c| c)
    }

    /// Returns a copy of the current slice of size `n`, if there is one.
    fn slice(&mut self, n: usize) -> Option<Self::Slice> {
        if self.len() < n {
            None
        } else {
            Some(&self[..n])
        }
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
        if let Some(token) = self.token() {
            if cond(&token) {
                *self = &self[1..];
                return Some(token)
            }
        }

        None
    }

    /// Checks if the current slice of size `n` (if any) fulfills `cond`. If so,
    /// the slice is consumed and returned. Otherwise, returns `None`.
    fn eat_slice<F>(&mut self, n: usize, mut cond: F) -> Option<Self::Slice>
        where F: FnMut(&Self::Slice) -> bool
    {
        if let Some(slice) = self.slice(n) {
            if cond(&slice) {
                *self = &self[slice.len()..];
                return Some(slice)
            }
        }

        None
    }

    /// Takes tokens while `cond` returns true, collecting them into a
    /// `Self::Many` and returning it.
    fn take<F>(&mut self, mut cond: F) -> Self::Many
        where F: FnMut(&Self::Token) -> bool
    {
        let mut consumed = 0;
        for c in self.iter() {
            if !cond(c) { break; }
            consumed += 1;
        }

        let value = &self[..consumed];
        *self = &self[consumed..];
        value
    }

    /// Skips tokens while `cond` returns true. Returns the number of skipped
    /// tokens.
    fn skip<F>(&mut self, mut cond: F) -> usize
        where F: FnMut(&Self::Token) -> bool
    {
        let mut skipped = 0;
        match self.take(|c| {  skipped += 1; cond(c) }) {
            &[] => 0,
            _ => skipped - 1
        }
    }

    /// Returns `true` if there are no more tokens.
    fn is_eof(&mut self) -> bool {
        self.is_empty()
    }

    #[inline(always)]
    fn context(&mut self) -> Option<Self::Context> {
        let n = ::std::cmp::min(self.len(), 5);
        if n == 0 {
            return None;
        }

        let bytes = &self[..n];
        if let Ok(string) = ::std::str::from_utf8(bytes) {
            Some(format!("{:?}", string))
        } else {
            Some(format!("{:?}", bytes))
        }
    }
}
