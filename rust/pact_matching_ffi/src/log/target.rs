//! The special log target to use for logs from this crate.

#![allow(unused)]

/// The log target to use through the crate. Note that normally the logging macros would
/// pull the module path of the code calling the logger, but in this case everything ends up
/// in a flat namespace on the C side anyway, and the logging makes clear what's being called.
/// So having a singular log target is fine.
pub(crate) const TARGET: &str = "pact::matching::ffi";
