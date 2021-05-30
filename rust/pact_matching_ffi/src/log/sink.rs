//! The sinks to which logs may be sent.

// All of this module is `pub(crate)` and should not appear in the C header file
// or documentation.

use std::convert::TryFrom;
use std::fmt::Debug;
use std::fs::File;
use std::io::{self, Stderr, Stdout};
use std::ops::Not;
use std::path::PathBuf;
use std::str::FromStr;

use fern::Dispatch;

use crate::log::inmem_buffer::InMemBuffer;

/// A sink for logs to be written to, based on a provider specifier.
#[derive(Debug)]
pub(crate) enum Sink {
    /// Write logs to stdout.
    Stdout(Stdout),

    /// Write logs to stderr.
    Stderr(Stderr),

    /// Write logs to a file.
    File(File),

    /// Write logs to a thread local memory buffer
    Buffer(InMemBuffer)
}

impl From<Sink> for Dispatch {
  fn from(sink: Sink) -> Dispatch {
    let dispatch = Dispatch::new();

    match sink {
      Sink::Stdout(stdout) => dispatch.chain(stdout),
      Sink::Stderr(stderr) => dispatch.chain(stderr),
      Sink::File(file) => dispatch.chain(file),
      Sink::Buffer(buffer) => dispatch.chain(buffer.boxed())
    }
  }
}

impl<'a> TryFrom<&'a str> for Sink {
  type Error = SinkSpecifierError;

  fn try_from(s: &'a str) -> Result<Sink, Self::Error> {
    let s = s.trim();

    match s {
      "stdout" => Ok(Sink::Stdout(io::stdout())),
      "stderr" => Ok(Sink::Stderr(io::stderr())),
      "buffer" => Ok(Sink::Buffer(InMemBuffer{})),
      _ => {
        let pat = "file ";
        if s.starts_with(pat).not() {
          Err(SinkSpecifierError::UnknownSinkType {
            name: s.to_owned(),
          })
        } else {
          match s.get(pat.len()..) {
            None => Err(SinkSpecifierError::MissingFilePath),
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
