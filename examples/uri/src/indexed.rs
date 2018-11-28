use std::borrow::Cow;
use std::ops::{Index, Range};
use std::fmt::{self, Debug};

use pear::{Input, Slice, Position, Length};

pub trait AsPtr {
    fn as_ptr(&self) -> *const u8;
    // unsafe fn from_raw<'a>(raw: *const u8, length: usize) -> &T;
}

impl AsPtr for str {
    fn as_ptr(&self) -> *const u8 {
        str::as_ptr(self)
    }
}

impl AsPtr for [u8] {
    fn as_ptr(&self) -> *const u8 {
        <[u8]>::as_ptr(self)
    }
}


#[derive(PartialEq)]
pub enum Indexed<'a, T: ?Sized + ToOwned + 'a> {
    Indexed(usize, usize),
    Concrete(Cow<'a, T>)
}

impl<'a, T: ?Sized + ToOwned + 'a, C: Into<Cow<'a, T>>> From<C> for Indexed<'a, T> {
    #[inline(always)]
    fn from(value: C) -> Indexed<'a, T> {
        Indexed::Concrete(value.into())
    }
}

impl<'a, T: ?Sized + ToOwned + 'a> Indexed<'a, T> {
    #[inline(always)]
    pub unsafe fn coerce<U: ?Sized + ToOwned>(self) -> Indexed<'a, U> {
        match self {
            Indexed::Indexed(a, b) => Indexed::Indexed(a, b),
            _ => panic!("cannot convert indexed T to U unless indexed")
        }
    }
}

use std::ops::Add;

impl<'a, T: ?Sized + ToOwned + 'a> Add for Indexed<'a, T> {
    type Output = Indexed<'a, T>;

    fn add(self, other: Indexed<'a, T>) -> Indexed<'a, T> {
        match self {
            Indexed::Indexed(a, b) => match other {
                Indexed::Indexed(c, d) if b == c && a < d => Indexed::Indexed(a, d),
                _ => panic!("+ requires indexed")
            }
            _ => panic!("+ requires indexed")
        }
    }
}

impl<'a, T: ?Sized + ToOwned + 'a> Indexed<'a, T>
    where T: Length + AsPtr + Index<Range<usize>, Output = T>
{
    // Returns `None` if `needle` is not a substring of `haystack`.
    pub fn checked_from(needle: &T, haystack: &T) -> Option<Indexed<'a, T>> {
        let haystack_start = haystack.as_ptr() as usize;
        let needle_start = needle.as_ptr() as usize;

        if needle_start < haystack_start {
            return None;
        }

        if (needle_start + needle.len()) > (haystack_start + haystack.len()) {
            return None;
        }

        let start = needle_start - haystack_start;
        let end = start + needle.len();
        Some(Indexed::Indexed(start, end))
    }

    // Caller must ensure that `needle` is a substring of `haystack`.
    pub unsafe fn unchecked_from(needle: &T, haystack: &T) -> Indexed<'a, T> {
        let haystack_start = haystack.as_ptr() as usize;
        let needle_start = needle.as_ptr() as usize;

        let start = needle_start - haystack_start;
        let end = start + needle.len();
        Indexed::Indexed(start, end)
    }

    /// Whether this string is derived from indexes or not.
    pub fn is_indexed(&self) -> bool {
        match *self {
            Indexed::Indexed(..) => true,
            Indexed::Concrete(..) => false,
        }
    }

    /// Whether this string is derived from indexes or not.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Retrieves the string `self` corresponds to. If `self` is derived from
    /// indexes, the corresponding subslice of `source` is returned. Otherwise,
    /// the concrete string is returned.
    ///
    /// # Panics
    ///
    /// Panics if `self` is an indexed string and `string` is None.
    // pub fn to_source(&self, source: Option<&'a T>) -> &T {
    pub fn to_source<'s>(&'s self, source: &'s Option<Cow<T>>) -> &'s T {
        use std::borrow::Borrow;
        if self.is_indexed() && source.is_none() {
            panic!("Cannot convert indexed str to str without base string!")
        }

        match *self {
            Indexed::Indexed(i, j) => &source.as_ref().unwrap()[i..j],
            Indexed::Concrete(ref mstr) => mstr.as_ref(),
        }
    }

}

impl<'a, T: ToOwned + ?Sized + 'a> Clone for Indexed<'a, T> {
    fn clone(&self) -> Self {
        match *self {
            Indexed::Indexed(a, b) => Indexed::Indexed(a, b),
            Indexed::Concrete(ref cow) => Indexed::Concrete(cow.clone())
        }
    }
}

impl<'a, T: ?Sized + 'a> Debug for Indexed<'a, T>
    where T: ToOwned + Debug, T::Owned: Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Indexed::Indexed(a, b) => fmt::Debug::fmt(&(a, b), f),
            Indexed::Concrete(ref cow) => fmt::Debug::fmt(cow, f),
        }
    }
}

impl<'a, T: ?Sized + Length + ToOwned + 'a> Length for Indexed<'a, T> {
    #[inline(always)]
    fn len(&self) -> usize {
        match *self {
            Indexed::Indexed(a, b) => (b - a) as usize,
            Indexed::Concrete(ref cow) => cow.len()
        }
    }
}

