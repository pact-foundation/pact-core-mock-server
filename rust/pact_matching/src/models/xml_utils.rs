use sxd_document::*;
use std::str;

pub fn parse_bytes(bytes: &Vec<u8>) -> Result<Package, String> {
  let string = str::from_utf8(bytes).map_err(|_| format!("{:?}", bytes))?;
  parser::parse(string).map_err(|e| format!("{:?}", e))
}
