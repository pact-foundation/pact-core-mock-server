//! The thread-local log dispatcher, which is cleared once applied.

// All of this module is `pub(crate)` and should not appear in the C header file
// or documentation.

use std::cell::RefCell;
use std::io::{stderr, stdout};

use anyhow::anyhow;
use log::{LevelFilter as LogLevelFilter, LevelFilter};
use tracing_log::AsTrace;
use tracing_subscriber::fmt::writer::{BoxMakeWriter, MakeWriterExt};
use tracing_subscriber::FmtSubscriber;
use tracing_subscriber::util::SubscriberInitExt;

use crate::log::sink::Sink;

thread_local! {
    // The thread-local logger. This is only populated during setup of the logger.
    /// cbindgen:ignore
    pub(crate) static LOGGER: RefCell<Vec<(String, LogLevelFilter)>> = RefCell::new(vec![]);
}

/// Initialise the data structure for a new logger-in-progress.
pub(crate) fn init_logger() {
    LOGGER.with(|logger| {
        *logger.borrow_mut() = vec![];
    });
}

/// Attach a sink to the logger-in-progress.
pub(crate) fn add_sink(sink_specifier: &str, level_filter: LogLevelFilter) -> anyhow::Result<()> {
    LOGGER.with(|logger_data| {
        let mut logger_inner = logger_data.borrow_mut();
        logger_inner.push((sink_specifier.to_string(), level_filter.clone()));
        Ok(())
    })
}

/// Apply the logger-in-progress as the global logger.
pub(crate) fn apply_logger() -> anyhow::Result<()> {
    LOGGER.with(|logger| {
      let mut logger_inner = logger.borrow_mut();

      let max_level = logger_inner.iter()
        .max_by(|a, b| a.1.cmp(&b.1))
        .map(|l| l.1)
        .unwrap_or(LogLevelFilter::Info);
      let subscriber_builder = FmtSubscriber::builder()
        .with_max_level(max_level.as_trace())
        .with_thread_names(true)
        .with_ansi(false) // Pact .Net can't deal with ANSI escape codes
      ;

      let subscriber = if let Some((sink, level)) = logger_inner.first() {
        let initial_writer = sink_to_make_writer(sink.as_str(), level);
        let writer = logger_inner.iter().skip(1).fold(initial_writer, |acc, (s, l)| {
          BoxMakeWriter::new(acc.and(sink_to_make_writer(s.as_str(), l)))
        });

        subscriber_builder.with_writer(writer).finish()
      } else {
        subscriber_builder.with_writer(BoxMakeWriter::new(stdout)).finish()
      };

      logger_inner.clear();
      subscriber.try_init().map_err(|err| anyhow!(err))
    })
}

fn sink_to_make_writer(sink: &str, level: &LevelFilter) -> BoxMakeWriter {
  // Safe to unwrap here, as the previous FFI step would have validated the sink and returned
  // an error back to the caller if the sink could not be constructed.
  let lvl = level.as_trace().into_level();
  match Sink::try_from(sink).unwrap() {
    Sink::Stdout(_) => {
      if let Some(lvl) = lvl {
        BoxMakeWriter::new(stdout.with_max_level(lvl))
      } else {
        BoxMakeWriter::new(stdout.with_filter(|_| false))
      }
    },
    Sink::Stderr(_) => {
      if let Some(lvl) = lvl {
        BoxMakeWriter::new(stderr.with_max_level(lvl))
      } else {
        BoxMakeWriter::new(stderr.with_filter(|_| false))
      }
    },
    Sink::File(f) => {
      if let Some(lvl) = lvl {
        BoxMakeWriter::new(f.with_max_level(lvl))
      } else {
        BoxMakeWriter::new(f.with_filter(|_| false))
      }
    },
    Sink::Buffer(b) => {
      if let Some(lvl) = lvl {
        BoxMakeWriter::new(b.with_max_level(lvl))
      } else {
        BoxMakeWriter::new(b.with_filter(|_| false))
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use log::LevelFilter;
  use tempfile::tempdir;

  use crate::log::logger::sink_to_make_writer;

  #[test]
  fn sink_to_make_writer_with_level_off() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.log");
    sink_to_make_writer("stdout", &LevelFilter::Off);
    sink_to_make_writer("stderr", &LevelFilter::Off);
    sink_to_make_writer("buffer", &LevelFilter::Off);
    let s = format!("file {}", file_path.to_string_lossy());
    sink_to_make_writer(s.as_str(), &LevelFilter::Off);
  }
}
