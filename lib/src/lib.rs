#![feature(core_intrinsics)]
#![feature(proc_macro_hygiene)]
#![feature(specialization)]

#[allow(unused_imports)] #[macro_use] extern crate pear_codegen;
#[doc(hidden)] pub use pear_codegen::*;

#[macro_use] mod macros;
mod input;
mod result;
mod debug;

#[macro_use] pub mod combinators;
pub mod parsers;

pub use input::*;
pub use result::*;
pub use debug::{parser_entry, parser_exit};

// TODO:
//  1) Figure out how to make maybe! macro work.
//      - I think this is a rustc bug. Something with name resolution.
//  2) Figure out how to pass `input` to macros, if at all.
//      - Perhaps only pass when macro name starts with `pear_`.
//  3) Pass parser name into `pear_error`.
