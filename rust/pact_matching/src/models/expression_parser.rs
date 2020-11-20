//! expression parser for generator expressions

use serde_json::{Value, json};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use nom::lib::std::convert::TryFrom;
use crate::models::json_utils::json_to_string;

/// Data type to cast to for provider state context values
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub enum DataType {
  /// String values
  STRING,
  /// Integer values
  INTEGER,
  /// Decimal values
  DECIMAL,
  /// Floating point values
  FLOAT,
  /// Don't convert values
  RAW,
  /// Boolean values
  BOOLEAN
}

impl DataType {
  /// Wraps the generated value in a DataValue
  pub fn wrap(&self, result: Result<String, String>) -> Result<DataValue, String> {
    result.map(|val| DataValue { generated: val, data_type: self.clone() })
  }
}

impl From<Value> for DataType {
  fn from(v: Value) -> Self {
    match v {
      Value::String(s) => match s.to_ascii_uppercase().as_str() {
        "STRING" => DataType::STRING,
        "INTEGER" => DataType::INTEGER,
        "DECIMAL" => DataType::DECIMAL,
        "FLOAT" => DataType::FLOAT,
        "BOOLEAN" => DataType::BOOLEAN,
        _ => DataType::RAW
      }
      _ => DataType::RAW
    }
  }
}

/// Data Value container for a generated value
pub struct DataValue {
  /// Original generated value
  pub generated: String,
  /// Data type to cast it as
  pub data_type: DataType
}

impl DataValue {
  /// Convert this data value to JSON using the associated data type
  pub fn as_json(&self) -> Result<Value, String> {
    match self.data_type {
      DataType::STRING => Ok(Value::String(self.generated.clone())),
      DataType::INTEGER => self.generated.parse::<usize>().map(|val| json!(val))
        .map_err(|err| format!("Number can not be generated from '{}' - {}", self.generated, err)),
      DataType::FLOAT | DataType::DECIMAL => self.generated.parse::<f64>().map(|val| json!(val))
        .map_err(|err| format!("Floating point number can not be generated from '{}' - {}", self.generated, err)),
      DataType::RAW => Ok(Value::String(self.generated.clone())),
      DataType::BOOLEAN => self.generated.parse::<bool>().map(|val| json!(val))
        .map_err(|err| format!("Boolean can not be generated from '{}' - {}", self.generated, err))
    }
  }
}

impl TryFrom<DataValue> for u16 {
  type Error = String;

  fn try_from(value: DataValue) -> Result<Self, Self::Error> {
    match value.data_type {
      DataType::INTEGER | DataType::RAW =>
        value.generated.parse::<u16>().map_err(|err| format!("Number can not be generated from '{}' - {}", value.generated, err)),
      DataType::DECIMAL | DataType::FLOAT =>
        value.generated.parse::<f64>()
          .map(|val| val as u16)
          .map_err(|err| format!("u16 can not be generated from '{}' - {}", value.generated, err)),
      _ => Err(format!("u16 can not be generated from {}", value.generated))
    }
  }
}

impl TryFrom<DataValue> for f64 {
  type Error = String;

  fn try_from(value: DataValue) -> Result<Self, Self::Error> {
    match value.data_type {
      DataType::INTEGER =>
        value.generated.parse::<usize>()
          .map(|val| val as f64)
          .map_err(|err| format!("Floating point number can not be generated from '{}' - {}", value.generated, err)),
      DataType::DECIMAL | DataType::FLOAT | DataType::RAW =>
        value.generated.parse::<f64>().map_err(|err| format!("Floating point number can not be generated from '{}' - {}", value.generated, err)),
      _ => Err(format!("Floating point number can not be generated from {}", value.generated))
    }
  }
}

impl TryFrom<DataValue> for String {
  type Error = String;

  fn try_from(value: DataValue) -> Result<Self, Self::Error> {
    match value.data_type {
      DataType::INTEGER | DataType::DECIMAL =>
        value.generated.parse::<i64>().map(|val| val.to_string())
          .map_err(|err| format!("Number can not be generated from '{}' - {}", value.generated, err)),
      DataType::FLOAT =>
        value.generated.parse::<f64>().map(|val| val.to_string())
          .map_err(|err| format!("Number can not be generated from '{}' - {}", value.generated, err)),
      DataType::BOOLEAN =>
        value.generated.parse::<bool>().map(|val| val.to_string())
          .map_err(|err| format!("Boolean can not be generated from '{}' - {}", value.generated, err)),
      DataType::RAW => Ok(value.generated.clone()),
      DataType::STRING => Ok(value.generated.clone())
    }
  }
}

/// Trait for resolvers of values
pub trait ValueResolver {
  /// Resolves the value by looking it up in the context
  fn resolve_value(&self, name: &str) -> Option<String>;
}

