use {Result, Input};
use parsers::*;

#[macro_export]
macro_rules! any {
    ($input:expr, $case:expr, $($rest:expr),*) => (
        match parse!($input, $case) {
            $crate::Result::Ok(output) => {
                $crate::Result::Ok(output)
            }
            $crate::Result::Err(_) => {
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
            $crate::Result::Ok(val) => {
                $crate::Result::Ok(Some(val))
            }
            $crate::Result::Err(_) => {
                $crate::Result::Ok(None)
            }
        }
    })
}

#[macro_export]
macro_rules! collect {
    // Ideally we could just call `repeat_while!` here, but rust says that it
    // doesn't have enough type information.
    ($input:expr, $value:expr, $cond:expr) => ({
        let mut values = Vec::new();
        #[warn(unused_assignments)]
        loop {
            if let $crate::Result::Ok(_) = eof($input) {
                break;
            }

            match parse!($input, $value) {
                $crate::Result::Ok(value) => values.push(value),
                $crate::Result::Err(_) => break
            }

            if let $crate::Result::Err(_) = parse!($input, $cond) {
                break;
            }
        }

        $crate::Result::Ok(values)
    });

    ($input:expr, $value:expr) => ({
        let mut values = Vec::new();
        #[warn(unused_assignments)]
        loop {
            if let $crate::Result::Ok(_) = eof($input) {
                break;
            }

            match parse!($input, $value) {
                $crate::Result::Ok(value) => values.push(value),
                $crate::Result::Err(_) => break
            }
        }

        $crate::Result::Ok(values)
    });
}

#[inline]
pub fn many<I: Input, O, F>(input: &mut I, f: F) -> Result<O, I>
    where F: Fn(&mut I) -> Result<O, I>
{
    loop {
        let output = f(input)?;
        if let Ok(_) = eof(input) {
            return Ok(output);
        }
    }
}

#[inline(always)]
pub fn surrounded<I: Input, O, F, P>(input: &mut I, mut p: P, f: F) -> Result<O, I>
    where F: Copy + FnMut(I::Token) -> bool,
          P: FnMut(&mut I) -> Result<O, I>
{
    skip_while(input, f)?;
    let output = p(input)?;
    skip_while(input, f)?;
    Ok(output)
}
