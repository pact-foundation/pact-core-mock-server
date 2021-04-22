//! The public FFI functions for initializing, adding sinks to, and applying a logger.

use crate::error::set_error_msg;
use crate::log::level_filter::LevelFilter;
use crate::log::logger::{add_sink, apply_logger, set_logger};
use crate::log::sink::Sink;
use crate::log::status::Status;
use fern::Dispatch;
use libc::{c_char, c_int};
use log::LevelFilter as LogLevelFilter;
use std::convert::TryFrom;
use std::ffi::{CStr, CString};

/// Convenience function to direct all logging to stdout.
#[no_mangle]
pub extern "C" fn log_to_stdout(level_filter: LevelFilter) -> c_int {
    logger_init();

    let spec = match CString::new("stdout") {
        Ok(spec) => spec,
        Err(e) => {
            set_error_msg(e.to_string());
            return Status::CantConstructSink as c_int;
        }
    };

    let status = logger_attach_sink(spec.as_ptr(), level_filter);
    if status != 0 {
        return status;
    }

    let status = logger_apply();
    if status != 0 {
        return status;
    }

    Status::Success as c_int
}

/// Convenience function to direct all logging to stderr.
#[no_mangle]
pub extern "C" fn log_to_stderr(level_filter: LevelFilter) -> c_int {
    logger_init();

    let spec = match CString::new("stderr") {
        Ok(spec) => spec,
        Err(e) => {
            set_error_msg(e.to_string());
            return Status::CantConstructSink as c_int;
        }
    };

    let status = logger_attach_sink(spec.as_ptr(), level_filter);
    if status != 0 {
        return status;
    }

    let status = logger_apply();
    if status != 0 {
        return status;
    }

    Status::Success as c_int
}

/// Convenience function to direct all logging to a file.
#[no_mangle]
pub extern "C" fn log_to_file(
    file_name: *const c_char,
    level_filter: LevelFilter,
) -> c_int {
    logger_init();

    let spec = {
        if file_name.is_null() {
            return Status::CantConstructSink as c_int;
        }

        let file_name =
            match unsafe { CStr::from_ptr(file_name) }.to_str() {
                Ok(file_name) => file_name,
                Err(e) => {
                    set_error_msg(e.to_string());
                    return Status::CantConstructSink as c_int;
                }
            };

        let spec = format!("file {}", file_name);

        match CString::new(spec) {
            Ok(spec) => spec,
            Err(e) => {
                set_error_msg(e.to_string());
                return Status::CantConstructSink as c_int;
            }
        }
    };

    let status = logger_attach_sink(spec.as_ptr(), level_filter);
    if status != 0 {
        return status;
    }

    let status = logger_apply();
    if status != 0 {
        return status;
    }

    Status::Success as c_int
}

// C API uses something like the pledge API to select write locations, including:
//
// * stdout (`logger_attach_sink("stdout", LevelFilter_Info)`)
// * stderr (`logger_attach_sink("stderr", LevelFilter_Debug)`)
// * file w/ file path (`logger_attach_sink("file /some/file/path", LevelFilter_Trace)`)
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
// int result = logger_attach_sink("stderr", FilterLevel_Debug);
// /* handle the error */
//
// int result = logger_attach_sink("file /some/file/path", FilterLevel_Info);
// /* handle the error */
//
// int result = logger_apply();
// /* handle the error */
// ```

/// Initialize the thread-local logger with no sinks.
///
/// This initialized logger does nothing until `logger_apply` has been called.
///
/// # Usage
///
/// ```c
/// logger_init();
/// ```
///
/// # Safety
///
/// This function is always safe to call.
#[no_mangle]
pub extern "C" fn logger_init() {
    set_logger(Dispatch::new());
}

/// Attach an additional sink to the thread-local logger.
///
/// This logger does nothing until `logger_apply` has been called.
///
/// Three types of sinks can be specified:
///
/// - stdout (`logger_attach_sink("stdout", LevelFilter_Info)`)
/// - stderr (`logger_attach_sink("stderr", LevelFilter_Debug)`)
/// - file w/ file path (`logger_attach_sink("file /some/file/path", LevelFilter_Trace)`)
///
/// # Usage
///
/// ```
/// int result = logger_attach_sink("file /some/file/path", LogLevel_Filter);
/// ```
///
/// # Error Handling
///
/// The return error codes are as follows:
///
/// - `-1`: Can't set logger (applying the logger failed, perhaps because one is applied already).
/// - `-2`: No logger has been initialized (call `logger_init` before any other log function).
/// - `-3`: The sink specifier was not UTF-8 encoded.
/// - `-4`: The sink type specified is not a known type (known types: "stdout", "stderr", or "file /some/path").
/// - `-5`: No file path was specified in a file-type sink specification.
/// - `-6`: Opening a sink to the specified file path failed (check permissions).
///
/// # Safety
///
/// This function checks the validity of the passed-in sink specifier, and errors
/// out if the specifier isn't valid UTF-8.
#[allow(clippy::missing_safety_doc)]
#[allow(clippy::not_unsafe_ptr_args_deref)]
#[no_mangle]
pub extern "C" fn logger_attach_sink(
    sink_specifier: *const c_char,
    level_filter: LevelFilter,
) -> c_int {
    // Get the specifier from the raw C string.
    let sink_specifier = unsafe { CStr::from_ptr(sink_specifier) };
    let sink_specifier = match sink_specifier.to_str() {
        Ok(sink_specifier) => sink_specifier,
        // TODO: Permit non-UTF8 strings, as some filesystems may have non-UTF8
        //       paths to which the user wants to direct the logging output.
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
