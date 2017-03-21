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
macro_rules! repeat_until {
    ($input:expr, $cond:expr, $($inner:tt)*) => ({
        let mut _result = $crate::ParseResult::Done(());
        loop {
            if let $crate::ParseResult::Done(_) = eof($input) {
                _result = $crate::ParseResult::Done(());
                break;
            }

            if let $crate::ParseResult::Done(_) = parse!($input, $cond) {
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

        _result
    });
}

// This one gives you the last result, while `repeat` does not.
#[macro_export]
macro_rules! many {
    ($input:expr, $($inner:tt)*) => ({
        #[warn(unused_assignments)]
        let mut result = parse!($input, { $($inner)* });
        loop {
            if let $crate::ParseResult::Done(_) = $crate::parsers::eof($input) {
                break;
            }

            match result {
                $crate::ParseResult::Done(_) => {
                    result = parse!($input, { $($inner)* });
                    continue;
                },
                $crate::ParseResult::Error(e) => {
                    result = $crate::ParseResult::Error(e);
                    break;
                }
            }
        }

        result
    });
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
macro_rules! ignore {
    ($input:expr, $($inner:tt)*) => ($($inner)*);
    ($($inner:tt)*) => ($($inner)*)
}

// #[macro_export]
// macro_rules! lift {
//     ($input:expr, $name:ident($($inner:tt)*)) => {
//         ::pear::ParseResult::Done(|i| $name(i, $($inner)*))
//     }
// }

#[macro_export]
macro_rules! from {
    ($input:expr, $result:expr) => (::pear::ParseResult::from($result));
    ($result:expr) => (::pear::ParseResult::from($result));
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

#[inline]
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
