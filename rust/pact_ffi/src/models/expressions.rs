//! Functions for dealing with matching rule expressions

use anyhow::Context;
use either::Either;
use libc::c_char;
use pact_models::generators::Generator;
use pact_models::matchingrules::expressions::{
  is_matcher_def,
  MatchingRuleDefinition,
  parse_matcher_def,
  ValueType
};
use tracing::{debug, error};

use crate::{as_ref, ffi_fn, safe_str};
use crate::util::{ptr, string};

/// Result of parsing a matching rule definition
#[derive(Debug, Clone)]
pub struct MatchingRuleDefinitionResult {
  result: Either<String, MatchingRuleDefinition>
}

ffi_fn! {
  /// Parse a matcher definition string into a MatchingRuleDefinition containing the example value,
  /// and matching rules and any generator.
  ///
  /// The following are examples of matching rule definitions:
  /// * `matching(type,'Name')` - type matcher with string value 'Name'
  /// * `matching(number,100)` - number matcher
  /// * `matching(datetime, 'yyyy-MM-dd','2000-01-01')` - datetime matcher with format string
  ///
  /// See [Matching Rule definition expressions](https://docs.rs/pact_models/latest/pact_models/matchingrules/expressions/index.html).
  ///
  /// The returned value needs to be freed up with the `pactffi_matcher_definition_delete` function.
  ///
  /// # Errors
  /// If the expression is invalid, the MatchingRuleDefinition error will be set. You can check for
  /// this value with the `pactffi_matcher_definition_error` function.
  ///
  /// # Safety
  ///
  /// This function is safe if the expression is a valid NULL terminated string pointer.
  fn pactffi_parse_matcher_definition(expression: *const c_char) -> *const MatchingRuleDefinitionResult {
    let expression = safe_str!(expression);
    let result = if is_matcher_def(expression) {
      match parse_matcher_def(expression) {
        Ok(definition) => {
          debug!("Parsed matcher definition '{}' to '{:?}'", expression, definition);
          MatchingRuleDefinitionResult {
            result: Either::Right(definition)
          }
        }
        Err(err) => {
          error!("Failed to parse matcher definition '{}': {}", expression, err);
          MatchingRuleDefinitionResult {
            result: Either::Left(err.to_string())
          }
        }
      }
    } else if expression.is_empty() {
      MatchingRuleDefinitionResult {
        result: Either::Left("Expected a matching rule definition, but got an empty string".to_string())
      }
    } else {
      MatchingRuleDefinitionResult {
        result: Either::Right(MatchingRuleDefinition {
          value: expression.to_string(),
          value_type: ValueType::String,
          rules: vec![],
          generator: None
        })
      }
    };

    ptr::raw_to(result) as *const MatchingRuleDefinitionResult
  } {
    ptr::null_to::<MatchingRuleDefinitionResult>()
  }
}

ffi_fn! {
  /// Returns any error message from parsing a matching definition expression. If there is no error,
  /// it will return a NULL pointer, otherwise returns the error message as a NULL-terminated string.
  /// The returned string must be freed using the `pactffi_string_delete` function once done with it.
  fn pactffi_matcher_definition_error(definition: *const MatchingRuleDefinitionResult) -> *const c_char {
    let definition = as_ref!(definition);
    if let Either::Left(error) = &definition.result {
      string::to_c(&error)? as *const c_char
    } else {
      ptr::null_to::<c_char>()
    }
  } {
    ptr::null_to::<c_char>()
  }
}

ffi_fn! {
  /// Returns the value from parsing a matching definition expression. If there was an error,
  /// it will return a NULL pointer, otherwise returns the value as a NULL-terminated string.
  /// The returned string must be freed using the `pactffi_string_delete` function once done with it.
  ///
  /// Note that different expressions values can have types other than a string. Use
  /// `pactffi_matcher_definition_value_type` to get the actual type of the value. This function
  /// will always return the string representation of the value.
  fn pactffi_matcher_definition_value(definition: *const MatchingRuleDefinitionResult) -> *const c_char {
    let definition = as_ref!(definition);
    if let Either::Right(definition) = &definition.result {
      string::to_c(&definition.value)? as *const c_char
    } else {
      ptr::null_to::<c_char>()
    }
  } {
    ptr::null_to::<c_char>()
  }
}

