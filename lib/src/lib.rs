// #![feature(core_intrinsics)]
#![feature(proc_macro_hygiene)]
#![feature(specialization)]

#[allow(unused_imports)] #[macro_use] extern crate pear_codegen;
#[doc(hidden)] pub use pear_codegen::*;

#[cfg(feature = "color")]
extern crate yansi;

#[macro_use] mod macros;
mod input;
mod result;
mod debug;

#[macro_use] pub mod combinators;
pub mod parsers;

pub use input::*;
pub use result::*;
pub use debug::{parser_entry, parser_exit};