/// Value resolver that looks a value up from a Map
#[derive(PartialEq, Debug, Clone)]
pub struct MapValueResolver<'a> {
  /// Map to resolve values from
  pub context: HashMap<&'a str, Value>
}

impl ValueResolver for MapValueResolver<'_> {
  fn resolve_value(&self, name: &str) -> Option<String> {
    self.context.get(name.into()).map(|val| json_to_string(val))
  }
}

/// If the String value contains any expression within it
pub fn contains_expressions(value: &str) -> bool {
  value.contains("${")
}

/// Parse the expressions and return the generated value
pub fn parse_expression(value: &str, value_resolver: &dyn ValueResolver) -> Result<String, String> {
  if contains_expressions(value) {
    replace_expressions(value, value_resolver)
  } else {
    Ok(value.to_string())
  }
}

fn replace_expressions(value: &str, value_resolver: &dyn ValueResolver) -> Result<String, String> {
  let mut result = String::default();
  let mut buffer = value;
  let mut position = buffer.find("${");
  while let Some(index) = position {
    result.push_str(&buffer[0..index]);
    let end_position = buffer.find('}')
      .ok_or(format!("Missing closing brace in expression string '{}'", value))?;
    if end_position - index > 2 {
      if let Some(lookup) = value_resolver.resolve_value(&buffer[(index + 2)..end_position]) {
        result.push_str(lookup.as_str());
      }
    }
    buffer = &buffer[(end_position + 1)..];
    position = buffer.find("${");
  }
  result.push_str(buffer);
  Ok(result)
}

#[cfg(test)]
mod tests {
  use super::*;
  use expectest::prelude::*;
  use maplit::hashmap;

  struct NullResolver;

  impl ValueResolver for NullResolver {
    fn resolve_value(&self, _: &str) -> Option<String> {
      None
    }
  }

  #[test]
  fn does_not_modify_strings_with_no_expressions() {
    expect!(parse_expression(&"this is not an expression".to_string(), &NullResolver)).to(be_ok().value("this is not an expression".to_string()));
    expect!(parse_expression(&"".to_string(), &NullResolver)).to(be_ok().value("".to_string()));
    expect!(parse_expression(&"looks like a $".to_string(), &NullResolver)).to(be_ok().value("looks like a $".to_string()));
  }

  #[test]
  fn parse_expression_with_an_expression() {
    let resolver = MapValueResolver { context: hashmap!{ "a" => json!("A") } };
    expect!(parse_expression(&"${a}".to_string(), &resolver)).to(be_ok().value("A".to_string()));
  }

  #[test]
  fn contains_expressions_test() {
    expect!(contains_expressions(&"${a}".to_string())).to(be_true());
    expect!(contains_expressions(&"$a".to_string())).to(be_false());
    expect!(contains_expressions(&"".to_string())).to(be_false());
    expect!(contains_expressions(&"this is not an expression".to_string())).to(be_false());
    expect!(contains_expressions(&"this ${is} an expression".to_string())).to(be_true());
  }

  #[test]
  fn returns_an_error_on_unterminated_expressions() {
    expect!(parse_expression(&"invalid ${a expression".to_string(), &NullResolver)).to(
      be_err().value("Missing closing brace in expression string \'invalid ${a expression\'".to_string()));
  }

  #[test]
  fn handles_empty_expression() {
    expect!(parse_expression(&"${}".to_string(), &NullResolver)).to(be_ok().value("".to_string()));
    expect!(parse_expression(&"${} ${} ${}".to_string(), &NullResolver)).to(be_ok().value("  ".to_string()));
  }

  #[test]
  fn replaces_the_expression_with_resolved_value() {
    let resolver = MapValueResolver { context: hashmap!{ "value" => json!("[value]") } };
    expect!(parse_expression(&"${value}".to_string(), &resolver)).to(be_ok().value("[value]".to_string()));
    expect!(parse_expression(&" ${value}".to_string(), &resolver)).to(be_ok().value(" [value]".to_string()));
    expect!(parse_expression(&"${value} ".to_string(), &resolver)).to(be_ok().value("[value] ".to_string()));
    expect!(parse_expression(&" ${value} ".to_string(), &resolver)).to(be_ok().value(" [value] ".to_string()));
    expect!(parse_expression(&" ${value} ${value} ".to_string(), &resolver)).to(be_ok().value(" [value] [value] ".to_string()));
    expect!(parse_expression(&"$${value}}".to_string(), &resolver)).to(be_ok().value("$[value]}".to_string()));
  }