ffi_fn! {
  /// Frees the memory used by the result of parsing the matching definition expression
  fn pactffi_matcher_definition_delete(definition: *const MatchingRuleDefinitionResult) {
    ptr::drop_raw(definition as *mut MatchingRuleDefinitionResult);
  }
}

ffi_fn! {
  /// Returns the generator from parsing a matching definition expression. If there was an error or
  /// there is no associated generator, it will return a NULL pointer, otherwise returns the generator
  /// as a pointer.
  fn pactffi_matcher_definition_generator(definition: *const MatchingRuleDefinitionResult) -> *const Generator {
    let definition = as_ref!(definition);
    if let Either::Right(definition) = &definition.result {
      if let Some(generator) = &definition.generator {
        generator as *const Generator
      } else {
        ptr::null_to::<Generator>()
      }
    } else {
      ptr::null_to::<Generator>()
    }
  } {
    ptr::null_to::<Generator>()
  }
}

/// The type of value detected after parsing the expression
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExpressionValueType {
  /// If the type is unknown
  Unknown,
  /// String type
  String,
  /// Numeric type
  Number,
  /// Integer numeric type (no significant figures after the decimal point)
  Integer,
  /// Decimal numeric type (at least one significant figure after the decimal point)
  Decimal,
  /// Boolean type
  Boolean
}

impl ExpressionValueType {
  /// Convert the ValueType into an ExpressionValueType which can be returned via FFI
  fn from_value_type(t: ValueType) -> ExpressionValueType {
    match t {
      ValueType::Unknown => ExpressionValueType::Unknown,
      ValueType::String => ExpressionValueType::String,
      ValueType::Number => ExpressionValueType::Number,
      ValueType::Integer => ExpressionValueType::Integer,
      ValueType::Decimal => ExpressionValueType::Decimal,
      ValueType::Boolean => ExpressionValueType::Boolean
    }
  }
}

ffi_fn! {
  /// Returns the type of the value from parsing a matching definition expression. If there was an
  /// error parsing the expression, it will return Unknown.
  fn pactffi_matcher_definition_value_type(definition: *const MatchingRuleDefinitionResult) -> ExpressionValueType {
    let definition = as_ref!(definition);
    if let Either::Right(definition) = &definition.result {
      ExpressionValueType::from_value_type(definition.value_type)
    } else {
      ExpressionValueType::Unknown
    }
  } {
    ExpressionValueType::Unknown
  }
}

#[cfg(test)]
mod tests {
  use std::ffi::CString;

  use expectest::prelude::*;
  use libc::c_char;

  use crate::models::expressions::{
    ExpressionValueType,
    MatchingRuleDefinitionResult,
    pactffi_matcher_definition_error,
    pactffi_matcher_definition_generator,
    pactffi_matcher_definition_value,
    pactffi_matcher_definition_value_type,
    pactffi_parse_matcher_definition
  };
  use crate::util::ptr;

  #[test_log::test]
  fn parse_expression_with_null() {
    let result = pactffi_parse_matcher_definition(ptr::null_to());
    expect!(result.is_null()).to(be_true());
  }

  #[test_log::test]
  fn parse_expression_with_empty_string() {
    let empty = CString::new("").unwrap();
    let result = pactffi_parse_matcher_definition(empty.as_ptr());
    expect!(result.is_null()).to(be_false());

    let error = pactffi_matcher_definition_error(result);
    let string = unsafe { CString::from_raw(error as *mut c_char) };
    expect!(string.to_string_lossy()).to(be_equal_to("Expected a matching rule definition, but got an empty string"));

    let definition = unsafe { Box::from_raw(result as *mut MatchingRuleDefinitionResult) };
    expect!(definition.result.left()).to(be_some().value("Expected a matching rule definition, but got an empty string"));
  }

