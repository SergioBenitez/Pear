// TODO(show): Finish 'Show' transition.
//
// Remember:
//
//  * We expect parsers to fail a lot. As such, doing anything expensive, such
//    as allocating a `Box<dyn Trait>` or rendering a `String` is a no-go. We've
//    benchmarked this, and it's a ~10% degradation, but note that this is
//    dependent on the size of the `String`s rendered.
//  * Introducing a `Box<dyn Trait>` is unlikely to work as it will default to
//    a static lifetime and force the introduction of a lifetime variable for
//    anything else. Unsafing into the lifetime _is not sound_.
//  * While the optimizer removes the expensive generation in many cases that
//    the error value is not used, it unfortunately does not do it
//    determistically, even with inlining everywhere.
//
// Ideas:
//
//  1. Ask Token<I> to return a proxy object that's 'static that can be `Show`d.
//     This seems unlikely to work for many cases or be just as slow.
//  2. Hint to parsers whether the error value will be used. If it will not be
//     used, don't generate the error value at all and just signal value. Such
//     an implementation could involve making `pear::Result` a real `enum` with
//     three variants: `Ok(T), Err(E), Failure`, returning `Failure` when no
//     more information is needed. Injection on whether the value is needed and
//     creation of the proper result value should occur automatically. Perhaps
//     the `AsResult` trait can help? To check: are there cases where an
//     itermediary parser will suggest emitting an error where a preceding
//     parser known it doesn't need it?
//
// TODO
// * Print parser arguments in debug/error output.

pub trait Show {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl std::fmt::Display for &dyn Show {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Show::fmt(*self, f)
    }
}

impl<T: Show + ?Sized> Show for &T {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <T as Show>::fmt(self, f)
    }
}

impl<T: Show> Show for Option<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(val) = self {
            <T as Show>::fmt(val, f)?;
        }

        Ok(())
    }
}

impl<T: Show> Show for [T] {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, value) in self.iter().enumerate() {
            if i > 0 { write!(f, " ")?; }
            write!(f, "{}", value as &dyn Show)?;
        }

        write!(f, ")")
    }
}

macro_rules! impl_for_slice_len {
    ($($n:expr),*) => ($(
        impl<T: Show> Show for [T; $n] {
            #[inline(always)]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Show::fmt(&self[..], f)
            }
        }
    )*)
}

impl_for_slice_len!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
    17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32
);

impl<T: Show> Show for Vec<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, value) in self.iter().enumerate() {
            if i > 0 { write!(f, " ")?; }
            write!(f, "{}", value as &dyn Show)?;
        }

        write!(f, ")")
    }
}

impl_show_with! { Debug,
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize
}

macro_rules! impl_with_tick_display {
    ($($T:ty,)*) => ($(
        impl Show for $T {
            #[inline(always)]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    )*)
}

impl_with_tick_display! {
    &str, String, char, std::borrow::Cow<'static, str>,
}
