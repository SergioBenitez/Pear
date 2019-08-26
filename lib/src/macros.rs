//! Macros.
//!
//! Attribute Macros:
//!
//!   * [`#[parser]`](#parser)
//!
//!     The core attribute macro. Can only be applied to free functions with at
//!     least one parameter and a return value. To typecheck, the free function
//!     must meet the following typing requirements:
//!
//!     - The _first_ parameter's type `&mut I` must be a mutable reference to a
//!       type that implements [`Input`]. This is the _input_ parameter.
//!     - The return type must be [`Result<O, I>`] where `I` is the inner type
//!       of the input parameter and `O` can be any type.
//!
//!     The following transformations are applied to the _contents_ of the
//!     attributed function:
//!
//!     - The functions first parameter (of type `&mut I`) is passed as the
//!       first parameter to every function call in the function with a posfix
//!       `?`. That is, every function call of the form `foo(a, b, c, ...)?` is
//!       converted to `foo(input, a, b, c, ...)?` where `input` is the input
//!       parameter.
//!     - The inputs to every macro whose name starts with `parse_` are prefixed
//!       with `[PARSER_NAME, INPUT]` where `PARSER_NAME` is the raw string
//!       literal of the functon's name and `INPUT` is the input parameter.
//!
//!     The following transformations are applied _around_ the attributed
//!     function:
//!
//!     - The [`Input::mark()`] method is called before the function executes.
//!       The returned mark, if any, is stored on the stack.
//!     - A return value of `O` is automatically converted (or "lifted") into a
//!       type of [`Result<O, I>`] by wrapping it in `Ok`.
//!     - If the function returns an `Err`, [`Input::context()`] is called with
//!       the current mark, and the returned context, if any, is pushed into the
//!       error via [`ParseError::push_context()`].
//!     - The [`Input::unmark()`] method is called after the function executes,
//!       passing in the current mark.
//!
//!     # Example
//!
//!     ```rust
//!     #![feature(proc_macro_hygiene)]
//!
//!     use pear::result::Result;
//!     use pear::macros::parser;
//!     use pear::parsers::*;
//!     #
//!     # use pear::macros::parse_declare;
//!     # parse_declare!(Input<'a>(Token = char, Slice = &'a str, Many = &'a str));
//!
//!     #[parser]
//!     fn ab_in_dots<'a, I: Input<'a>>(input: &mut I) -> Result<&'a str, I> {
//!         eat('.')?;
//!         let inside = take_while(|&c| c == 'a' || c == 'b')?;
//!         eat('.')?;
//!
//!         inside
//!     }
//!
//!     # use pear::{macros::parse, input::Text};
//!     #
//!     let x = parse!(ab_in_dots: &mut Text::from(".abba."));
//!     assert_eq!(x.unwrap(), "abba");
//!
//!     let x = parse!(ab_in_dots: &mut Text::from(".ba."));
//!     assert_eq!(x.unwrap(), "ba");
//!
//!     let x = parse!(ab_in_dots: &mut Text::from("..."));
//!     assert!(x.is_err());
//!     ```
//!
//! Bang Macros:
//!
//!   * [`parse!`](#parse)
//!
//!     Runs the parser with the given ame and input. After the parser returns,
//!     runs the [`eof()`] parser. Returns the combined result.
//!
//!     Syntax:
//!
//!     ```text
//!     parse := PARSER_NAME ':' INPUT_EXPR
//!
//!     PARSER_NAME := rust identifier to parser function
//!     INPUT_EXPR := any valid rust expression which resolves to a mutable
//!                   reference to type that implements `Input`
//!     ```
//!
//!   * [`parse_context!`](#parse_context)
//!
//!     Invoked with no arguments: `parse_context!()`. Returns the current
//!     context given the current mark.
//!
//!   * [`parse_marker!`](#parse_marker)
//!
//!     Invoked with no arguments: `parse_marker!()`. Returns the current mark.
//!
//!   * [`switch!`](#switch)
//!
//!     Invoked much like match, except each condition must be a parser, which is
//!     executed, and the corresponding arm is executed only if the parser
//!     succeeds. Once a condition succeeds, no other condition is executed.
//!
//!     ```rust,ignore
//!     switch! {
//!         parser() => expr,
//!         x@parser1() | x@parser2(a, b, c) => expr(x),
//!         _ => last_expr
//!     }
//!     ```
//!
//!   * [`parse_declare!`](#parse_declare)
//!   * [`parse_error!`](#parse_error)
//!   * [`parse_try!`](#parse_try)
//!
//! [`Input`]: crate::input::Input
//! [`Result<O, I>`]: crate::result::Result
//! [`Input::mark()`]: crate::input::Input::mark()
//! [`Input::unmark()`]: crate::input::Input::unmark()
//! [`Input::context()`]: crate::input::Input::context()
//! [`ParseError::push_context()`]: crate::error::ParseError::push_context()
//! [`eof()`]: crate::parsers::eof()

