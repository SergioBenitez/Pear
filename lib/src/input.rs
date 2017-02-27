use std::fmt::Debug;

use num_traits::{Num, Zero, One};

pub trait Input: Sized + PartialEq + Copy + Debug {
    type Token: PartialEq + Copy;
    type Index: Num + Copy;

    fn find(&self, token: Self::Token) -> Option<Self::Index>;
    fn get(&self, index: Self::Index) -> Option<Self::Token>;
    fn slice(&self, from: Self::Index, to: Self::Index) -> Self;
    fn try_slice_to(&self, to: Self::Index) -> Option<Self>;
    fn slice_from(&self, from: Self::Index) -> Self;
    fn len(&self) -> Self::Index;

    #[inline(always)]
    fn slice_to(&self, index: Self::Index) -> Self {
        self.try_slice_to(index).expect("slice_to index out of bounds")
    }

    #[inline(always)]
    fn split_at(&self, index: Self::Index) -> (Self, Self) {
        (self.slice_to(index), self.slice_from(index + Self::Index::one()))
    }
}

impl<'a> Input for &'a str {
    type Token = u8;
    type Index = usize;

    #[inline(always)]
    fn find(&self, c: u8) -> Option<usize> {
        str::find(self, c as char)
    }

    fn get(&self, index: Self::Index) -> Option<Self::Token> {
        let bytes = self.as_bytes();
        if index < bytes.len() {
            unsafe { Some(*bytes.get_unchecked(index)) }
        } else {
            None
        }
    }

    #[inline(always)]
    fn slice(&self, from: Self::Index, to: Self::Index) -> Self {
        &self[from..to]
    }

    #[inline(always)]
    fn try_slice_to(&self, index: Self::Index) -> Option<Self> {
        if index <= self.len() {
            unsafe { Some(self.slice_unchecked(0, index)) }
        } else {
            None
        }
    }

    #[inline(always)]
    fn slice_from(&self, from: Self::Index) -> Self {
        &self[from..]
    }

    #[inline(always)]
    fn split_at(&self, index: Self::Index) -> (Self, Self) {
        str::split_at(self, index)
    }

    #[inline(always)]
    fn len(&self) -> Self::Index {
        str::len(self)
    }
}

