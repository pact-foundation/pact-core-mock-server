//! Logging support. Provides a static store to accumulate log entries and a task local ID
//! to accumulate the entries against. If not task local value is available, uses a "global"
//! entry

use std::collections::HashMap;
use std::sync::Mutex;

use bytes::{BufMut, Bytes, BytesMut};
use lazy_static::lazy_static;
use tokio::task_local;

lazy_static! {
  /// Memory buffer for the buffer logger. This is needed here because there is no
  /// way to get the logger sync from the Dispatch struct. The buffer will be emptied
  /// when the contents is fetched via an FFI call.
  ///
  /// Accumulates the log entries against a task local ID. If the ID is not set, accumulates against
  /// the "global" ID.
  /// cbindgen:ignore
  static ref LOG_BUFFER: Mutex<HashMap<String, BytesMut>> = Mutex::new(HashMap::new());
}

task_local! {
  /// Log ID to accumulate logs against
  #[allow(missing_docs)]
  pub static LOG_ID: String;
}

/// Fetches the contents from the id scoped in-memory buffer and empties the buffer.
pub fn fetch_buffer_contents(id: &str) -> Bytes {
  let mut inner = LOG_BUFFER.lock().unwrap();
  let buffer = inner.entry(id.to_string())
    .or_insert_with(|| BytesMut::with_capacity(256));
  buffer.split().freeze()
}

/// Writes the provided bytes to the task local ID scoped in-memory buffer. If there is no
/// task local ID set, will write to the "global" buffer.
pub fn write_to_log_buffer(buf: &[u8]) {
  let id = LOG_ID.try_with(|id| id.clone()).unwrap_or_else(|_| "global".into());
  let mut inner = LOG_BUFFER.lock().unwrap();
  let buffer = inner.entry(id)
    .or_insert_with(|| BytesMut::with_capacity(256));
  buffer.put(buf);
}