#[doc(hidden)] pub use pear_codegen::{parser, switch};
#[doc(hidden)] pub use crate::{parse, parse_declare, parse_error, parse_try, is_parse_debug};
#[doc(hidden)] pub use crate::{parse_marker, parse_mark, parse_context};

#[doc(hidden)]
#[macro_export]
macro_rules! parse {
    ($parser:ident : $e:expr) => ({
        let input = $e;
        (move || {
            let result = $parser(input)?;
            $crate::parsers::eof(input)?;
            $crate::result::AsResult::as_result(result)
        })()
    })
}

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! parse_declare {
    (pub($($inner:tt)+) $($rest:tt)*) => { $crate::_parse_declare!([pub($($inner)+)] $($rest)*); };
    (pub $($rest:tt)*) => { $crate::_parse_declare!([pub] $($rest)*); };
    ($($rest:tt)*) => { $crate::_parse_declare!([] $($rest)*); }
}

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! _parse_declare {
    ([$($vis:tt)*] $input:ident $(<$($gen:tt),+>)* ($($T:ident = $t:ty),*)) => {
        $($vis)* trait $input $(<$($gen),+>)*: $crate::input::Input<$($T = $t),*> {  }

        impl<$($($gen,)+)* T> $input $(<$($gen)+>)* for T
            where T: $crate::input::Input<$($T = $t),*> + $($($gen),+)* {  }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! parse_error {
    ([$n:expr; $i:expr; $m:expr] $err:expr) => {
        parse_error!([$n; $i; $m] $err,)
    };
    ([$n:expr; $i:expr; $m:expr] $fmt:expr, $($arg:tt)*) => {
        Err($crate::error::ParseError::custom(format!($fmt, $($arg)*)))
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! parse_marker {
    ([$n:expr; $i:expr; $marker:expr]) => (*$marker);
}

#[doc(hidden)]
#[macro_export]
macro_rules! parse_mark {
    ([$info:expr; $input:expr; $marker:expr]) => {{
        *$marker = $crate::input::Input::mark($input, $info);
    }}
}

#[doc(hidden)]
#[macro_export]
macro_rules! parse_context {
    ([$n:expr; $i:expr; $marker:expr]) => ($crate::input::Input::context($i, $marker));
}

/// FIXME: This is an issue with rustc here where if `$input` is `expr`
/// everything fails.
#[doc(hidden)]
#[macro_export]
macro_rules! parse_try {
    ([$n:expr; $input:ident; $m:expr] $e:expr) => {{
        $crate::macros::switch! { [$n;$input;$m] result@$e => { Some(result) }, _ => { None } }
    }};
    ([$n:expr; $input:ident; $m:expr] $e:expr => $r:expr) => {{
        $crate::macros::switch! { [$n;$input;$m] $e => { Some($r) }, _ => { None } }
    }};
    ([$n:expr; $input:ident; $m:expr] $pat:ident@$e:expr => $r:expr) => {{
        $crate::macros::switch! { [$n;$input;$m] $pat@$e => { Some($r) }, _ => { None } }
    }}
}

#[doc(hidden)]
#[macro_export]
macro_rules! is_parse_debug {
    () => ({
        #[cfg(debug_assertions)]
        let result = ::std::env::var("PARSE_DEBUG").is_ok();
        #[cfg(not(debug_assertions))]
        let result = false;
        result
    });

    ($kind:expr) => ({
        #[cfg(debug_assertions)]
        let result = ::std::env::var("PARSE_DEBUG").map(|v| v == $kind).unwrap_or(false);
        #[cfg(not(debug_assertions))]
        let result = false;
        result
    })
}
