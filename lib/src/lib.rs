extern crate num_traits;

mod input;
pub mod parsers;

pub use input::Input;

#[macro_export]
macro_rules! parse {
    ($($input:tt)*) => ({
        #[derive(nosh_parse)]
        #[allow(unused)]
        enum DummyEnumForProcMacros {
            Input = (stringify!($($input)*), 0).1
        }

        get_expr!()
    })
}

pub type ParseError = &'static str;

#[derive(Debug)]
pub enum ParseResult<I: Input, R> {
    Done(I, R),
    Error(ParseError)
}

fn main() {
}
