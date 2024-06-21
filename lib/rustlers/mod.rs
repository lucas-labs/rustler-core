//! The rustlers module defines [`Rustler`], a trait that allows rustling data from a source.

#![doc = include_str!("README.md")]

mod rustler;

pub mod rustlerjar;
pub mod svc;
pub use rustler::*;
