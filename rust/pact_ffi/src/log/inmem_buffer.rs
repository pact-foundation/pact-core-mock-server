//! In-memory buffer for logging output.

use std::io;
use std::io::Write;
use tracing_subscriber::fmt::MakeWriter;

use pact_matching::logging::write_to_log_buffer;

/// In-memory buffer for logging output. Sends output to global static `LOG_BUFFER` in the pact_matching
/// crate. If there is a task local ID found, will accumulate against that ID, otherwise will
/// accumulate against the "global" ID.
#[derive(Debug, Copy, Clone)]
pub(crate) struct InMemBuffer { }

impl Write for InMemBuffer {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    write_to_log_buffer(buf);
    Ok(buf.len())
  }

  fn flush(&mut self) -> io::Result<()> {
    // no-op
    Ok(())
  }
}

impl InMemBuffer {
  /// Box this buffer
  pub(crate) fn boxed(self) -> Box<dyn Write + Send> {
    Box::new(self)
  }
}

impl <'a> MakeWriter<'a> for InMemBuffer {
  type Writer = InMemBuffer;

  fn make_writer(&'a self) -> Self::Writer {
    *self
  }
}
