//! Sets up a log sink to view logs from the FFI.

mod ffi;
mod level_filter;
mod logger;
mod sink;
mod status;
mod inmem_buffer;

pub use crate::log::ffi::{
    pactffi_logger_apply,
    pactffi_logger_attach_sink,
    pactffi_logger_init,
    pactffi_fetch_log_buffer,
    pactffi_log_to_stdout,
    pactffi_log_to_stderr,
    pactffi_log_to_file,
    pactffi_log_to_buffer
};
