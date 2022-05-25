use std::fmt::{self, Display};

pub trait Length {
    fn len(&self) -> usize;
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

pub trait Input: Sized {
    type Token: PartialEq + Copy;
    type Slice: PartialEq + Clone + Length;
    type InSlice: PartialEq + Clone + Length;
    type Many: Length;
    type Context: Display;

    fn peek(&mut self) -> Option<Self::Token>;
    fn peek_slice(&mut self, _in: Self::InSlice) -> Option<Self::Slice>;
    fn advance(&mut self, _n: usize);
    fn is_empty(&mut self) -> bool;
    fn take_many<F: FnMut(Self::Token) -> bool>(&mut self, cond: F) -> Self::Many;
    fn skip_many<F: FnMut(Self::Token) -> bool>(&mut self, cond: F) -> usize;

    #[inline(always)]
    fn context(&mut self) -> Option<Self::Context> {
        None
    }
}

impl<'a> Input for &'a str {
    type Token = char;
    type InSlice = &'a str;
    type Slice = &'a str;
    type Many = Self::Slice;
    type Context = &'a str;

    #[inline(always)]
    fn peek(&mut self) -> Option<Self::Token> {
        self.chars().next()
    }

    #[inline(always)]
    fn peek_slice(&mut self, slice: Self::InSlice) -> Option<Self::Slice> {
        if self.len() >= slice.len() {
            let out_slice = &self[..slice.len()];
            if out_slice == slice {
                return Some(out_slice)
            }
        }

        None
    }

    #[inline(always)]
    fn skip_many<F>(&mut self, cond: F) -> usize
        where F: FnMut(Self::Token) -> bool
    {
        self.take_many(cond).len()
    }

    #[inline]
    fn take_many<F>(&mut self, mut cond: F) -> Self::Many
        where F: FnMut(Self::Token) -> bool
    {
        for (i, c) in self.chars().enumerate() {
            if !cond(c) {
                let value = &self[..i];
                self.advance(i);
                return value;
            }
        }

        let value = *self;
        self.advance(self.len());
        value
    }

    #[inline(always)]
    fn advance(&mut self, count: usize) {
        *self = &self[count..];
    }

    #[inline(always)]
    fn is_empty(&mut self) -> bool {
        str::is_empty(self)
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

        if is_debug!() {
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

impl<'a> Input for Text<'a> {
    type Token = char;
    type InSlice = &'a str;
    type Slice = &'a str;
    type Many = Self::Slice;
    type Context = Position<'a>;

    #[inline(always)]
    fn peek(&mut self) -> Option<Self::Token> {
        self.current.peek()
    }

    #[inline(always)]
    fn peek_slice(&mut self, slice: Self::InSlice) -> Option<Self::Slice> {
        self.current.peek_slice(slice)
    }

    #[inline(always)]
    fn skip_many<F>(&mut self, cond: F) -> usize
        where F: FnMut(Self::Token) -> bool
    {
        self.current.skip_many(cond)
    }

    #[inline(always)]
    fn take_many<F>(&mut self, cond: F) -> Self::Many
        where F: FnMut(Self::Token) -> bool
    {
        self.current.take_many(cond)
    }

    #[inline(always)]
    fn advance(&mut self, count: usize) {
        self.current.advance(count)
    }

    #[inline(always)]
    fn is_empty(&mut self) -> bool {
        self.current.is_empty()
    }

    fn context(&mut self) -> Option<Position<'a>> {
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

use std::fs::File;
use std::io::{self, Read, BufReader};

use std::cmp::min;
use std::marker::PhantomData;

// Ideally, this would hold a `String` inside. But we need a lifetime parameter
// here so we can return an &'a str from `peek_slice`. The alternative is to
// give a lifetime to the `Input` trait and use it in the `peek_slice` method.
// But that lifetime will pollute everything. Finally, the _correct_ thing is
// for Rust to let us reference the lifetime of `self` in an associated type.
// That requires something like https://github.com/rust-lang/rfcs/pull/1598.
#[derive(Debug)]
pub struct StringFile<'s> {
    buffer: Vec<u8>,
    consumed: usize,
    pos: usize,
    reader: BufReader<File>,
    _string: PhantomData<&'s str>
}

impl<'s> StringFile<'s> {
    #[inline(always)]
    pub fn open(path: &str) -> io::Result<StringFile<'s>> {
        Ok(StringFile::new(File::open(path)?, 1024))
    }

    #[inline(always)]
    pub fn open_with_cap(path: &str, cap: usize) -> io::Result<StringFile<'s>> {
        Ok(StringFile::new(File::open(path)?, cap))
    }

    #[inline(always)]
    pub fn new(file: File, cap: usize) -> StringFile<'s> {
        StringFile {
            buffer: vec![0; cap],
            consumed: 0,
            pos: 0,
            reader: BufReader::new(file),
            _string: PhantomData
        }
    }

