//! Defines an alias for the type returned by `std::panic::catch_unwind`.

use crate::error::error_msg::ErrorMsg;
use std::any::Any;

/// The error type returned by `std::panic::catch_unwind`.
pub(crate) type AnyError = Box<dyn Any + Send + 'static>;

/// An extension trait for extracting an error message out of an `AnyError`.
pub(crate) trait ToErrorMsg {
    fn to_error_msg(self) -> String;
}

impl ToErrorMsg for AnyError {
    /// This works with an `AnyError` taken from `std::panic::catch_unwind`,
    /// attempts to extract an error message out of it by constructing the
    /// `ErrorMsg` type, and then converts that to a string, which is passed
    /// to `update_last_error`.
    ///
    /// Note that if an error message can't be extracted from the `AnyError`,
    /// there will still be an update to the `LAST_ERROR`, reporting that an
    /// unknown error occurred.
    fn to_error_msg(self) -> String {
        ErrorMsg::from(self).to_string()
    }
}
