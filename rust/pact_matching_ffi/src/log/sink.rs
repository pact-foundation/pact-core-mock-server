//! The sinks to which logs may be sent.

use fern::Dispatch;
use std::convert::TryFrom;
use std::fs::File;
use std::io::{self, Stderr, Stdout};
use std::ops::Not;
use std::path::PathBuf;
use std::str::FromStr;

/// A sink for logs to be written to, based on a provider specifier.
#[derive(Debug)]
pub(crate) enum Sink {
    /// Write logs to stdout.
    Stdout(Stdout),

    /// Write logs to stderr.
    Stderr(Stderr),

    /// Write logs to a file.
    File(File),
}

impl Into<Dispatch> for Sink {
    fn into(self) -> Dispatch {
        let dispatch = Dispatch::new();

        match self {
            Sink::Stdout(stdout) => dispatch.chain(stdout),
            Sink::Stderr(stderr) => dispatch.chain(stderr),
            Sink::File(file) => dispatch.chain(file),
        }
    }
}

impl<'a> TryFrom<&'a str> for Sink {
    type Error = SinkSpecifierError;

    fn try_from(s: &'a str) -> Result<Sink, Self::Error> {
        let s = s.trim();

        if s == "stdout" {
            return Ok(Sink::Stdout(io::stdout()));
        } else if s == "stderr" {
            return Ok(Sink::Stderr(io::stderr()));
        }

        let pat = "file ";

        if s.starts_with(pat).not() {
            return Err(SinkSpecifierError::UnknownSinkType {
                name: s.to_owned(),
            });
        }

        match s.get(pat.len()..) {
            None => return Err(SinkSpecifierError::MissingFilePath),
            Some(remainder) => {
                match File::create(remainder) {
                    Ok(file) => Ok(Sink::File(file)),
                    Err(source) => {
                        // PANIC SAFETY: This `unwrap` is fine because the `PathBuf` impl of `FromStr` has an associated
                        // `Self::Error` type of `Infallible`, indicating the operation always succeeds.
                        let path = PathBuf::from_str(remainder).unwrap();
                        Err(SinkSpecifierError::CantMakeFile {
                            path,
                            source,
                        })
                    }
                }
            }
        }
    }
}

/// An error arising from attempting to parse a sink specifier string.
#[derive(Debug, thiserror::Error)]
pub(crate) enum SinkSpecifierError {
    #[error("unknown logger sink type (was '{name}', should be \"stdout\"/\"stderr\"/or \"file <file path>\")")]
    UnknownSinkType { name: String },

    #[error("missing path in file sink specifier")]
    MissingFilePath,

    #[error("can't make log sink file at path: '{path}'")]
    CantMakeFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
