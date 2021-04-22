//! The thread-local log dispatcher, which is cleared once applied.

// All of this module is `pub(crate)` and should not appear in the C header file
// or documentation.

use fern::Dispatch;
use log::SetLoggerError;
use std::cell::RefCell;

thread_local! {
    // The thread-local logger. This is only populated during setup of the logger.
    /// cbindgen:ignore
    pub(crate) static LOGGER: RefCell<Option<Dispatch>> = RefCell::new(None);
}

/// Set a new dispatcher as the logger-in-progress.
pub(crate) fn set_logger(dispatch: Dispatch) {
    LOGGER.with(|logger| {
        *logger.borrow_mut() = Some(dispatch);
    });
}

/// Attach a sink to the logger-in-progress.
pub(crate) fn add_sink(dispatch: Dispatch) -> Result<(), LoggerError> {
    match LOGGER.with(|logger| logger.borrow_mut().take()) {
        None => Err(LoggerError::NoLogger),
        Some(top_level_dispatch) => {
            let top_level_dispatch = top_level_dispatch.chain(dispatch);

            LOGGER.with(|logger| {
                *logger.borrow_mut() = Some(top_level_dispatch)
            });

            Ok(())
        }
    }
}

/// Apply the logger-in-progress as the global logger.
pub(crate) fn apply_logger() -> Result<(), LoggerError> {
    match LOGGER.with(|logger| logger.borrow_mut().take()) {
        Some(logger) => Ok(logger.apply()?),
        None => Err(LoggerError::NoLogger),
    }
}

/// An error arising from initializing, populating, and applying the logger.
#[derive(Debug, thiserror::Error)]
pub(crate) enum LoggerError {
    #[error("no logger initialized")]
    NoLogger,

    #[error(transparent)]
    ApplyLoggerFailed(#[from] SetLoggerError),
}
