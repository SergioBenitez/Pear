use std::borrow::Cow;
use std::ops::{Index, Range};
use std::fmt::{self, Debug};

use pear::{Input, Length};

pub trait AsPtr {
    type Output;
    fn as_ptr(&self) -> *const Self::Output;
}

impl AsPtr for str {
    type Output = u8;

    fn as_ptr(&self) -> *const u8 {
        str::as_ptr(self)
    }
}

impl AsPtr for [u8] {
    type Output = u8;

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

impl<'a, T: ?Sized + ToOwned + 'a> Indexed<'a, T>
    where T: Length + AsPtr + Index<Range<usize>, Output = T>
{
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

    /// Retrieves the string `self` corresponds to. If `self` is derived from
    /// indexes, the corresponding subslice of `string` is returned. Otherwise,
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

macro_rules! impl_indexed_input {
    ($T:ty, token = $token:ty) => (
        impl<'a> From<&'a $T> for IndexedInput<'a, $T> {
            #[inline(always)]
            fn from(source: &'a $T) -> Self {
                IndexedInput { source: source, current: source }
            }
        }

        impl<'a> Input for IndexedInput<'a, $T> {
            type Token = $token;
            type InSlice = &'a $T;
            type Slice = Indexed<'static, $T>;
            type Many = Indexed<'static, $T>;
            type Context = &'a str;

            #[inline(always)]
            fn peek(&mut self) -> Option<Self::Token> {
                self.current.peek()
            }

            #[inline(always)]
            fn peek_slice(&mut self, slice: Self::InSlice) -> Option<Self::Slice> {
                self.current.peek_slice(slice)
                    .map(|slice| unsafe {
                        Indexed::unchecked_from(slice, self.source)
                    })
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
                println!("Currentm: {:?}", self.current);
                let many = self.current.take_many(cond);
                println!("Currentm+: {:?}", self.current);
                unsafe { Indexed::unchecked_from(many, self.source) }
            }

            #[inline(always)]
            fn advance(&mut self, count: usize) {
                self.current.advance(count)
            }

            #[inline(always)]
            fn is_empty(&mut self) -> bool {
                println!("Current: {:?}", self.current);
                self.current.is_empty()
            }
        }
    )
}


impl_indexed_input!([u8], token = u8);
// impl_indexed_input!(str, token = char);
