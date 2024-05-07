//! Collection of utilities for working with XML

use std::str;

use anyhow::anyhow;
use sxd_document::*;

/// Parses a vector of bytes into a XML document
pub fn parse_bytes(bytes: &[u8]) -> anyhow::Result<Package> {
  let string = str::from_utf8(bytes)?;
  match parser::parse(string) {
    Ok(doc) => Ok(doc),
    Err(err) => Err(anyhow!("Failed to parse bytes as XML - {}", err))
  }
}