#[derive(Debug)]
pub struct IndexedInput<'a, T: ?Sized + 'a> {
    source: &'a T,
    current: &'a T
}

impl<'a, T: ?Sized + 'a> IndexedInput<'a, T> {
    pub fn source(&self) -> &T {
        self.source
    }
}

impl<'a, T: ToOwned + ?Sized + 'a> IndexedInput<'a, T> {
    #[inline(always)]
    pub fn cow_source(&self) -> Cow<'a, T> {
        Cow::Borrowed(self.source)
    }
}

impl<'a> IndexedInput<'a, [u8]> {
    pub fn backtrack(&mut self, n: usize) -> ::pear::Result<(), Self> {
        let source_addr = self.source.as_ptr() as usize;
        let current_addr = self.current.as_ptr() as usize;
        if current_addr > n && (current_addr - n) >= source_addr {
            let size = self.current.len() + n;
            let addr = (current_addr - n) as *const u8;
            self.current = unsafe { ::std::slice::from_raw_parts(addr, size) };
            Ok(())
        } else {
            let diag = format!("({}, {:x} in {:x})", n, current_addr, source_addr);
            Err(pear_error!([backtrack; self] "internal error: {}", diag))
        }
    }

    pub fn len(&self) -> usize {
        self.source.len()
    }
}

macro_rules! impl_indexed_input {
    ($T:ty, token = $token:ty) => (
        impl<'a> From<&'a $T> for IndexedInput<'a, $T> {
            #[inline(always)]
            fn from(source: &'a $T) -> Self {
                IndexedInput { source: source, current: source }
            }
        }

        impl<'a, 'b: 'a> Slice<IndexedInput<'a, $T>> for &'b $T {
            fn eq_slice(&self, other: &Indexed<'a, $T>) -> bool {
                self == &other.to_source(&None)
            }

            fn into_slice(self) -> Indexed<'a, $T> {
                Indexed::Concrete(self.into())
            }
        }

        impl<'a> Input for IndexedInput<'a, $T> {
            type Token = $token;
            type Slice = Indexed<'a, $T>;
            type Many = Indexed<'a, $T>;
            type Context = Context;

            /// Returns a copy of the current token, if there is one.
            fn token(&mut self) -> Option<Self::Token> {
                self.current.token()
            }

            /// Returns a copy of the current slice of size `n`, if there is one.
            fn slice(&mut self, n: usize) -> Option<Self::Slice> {
                self.current.slice(n)
                    .map(|s| unsafe { Indexed::unchecked_from(s, self.source) })
            }

            /// Checks if the current token fulfills `cond`.
            fn peek<F>(&mut self, cond: F) -> bool
                where F: FnMut(&Self::Token) -> bool
            {
                self.current.peek(cond)
            }

            /// Checks if the current slice of size `n` (if any) fulfills `cond`.
            fn peek_slice<F>(&mut self, n: usize, mut cond: F) -> bool
                where F: FnMut(&Self::Slice) -> bool
            {
                self.current.peek_slice(n, |&s| cond(&Indexed::Concrete(s.into())))
            }

            /// Checks if the current token fulfills `cond`. If so, the token is
            /// consumed and returned. Otherwise, returrustc --explain E0284ns `None`.
            fn eat<F>(&mut self, cond: F) -> Option<Self::Token>
                where F: FnMut(&Self::Token) -> bool
            {
                self.current.eat(cond)
            }

            /// Checks if the current slice of size `n` (if any) fulfills `cond`. If so,
            /// the slice is consumed and returned. Otherwise, returns `None`.
            fn eat_slice<F>(&mut self, n: usize, mut cond: F) -> Option<Self::Slice>
                where F: FnMut(&Self::Slice) -> bool
            {
                self.current
                    .eat_slice(n, |&s| cond(&Indexed::Concrete(s.into())))
                    .map(|s| unsafe { Indexed::unchecked_from(s, self.source) })
            }

            /// Takes tokens while `cond` returns true, collecting them into a
            /// `Self::Many` and returning it.
            fn take<F>(&mut self, cond: F) -> Self::Many
                where F: FnMut(&Self::Token) -> bool
            {
                let many = self.current.take(cond);
                unsafe { Indexed::unchecked_from(many, self.source) }
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
                let offset = self.source.len() - self.current.len();
                let bytes: &[u8] = self.current.as_ref();
                let string = String::from_utf8(bytes.into()).ok()?;
                Some(Context { offset, string })
            }
        }
    )
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Context {
    pub offset: usize,
    pub string: String
}

impl ::std::fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const LIMIT: usize = 7;
        write!(f, "{}", self.offset)?;

        if self.string.len() > LIMIT {
            write!(f, " ({}..)", &self.string[..LIMIT])
        } else if !self.string.is_empty() {
            write!(f, " ({})", &self.string)
        } else {
            Ok(())
        }
    }
}

impl_indexed_input!([u8], token = u8);
impl_indexed_input!(str, token = char);
