//! V4 specification models

use std::fmt::{Display, Formatter};
use std::fmt;

use anyhow::anyhow;

/// V4 Interaction Type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum V4InteractionType {
  /// Synchronous HTTP Request Response
  Synchronous_HTTP,
  /// Asynchronous Messages
  Asynchronous_Messages,
  /// Synchronous Messages in the form Request message -> Response messages
  Synchronous_Messages
}

impl Default for V4InteractionType {
  fn default() -> Self {
    V4InteractionType::Synchronous_HTTP
  }
}

impl Display for V4InteractionType {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match *self {
      V4InteractionType::Synchronous_HTTP => f.write_str("Synchronous/HTTP"),
      V4InteractionType::Asynchronous_Messages => f.write_str("Asynchronous/Messages"),
      V4InteractionType::Synchronous_Messages => f.write_str("Synchronous/Messages")
    }
  }
}

impl V4InteractionType {
  /// Returns the V4 interaction type from the string value
  pub fn from_str(type_str: &str) -> anyhow::Result<V4InteractionType> {
    match type_str {
      "Synchronous/HTTP" => Ok(V4InteractionType::Synchronous_HTTP),
      "Asynchronous/Messages" => Ok(V4InteractionType::Asynchronous_Messages),
      "Synchronous/Messages" => Ok(V4InteractionType::Synchronous_Messages),
      _ => Err(anyhow!("'{}' is not a valid V4 interaction type", type_str))
    }
  }
}
