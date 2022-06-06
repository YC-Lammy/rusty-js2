#![feature(unboxed_closures)]
#![feature(fn_traits)]

use std::{marker, borrow::Borrow};
use std::ops::Deref;

mod jit;
//mod interpretor;
mod operator;
mod value;
mod builtins;
mod heap;
mod string_allocator;

pub mod runtime;
mod vm;
mod error;
mod garbage_collector;
mod parse;
mod module;

mod bindgen;

mod utils;
mod allocator;

pub mod prelude;

