//! The public FFI functions for initializing, adding sinks to, and applying a logger.

use crate::log::level_filter::LevelFilter;
use crate::log::logger::{add_sink, apply_logger, set_logger};
use crate::log::sink::Sink;
use crate::log::status::Status;
use fern::Dispatch;
use libc::{c_char, c_int};
use log::LevelFilter as LogLevelFilter;
use std::convert::TryFrom;
use std::ffi::CStr;

// C API uses something like the pledge API to select write locations, including:
//
// * stdout
// * stderr
// * file w/ file path
//
// The general flow is:
//
// 1. Call `logger_init` to create a `Dispatch` struct.
// 2. Call `logger_attach_sink` to add an additional sink, using bitflags to set the metadata.
// 3. Call `logger_apply` to finalize the logger and enable logging to the configured sinks.
//
// Once `logger_apply` has been called, any additional calls to `logger_attach_sink` will fail
// with an error indicating the logger has been applied already.
//
// ```
// logger_init();
//
// int result = logger_attach_sink("stdout");
// /* handle the error */
//
// int result = logger_attach_sink("file /some/file/path")
// /* handle the error */
//
// int result = logger_apply();
// /* handle the error */
// ```

/// Initialize the thread-local logger with no sinks.
///
/// This initialized logger does nothing until `logger_apply` has been called.
#[no_mangle]
pub extern "C" fn logger_init() {
    set_logger(Dispatch::new());
}

/// Attach an additional sink to the thread-local logger.
///
/// This logger does nothing until `logger_apply` has been called.
#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn logger_attach_sink(
    sink_specifier: *const c_char,
    level_filter: LevelFilter,
) -> c_int {
    // Get the specifier from the raw C string.
    let sink_specifier = unsafe { CStr::from_ptr(sink_specifier) };
    let sink_specifier = match sink_specifier.to_str() {
        Ok(sink_specifier) => sink_specifier,
        Err(_) => return Status::SpecifierNotUtf8 as c_int,
    };

    // Attempt to construct a sink from the specifier.
    let sink = match Sink::try_from(sink_specifier) {
        Ok(sink) => sink,
        Err(err) => return Status::from(err) as c_int,
    };

    // Convert from our `#[repr(C)]` LevelFilter to the one from the `log` crate.
    let level_filter: LogLevelFilter = level_filter.into();

    // Construct a dispatcher from the sink and level filter.
    let dispatch = Into::<Dispatch>::into(sink)
        .level(level_filter)
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                record.level(),
                record.target(),
                message
            ))
        });

    // Take the existing logger, if there is one, add a new sink to it, and put it back.
    let status = match add_sink(dispatch) {
        Ok(_) => Status::Success,
        Err(err) => Status::from(err),
    };

    status as c_int
}

/// Apply the thread-local logger to the program.
///
/// Any attempts to modify the logger after the call to `logger_apply` will fail.
#[no_mangle]
pub extern "C" fn logger_apply() -> c_int {
    let status = match apply_logger() {
        Ok(_) => Status::Success,
        Err(err) => Status::from(err),
    };

    status as c_int
}
