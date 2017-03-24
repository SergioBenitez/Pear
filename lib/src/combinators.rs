use {ParseResult, Input};
use ParseResult::*;
use parsers::*;

#[macro_export]
macro_rules! switch {
    ($input:expr, _ => $value:expr) => (parse!($input, $value));

    ($input:expr, $matcher:expr => $value:expr, $($tokens:tt)*) => ({
        match parse!($input, $matcher) {
            $crate::ParseResult::Done(_) => {
                parse!($input, $value)
            }
            $crate::ParseResult::Error(_) => {
                switch!($input, $($tokens)*)
            }
        }
    });

    ($input:expr, $matcher:expr => $value:expr) => ({
        match parse!($input, $matcher) {
            $crate::ParseResult::Done(_) => {
                parse!($input, $value)
            }
            $crate::ParseResult::Error(e) => {
                $crate::ParseResult::Error(e)
            }
        }
    });
}

#[macro_export]
macro_rules! try_switch {
    ($input:expr, _ => $value:expr) => (parse!($input, $value));

    ($input:expr, $matcher:expr => $value:expr, $($tokens:tt)*) => ({
        match parse!($input, $matcher) {
            $crate::ParseResult::Done(_) => {
                parse!($input, $value)
            }
            $crate::ParseResult::Error(_) => {
                switch!($input, $($tokens)*)
            }
        }
    });

    ($input:expr, $matcher:expr => $value:expr) => ({
        match parse!($input, $matcher) {
            $crate::ParseResult::Done(_) => {
                parse!($input, $value)
            }
            $crate::ParseResult::Error(_) => {
                $crate::ParseResult::Done(())
            }
        }
    });
}

#[macro_export]
macro_rules! any {
    ($input:expr, $case:expr, $($rest:expr),*) => (
        match parse!($input, $case) {
            $crate::ParseResult::Done(output) => {
                $crate::ParseResult::Done(output)
            }
            $crate::ParseResult::Error(_) => {
                any!($input, $($rest),*)
            }
        }
    );

    ($input:expr, $case:expr) => (
        parse!($input, $case)
    )
}

#[macro_export]
macro_rules! maybe {
    ($input:expr, $value:expr) => ({
        match parse!($input, $value) {
            $crate::ParseResult::Done(val) => {
                $crate::ParseResult::Done(Some(val))
            }
            $crate::ParseResult::Error(_) => {
                $crate::ParseResult::Done(None)
            }
        }
    })
}

#[macro_export]
macro_rules! repeat {
    ($input:expr, $($inner:tt)*) => ({
        #[warn(unused_assignments)]
        let mut _result = $crate::ParseResult::Done(());
        loop {
            if let $crate::ParseResult::Done(_) = eof($input) {
                _result = $crate::ParseResult::Done(());
                break;
            }

            match parse!($input, { $($inner)* }) {
                $crate::ParseResult::Done(_) => continue,
                $crate::ParseResult::Error(e) => {
                    _result = $crate::ParseResult::Error(e);
                    break;
                }
            }
        }

        match _result {
            $crate::ParseResult::Done(_) => $crate::ParseResult::Done(()),
            $crate::ParseResult::Error(e) => $crate::ParseResult::Error(e)
        }
    });
}

#[macro_export]
macro_rules! try_repeat {
    ($input:expr, $($inner:tt)*) => ({
        #[warn(unused_assignments)]
        loop {
            if let $crate::ParseResult::Done(_) = eof($input) {
                break;
            }

            match parse!($input, { $($inner)* }) {
                $crate::ParseResult::Done(_) => continue,
                $crate::ParseResult::Error(_) => break
            }
        }

        $crate::ParseResult::Done(())
    });
}

#[macro_export]
macro_rules! repeat_while {
    ($input:expr, $cond:expr, $($inner:tt)*) => ({
        #[warn(unused_assignments)]
        let mut _result = $crate::ParseResult::Done(());
        loop {
            if let $crate::ParseResult::Done(_) = eof($input) {
                _result = $crate::ParseResult::Done(());
                break;
            }

            if let $crate::ParseResult::Error(e) = parse!($input, { $($inner)* }) {
                _result = $crate::ParseResult::Error(e);
                break;
            }

            if let $crate::ParseResult::Error(_) = parse!($input, $cond) {
                break;
            }
        }

        _result
    });
}

#[macro_export]
macro_rules! switch_repeat {
    ($input:expr, $($cases:tt)*) => (repeat!($input, switch!($($cases)*)))
}

#[macro_export]
macro_rules! collect {
    // Ideally we could just call `repeat_while!` here, but rust says that it
    // doesn't have enough type information.
    ($input:expr, $value:expr, $cond:expr) => ({
        let mut values = Vec::new();
        #[warn(unused_assignments)]
        loop {
            if let $crate::ParseResult::Done(_) = eof($input) {
                break;
            }

            match parse!($input, $value) {
                $crate::ParseResult::Done(value) => values.push(value),
                $crate::ParseResult::Error(_) => break
            }

            if let $crate::ParseResult::Error(_) = parse!($input, $cond) {
                break;
            }
        }

        $crate::ParseResult::Done(values)
    });
}

#[inline]
pub fn many<I: Input, O, F>(input: &mut I, f: F) -> ParseResult<I, O>
    where F: Fn(&mut I) -> ParseResult<I, O>
{
    loop {
        let output = match f(input) {
            Done(output) => output,
            Error(e) => return Error(e)
        };

        if let Done(_) = eof(input) {
            return Done(output);
        }
    }
}

#[inline(always)]
pub fn surrounded<I: Input, O, F, P>(input: &mut I, p: P, f: F) -> ParseResult<I, O>
    where F: Copy + Fn(I::Token) -> bool,
          P: Fn(&mut I) -> ParseResult<I, O>
{
    skip_while(input, f);

    let output = match p(input) {
        Done(output) => Done(output),
        Error(e) => return Error(e)
    };

    skip_while(input, f);

    output
}
