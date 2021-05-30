//! Thread local in-memory buffer for logging output

use std::io;
use std::io::Write;
use std::sync::Mutex;

use bytes::{BufMut, Bytes, BytesMut};
use lazy_static::lazy_static;

lazy_static! {
  // Memory buffer for the buffer logger. This is needed here because there is no
  // way to get the logger sync from the Dispatch struct. The buffer will be emptied
  // when the contents is fetched via an FFI call. A buffer is created per thread.
  /// cbindgen:ignore
  static ref BUFFER: Mutex<BytesMut> = Mutex::new(BytesMut::with_capacity(256));
}

/// Fetches the contents from the in-memory buffer and empties the buffer
pub(crate) fn fetch_buffer_contents() -> Bytes {
  let mut inner = BUFFER.lock().unwrap();
  inner.split().freeze()
}

/// In-memory thread local buffer for logging output. Sends output to `BUFFER`
#[derive(Debug)]
pub(crate) struct InMemBuffer { }

impl Write for InMemBuffer {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    let mut inner = BUFFER.lock().unwrap();
    inner.put(buf);
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
