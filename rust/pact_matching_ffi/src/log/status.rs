//! Status returned to the C caller for log FFI functions.

use crate::log::logger::LoggerError;
use crate::log::sink::SinkSpecifierError;
use log::SetLoggerError;

/// An enum representing the status codes which can be returned to the C caller.
pub(crate) enum Status {
    /// Opening a sink to the given file failed.
    CantOpenSinkToFile = -6,

    /// No file path was specified in the sink specification.
    MissingFilePath = -5,

    /// The sink type specified is not a known type.
    UnknownSinkType = -4,

    /// The sink specifier was not UTF-8 encoded.
    SpecifierNotUtf8 = -3,

    /// No logger has been initialized.
    NoLogger = -2,

    /// Can't set the logger
    CantSetLogger = -1,

    /// Operation succeeded.
    Success = 0,
}

impl From<SetLoggerError> for Status {
    fn from(_err: SetLoggerError) -> Status {
        Status::CantSetLogger
    }
}

impl From<SinkSpecifierError> for Status {
    fn from(err: SinkSpecifierError) -> Status {
        match err {
            SinkSpecifierError::UnknownSinkType { .. } => {
                Status::UnknownSinkType
            }
            SinkSpecifierError::MissingFilePath { .. } => {
                Status::MissingFilePath
            }
            SinkSpecifierError::CantMakeFile { .. } => {
                Status::CantOpenSinkToFile
            }
        }
    }
}

impl From<LoggerError> for Status {
    fn from(err: LoggerError) -> Status {
        match err {
            LoggerError::NoLogger => Status::NoLogger,
            LoggerError::ApplyLoggerFailed(err) => err.into(),
        }
    }
}
