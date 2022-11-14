//! FFI functions to deal with matching rules

use pact_models::matchingrules::MatchingRule;
use libc::c_char;

use crate::{ffi_fn, as_ref};
use crate::util::{ptr, string};

ffi_fn! {
  /// Get the JSON form of the matching rule.
  ///
  /// The returned string must be deleted with `pactffi_string_delete`.
  ///
  /// # Safety
  ///
  /// This function will fail if it is passed a NULL pointer, or the iterator that owns the
  /// value of the matching rule has been deleted.
  fn pactffi_matching_rule_json(rule: *const MatchingRule) -> *const c_char {
    let rule = as_ref!(rule);
    let json = rule.to_json().to_string();
    string::to_c(&json)? as *const c_char
  } {
    ptr::null_to::<c_char>()
  }
}

#[cfg(test)]
mod tests {
  use std::ffi::CString;

  use expectest::prelude::*;
  use libc::c_char;
  use pact_models::matchingrules::MatchingRule;

  use crate::models::matching_rules::pactffi_matching_rule_json;

  #[test]
  fn matching_rule_json() {
    let rule = MatchingRule::Regex("\\d+".to_string());
    let rule_ptr = &rule as *const MatchingRule;
    let json_ptr = pactffi_matching_rule_json(rule_ptr);
    let json = unsafe { CString::from_raw(json_ptr as *mut c_char) };
    expect!(json.to_string_lossy()).to(be_equal_to("{\"match\":\"regex\",\"regex\":\"\\\\d+\"}"));
  }
}
