//! A crate exposing the `pact_matching` API to other languages
//! via a C Foreign Function Interface.

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]

pub mod error;
pub mod log;
pub(crate) mod util;
