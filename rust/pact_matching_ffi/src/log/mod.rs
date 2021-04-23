//! Sets up a log sink to view logs from the FFI.

mod ffi;
mod level_filter;
mod logger;
mod sink;
mod status;
mod target;

pub use crate::log::ffi::{
    logger_apply, logger_attach_sink, logger_init,
};
pub(crate) use crate::log::target::TARGET;