    #[inline(always)]
    pub fn available(&self) -> usize {
        self.pos - self.consumed
    }

    fn read_into_peek(&mut self, num: usize) -> io::Result<usize> {
        if self.available() >= num {
            return Ok(num);
        }

        let needed = num - self.available();
        let to_read = min(self.buffer.len() - self.pos, needed);
        let (i, j) = (self.pos, self.pos + to_read);
        let read = self.reader.read(&mut self.buffer[i..j])?;

        self.pos += read;
        Ok(self.available())
    }

    // Panics if at least `num` aren't available.
    #[inline(always)]
    fn peek_bytes(&self, num: usize) -> &[u8] {
        &self.buffer[self.consumed..(self.consumed + num)]
    }

    fn consume(&mut self, num: usize) {
        if self.pos < num {
            let left = (num - self.pos) as u64;
            self.consumed = 0;
            self.pos = 0;
            // TOOD: Probably don't ignore this?
            let _ = io::copy(&mut self.reader.by_ref().take(left), &mut io::sink());
        } else {
            self.consumed += num;
        }
    }

    #[inline]
    fn peek_char(&mut self) -> Option<char> {
        let available = match self.read_into_peek(4) {
            Ok(n) => n,
            Err(_) => return None
        };

        let bytes = self.peek_bytes(available);
        let string = match ::std::str::from_utf8(bytes) {
            Ok(string) => string,
            Err(e) => match ::std::str::from_utf8(&bytes[..e.valid_up_to()]) {
                Ok(string) => string,
                Err(_) => return None
            }
        };

        string.chars().next()
    }
}

impl<'s> Input for StringFile<'s> {
    type Token = char;
    type InSlice = &'s str;
    type Slice = &'s str;
    type Many = String;
    type Context = &'s str;

    // If we took Self::Token here, we'd know the length of the character.
    #[inline(always)]
    fn peek(&mut self) -> Option<Self::Token> {
        self.peek_char()
    }

    fn take_many<F: FnMut(Self::Token) -> bool>(&mut self, mut cond: F) -> Self::Many {
        let mut result = String::new();
        while let Some(c) = self.peek_char() {
            if cond(c) {
                result.push(c);
                self.consume(c.len_utf8());
            } else {
                break;
            }
        }

        result
    }

    fn skip_many<F: FnMut(Self::Token) -> bool>(&mut self, mut cond: F) -> usize {
        let mut taken = 0;
        while let Some(c) = self.peek_char() {
            if cond(c) {
                self.consume(c.len_utf8());
                taken += 1;
            } else {
                return taken;
            }
        }

        taken
    }

    fn peek_slice(&mut self, slice: Self::InSlice) -> Option<Self::Slice> {
        let available = match self.read_into_peek(slice.len()) {
            Ok(n) => n,
            Err(_) => return None
        };

        let bytes = self.peek_bytes(available);
        let string = match ::std::str::from_utf8(bytes) {
            Ok(string) => string,
            Err(e) => match ::std::str::from_utf8(&bytes[..e.valid_up_to()]) {
                Ok(string) => string,
                Err(_) => return None
            }
        };

        match string == slice {
            true => Some(slice),
            false => None
        }
    }

    #[inline(always)]
    fn advance(&mut self, count: usize) {
        self.consume(count);
    }

    #[inline(always)]
    fn is_empty(&mut self) -> bool {
        match self.read_into_peek(1) {
            Ok(0) | Err(_) => true,
            Ok(_) => false,
        }
    }
}

impl<'a> Input for &'a [u8] {
    type Token = u8;
    type InSlice = &'a [u8];
    type Slice = &'a [u8];
    type Many = Self::Slice;
    type Context = &'a str;

    #[inline(always)]
    fn peek(&mut self) -> Option<Self::Token> {
        match self.is_empty() {
            true => None,
            false => Some(self[0])
        }
    }

    #[inline(always)]
    fn peek_slice(&mut self, slice: Self::InSlice) -> Option<Self::Slice> {
        if self.len() >= slice.len() {
            let out_slice = &self[..slice.len()];
            if out_slice == slice {
                return Some(out_slice)
            }
        }

        None
    }

    #[inline(always)]
    fn skip_many<F>(&mut self, cond: F) -> usize
        where F: FnMut(Self::Token) -> bool
    {
        self.take_many(cond).len()
    }

    fn take_many<F>(&mut self, mut cond: F) -> Self::Many
        where F: FnMut(Self::Token) -> bool
    {
        for (i, c) in self.iter().enumerate() {
            if !cond(*c) {
                let value = &self[..i];
                self.advance(i);
                return value;
            }
        }

        let value = *self;
        self.advance(self.len());
        value
    }

    #[inline(always)]
    fn advance(&mut self, count: usize) {
        *self = &self[count..];
    }

    #[inline(always)]
    fn is_empty(&mut self) -> bool {
        self.len() == 0
    }
}