  #[test_log::test]
  fn parse_expression_with_invalid_expression() {
    let value = CString::new("matching(type,").unwrap();
    let result = pactffi_parse_matcher_definition(value.as_ptr());
    expect!(result.is_null()).to(be_false());

    let error = pactffi_matcher_definition_error(result);
    let string = unsafe { CString::from_raw(error as *mut c_char) };
    expect!(string.to_string_lossy()).to(be_equal_to("expected a primitive value"));

    let value = pactffi_matcher_definition_value(result);
    expect!(value.is_null()).to(be_true());

    let generator = pactffi_matcher_definition_generator(result);
    expect!(generator.is_null()).to(be_true());

    let value_type = pactffi_matcher_definition_value_type(result);
    expect!(value_type).to(be_equal_to(ExpressionValueType::Unknown));

    let definition = unsafe { Box::from_raw(result as *mut MatchingRuleDefinitionResult) };
    expect!(definition.result.left()).to(be_some().value("expected a primitive value"));
  }

  #[test_log::test]
  fn parse_expression_with_valid_expression() {
    let value = CString::new("matching(type,'Name')").unwrap();
    let result = pactffi_parse_matcher_definition(value.as_ptr());
    expect!(result.is_null()).to(be_false());

    let error = pactffi_matcher_definition_error(result);
    expect!(error.is_null()).to(be_true());

    let value = pactffi_matcher_definition_value(result);
    expect!(value.is_null()).to(be_false());
    let string = unsafe { CString::from_raw(value as *mut c_char) };
    expect!(string.to_string_lossy()).to(be_equal_to("Name"));

    let generator = pactffi_matcher_definition_generator(result);
    expect!(generator.is_null()).to(be_true());

    let value_type = pactffi_matcher_definition_value_type(result);
    expect!(value_type).to(be_equal_to(ExpressionValueType::String));

    let definition = unsafe { Box::from_raw(result as *mut MatchingRuleDefinitionResult) };
    expect!(definition.result.as_ref().left()).to(be_none());
    expect!(definition.result.as_ref().right()).to(be_some());
  }

  #[test_log::test]
  fn parse_expression_with_normal_string() {
    let value = CString::new("I am not an expression").unwrap();
    let result = pactffi_parse_matcher_definition(value.as_ptr());
    expect!(result.is_null()).to(be_false());

    let error = pactffi_matcher_definition_error(result);
    expect!(error.is_null()).to(be_true());

    let value = pactffi_matcher_definition_value(result);
    expect!(value.is_null()).to(be_false());
    let string = unsafe { CString::from_raw(value as *mut c_char) };
    expect!(string.to_string_lossy()).to(be_equal_to("I am not an expression"));

    let value_type = pactffi_matcher_definition_value_type(result);
    expect!(value_type).to(be_equal_to(ExpressionValueType::String));

    let definition = unsafe { Box::from_raw(result as *mut MatchingRuleDefinitionResult) };
    expect!(definition.result.as_ref().left()).to(be_none());
    expect!(definition.result.as_ref().right()).to(be_some());
    expect!(definition.result.as_ref().right().unwrap().rules.is_empty()).to(be_true());
  }

  #[test_log::test]
  fn parse_expression_with_generator() {
    let value = CString::new("matching(date,'yyyy-MM-dd', '2000-01-02')").unwrap();
    let result = pactffi_parse_matcher_definition(value.as_ptr());
    expect!(result.is_null()).to(be_false());

    let error = pactffi_matcher_definition_error(result);
    expect!(error.is_null()).to(be_true());

    let value = pactffi_matcher_definition_value(result);
    expect!(value.is_null()).to(be_false());
    let string = unsafe { CString::from_raw(value as *mut c_char) };
    expect!(string.to_string_lossy()).to(be_equal_to("2000-01-02"));

    let generator = pactffi_matcher_definition_generator(result);
    expect!(generator.is_null()).to(be_false());

    let value_type = pactffi_matcher_definition_value_type(result);
    expect!(value_type).to(be_equal_to(ExpressionValueType::String));

    let definition = unsafe { Box::from_raw(result as *mut MatchingRuleDefinitionResult) };
    expect!(definition.result.as_ref().left()).to(be_none());
    expect!(definition.result.as_ref().right()).to(be_some());
  }
}