  #[test]
  fn with_a_defined_type_converts_the_expression_into_the_correct_type() {
    expect!(String::try_from(DataValue { generated: "string".into(), data_type: DataType::RAW })).to(be_ok().value("string".to_string()));
    expect!(String::try_from(DataValue { generated: "string".into(), data_type: DataType::STRING })).to(be_ok().value("string".to_string()));
    expect!(String::try_from(DataValue { generated: "100".into(), data_type: DataType::RAW })).to(be_ok().value("100".to_string()));
    expect!(String::try_from(DataValue { generated: "100".into(), data_type: DataType::STRING })).to(be_ok().value("100".to_string()));
    expect!(String::try_from(DataValue { generated: "100".into(), data_type: DataType::INTEGER })).to(be_ok().value("100".to_string()));
    expect!(String::try_from(DataValue { generated: "100".into(), data_type: DataType::FLOAT })).to(be_ok().value("100".to_string()));

    expect!(u16::try_from(DataValue { generated: "100".into(), data_type: DataType::RAW })).to(be_ok().value(100));
    expect!(u16::try_from(DataValue { generated: "100".into(), data_type: DataType::INTEGER })).to(be_ok().value(100));
    expect!(u16::try_from(DataValue { generated: "100.6".into(), data_type: DataType::INTEGER })).to(be_err());
    expect!(u16::try_from(DataValue { generated: "100".into(), data_type: DataType::FLOAT })).to(be_ok().value(100));
    expect!(u16::try_from(DataValue { generated: "100.6".into(), data_type: DataType::FLOAT })).to(be_ok().value(100));
    expect!(u16::try_from(DataValue { generated: "100".into(), data_type: DataType::STRING })).to(be_err());

    expect!(f64::try_from(DataValue { generated: "100.6".into(), data_type: DataType::RAW })).to(be_ok().value(100.6));
    expect!(f64::try_from(DataValue { generated: "100.6".into(), data_type: DataType::INTEGER })).to(be_err());
    expect!(f64::try_from(DataValue { generated: "100.6".into(), data_type: DataType::FLOAT })).to(be_ok().value(100.6));
    expect!(f64::try_from(DataValue { generated: "100.6".into(), data_type: DataType::STRING })).to(be_err());

    expect!(DataValue { generated: "string".into(), data_type: DataType::RAW }.as_json()).to(be_ok().value(json!("string")));
    expect!(DataValue { generated: "string".into(), data_type: DataType::STRING }.as_json()).to(be_ok().value(json!("string")));
    expect!(DataValue { generated: "string".into(), data_type: DataType::INTEGER }.as_json()).to(be_err());
    expect!(DataValue { generated: "string".into(), data_type: DataType::FLOAT }.as_json()).to(be_err());
    expect!(DataValue { generated: "string".into(), data_type: DataType::DECIMAL }.as_json()).to(be_err());
    expect!(DataValue { generated: "string".into(), data_type: DataType::BOOLEAN }.as_json()).to(be_err());

    expect!(DataValue { generated: "100".into(), data_type: DataType::RAW }.as_json()).to(be_ok().value(json!("100")));
    expect!(DataValue { generated: "100".into(), data_type: DataType::STRING }.as_json()).to(be_ok().value(json!("100")));
    expect!(DataValue { generated: "100".into(), data_type: DataType::INTEGER }.as_json()).to(be_ok().value(json!(100)));
    expect!(DataValue { generated: "100".into(), data_type: DataType::FLOAT }.as_json()).to(be_ok().value(json!(100.0)));
    expect!(DataValue { generated: "100".into(), data_type: DataType::DECIMAL }.as_json()).to(be_ok().value(json!(100.0)));
    expect!(DataValue { generated: "100".into(), data_type: DataType::BOOLEAN }.as_json()).to(be_err());

    expect!(DataValue { generated: "100.5".into(), data_type: DataType::RAW }.as_json()).to(be_ok().value(json!("100.5")));
    expect!(DataValue { generated: "100.5".into(), data_type: DataType::STRING }.as_json()).to(be_ok().value(json!("100.5")));
    expect!(DataValue { generated: "100.5".into(), data_type: DataType::INTEGER }.as_json()).to(be_err());
    expect!(DataValue { generated: "100.5".into(), data_type: DataType::FLOAT }.as_json()).to(be_ok().value(json!(100.5)));
    expect!(DataValue { generated: "100.5".into(), data_type: DataType::DECIMAL }.as_json()).to(be_ok().value(json!(100.5)));
    expect!(DataValue { generated: "100.5".into(), data_type: DataType::BOOLEAN }.as_json()).to(be_err());

    expect!(DataValue { generated: "true".into(), data_type: DataType::RAW }.as_json()).to(be_ok().value(json!("true")));
    expect!(DataValue { generated: "true".into(), data_type: DataType::STRING }.as_json()).to(be_ok().value(json!("true")));
    expect!(DataValue { generated: "true".into(), data_type: DataType::INTEGER }.as_json()).to(be_err());
    expect!(DataValue { generated: "true".into(), data_type: DataType::FLOAT }.as_json()).to(be_err());
    expect!(DataValue { generated: "true".into(), data_type: DataType::DECIMAL }.as_json()).to(be_err());
    expect!(DataValue { generated: "true".into(), data_type: DataType::BOOLEAN }.as_json()).to(be_ok().value(json!(true)));
  }
}
