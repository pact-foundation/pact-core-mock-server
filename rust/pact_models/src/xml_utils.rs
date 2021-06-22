//! Collection of utilities for working with XML

use sxd_document::*;
use std::str;

/// Parses a vector of bytes into a XML document
pub fn parse_bytes(bytes: &[u8]) -> Result<Package, String> {
  let string = str::from_utf8(bytes).map_err(|_| format!("{:?}", bytes))?;
  parser::parse(string).map_err(|e| format!("{:?}", e))
}
