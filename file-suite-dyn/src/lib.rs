#![doc = include_str!("../README.md")]

pub mod any_conv;

pub mod any_of;

extern crate alloc;

#[doc(hidden)]
pub use ::file_suite_proc::kebab_paste;

#[doc(hidden)]
pub use alloc::boxed::Box;
