#![feature(proc_macro_hygiene)]
#![feature(specialization)]

#![warn(rust_2018_idioms)]

pub mod macros;
pub mod input;
pub mod result;
pub mod error;
pub mod parsers;
pub mod combinators;

#[doc(hidden)] pub mod debug;
