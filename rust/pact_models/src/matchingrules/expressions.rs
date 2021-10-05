//! Matching Rule definition expressions
//!
//! Parser for parsing matching rule definitions into a value, matching rules and generator tuple.
//!
//! The following are examples of matching rule definitions:
//! * `matching(type,'Name')` - type matcher
//! * `matching(number,100)` - number matcher
//! * `matching(datetime, 'yyyy-MM-dd','2000-01-01')` - datetime matcher with format string
//!

use anyhow::anyhow;
use itertools::Either;
use log::warn;
use logos::{Lexer, Logos};

use crate::generators::Generator;
use crate::matchingrules::MatchingRule;
use crate::matchingrules::MatchingRule::NotEmpty;

/// Type to associate with an expression element
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ValueType {
  Unknown,
  String,
  Number,
  Integer,
  Decimal,
  Boolean
}

impl ValueType {
  /// Merge this value type with the other one
  pub fn merge(self, other: ValueType) -> ValueType {
    match (self, other) {
      (ValueType::String, ValueType::String) => ValueType::String,
      (ValueType::Number, ValueType::Number) => ValueType::Number,
      (ValueType::Number, ValueType::Boolean) => ValueType::Number,
      (ValueType::Number, ValueType::Unknown) => ValueType::Number,
      (ValueType::Number, ValueType::Integer) => ValueType::Integer,
      (ValueType::Number, ValueType::Decimal) => ValueType::Decimal,
      (ValueType::Number, ValueType::String) => ValueType::String,
      (ValueType::Integer, ValueType::Number) => ValueType::Integer,
      (ValueType::Integer, ValueType::Boolean) => ValueType::Integer,
      (ValueType::Integer, ValueType::Unknown) => ValueType::Integer,
      (ValueType::Integer, ValueType::Integer) => ValueType::Integer,
      (ValueType::Integer, ValueType::Decimal) => ValueType::Decimal,
      (ValueType::Integer, ValueType::String) => ValueType::String,
      (ValueType::Decimal, ValueType::Number) => ValueType::Decimal,
      (ValueType::Decimal, ValueType::Boolean) => ValueType::Decimal,
      (ValueType::Decimal, ValueType::Unknown) => ValueType::Decimal,
      (ValueType::Decimal, ValueType::Integer) => ValueType::Decimal,
      (ValueType::Decimal, ValueType::Decimal) => ValueType::Decimal,
      (ValueType::Decimal, ValueType::String) => ValueType::String,
      (ValueType::Boolean, ValueType::Number) => ValueType::Number,
      (ValueType::Boolean, ValueType::Integer) => ValueType::Integer,
      (ValueType::Boolean, ValueType::Decimal) => ValueType::Decimal,
      (ValueType::Boolean, ValueType::Unknown) => ValueType::Boolean,
      (ValueType::Boolean, ValueType::String) => ValueType::String,
      (ValueType::Boolean, ValueType::Boolean) => ValueType::Boolean,
      (ValueType::String, _) => ValueType::String,
      (_, _) => other
    }
  }
}

/// Reference to another attribute that defines the structure of the matching rule
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchingReference {
  /// Name of the attribute that the reference is to
  pub name: String
}

/// Matching rule definition constructed from parsing a matching rule definition expression
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchingRuleDefinition {
  pub value: String,
  pub value_type: ValueType,
  pub rules: Vec<Either<MatchingRule, MatchingReference>>,
  pub generator: Option<Generator>
}

impl MatchingRuleDefinition {
  /// Construct a new MatchingRuleDefinition
  pub fn new(
    value: String,
    value_type: ValueType,
    matching_rule: MatchingRule,
    generator: Option<Generator>
  ) -> Self {
    MatchingRuleDefinition {
      value,
      value_type,
      rules: vec![ Either::Left(matching_rule) ],
      generator
    }
  }

  /// Merges two matching rules definitions. This is used when multiple matching rules are
  /// provided for a single element.
  pub fn merge(&self, other: &MatchingRuleDefinition) -> MatchingRuleDefinition {
    if !self.value.is_empty() && !other.value.is_empty() {
      warn!("There are multiple matching rules with values for the same value. There is no \
        reliable way to combine them, so the later value ('{}') will be ignored.", other.value)
    }

    if self.generator.is_some() && other.generator.is_some() {
      warn!("There are multiple generators for the same value. There is no reliable way to combine \
       them, so the later generator ({:?}) will be ignored.", other.generator)
    }

    MatchingRuleDefinition {
      value: if self.value.is_empty() { other.value.clone() } else { self.value.clone() },
      value_type: self.value_type.merge(other.value_type),
      rules: [self.rules.clone(), other.rules.clone()].concat(),
      generator: self.generator.as_ref().or(other.generator.as_ref()).cloned()
    }
  }
}

#[derive(Logos, Debug, PartialEq)]
enum MatcherDefinitionToken {
  #[token("matching")]
  Matching,

  #[token("notEmpty")]
  NotEmpty,

  #[token("(")]
  LeftBracket,

  #[token(")")]
  RightBracket,

  #[token(",")]
  Comma,

  #[regex("'[^']*'")]
  String,

  #[regex("[a-zA-Z]+")]
  Id,

  #[regex("-?[0-9]+", |lex| lex.slice().parse())]
  Int(i64),

  #[regex(r"-?[0-9]\.[0-9]+")]
  Decimal,

  #[regex(r"true|false")]
  Boolean,

  #[regex(r"null")]
  Null,

  #[error]
  #[regex(r"[ \t\n\f]+", logos::skip)]
  Error
}

/// Parse a matcher definition into a MatchingRuleDefinition containing the example value, matching rules and any
/// generator.
/// The following are examples of matching rule definitions:
/// * `matching(type,'Name')` - type matcher
/// * `matching(number,100)` - number matcher
/// * `matching(datetime, 'yyyy-MM-dd','2000-01-01')` - datetime matcher with format string
pub fn parse_matcher_def(v: &str) -> anyhow::Result<MatchingRuleDefinition> {
  let mut lex = MatcherDefinitionToken::lexer(v);
  matching_definition(&mut lex).map_err(|err| {
    anyhow!("'{}' is not a valid value definition: {}", v, err)
  })
}

// matchingDefinition returns [ MatchingRuleDefinition value ] :
//     matchingDefinitionExp { $value = $matchingDefinitionExp.value; } ( COMMA e=matchingDefinitionExp {  if ($value != null) { $value = $value.merge($e.value); } } )* EOF
//     ;
fn matching_definition(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<MatchingRuleDefinition> {
  let value = matching_definition_exp(lex)?;
  while let Some(next) = lex.next() {
    if next == MatcherDefinitionToken::Comma {
      value.merge(&matching_definition_exp(lex)?);
    } else {
      return Err(anyhow!("expected comma, got '{}'", lex.slice()));
    }
  }

  let remainder = lex.remainder();
  if !remainder.is_empty() {
    Err(anyhow!("expected not more tokens, got '{}' with '{}' remaining", lex.slice(), remainder))
  } else {
    Ok(value)
  }
}

// matchingDefinitionExp returns [ MatchingRuleDefinition value ] :
//     (
//       'matching' LEFT_BRACKET matchingRule RIGHT_BRACKET {
//         if ($matchingRule.reference != null) {
//           $value = new MatchingRuleDefinition($matchingRule.value, $matchingRule.reference, $matchingRule.generator);
//         } else {
//           $value = new MatchingRuleDefinition($matchingRule.value, $matchingRule.rule, $matchingRule.generator);
//         }
//       }
//       | 'notEmpty' LEFT_BRACKET string RIGHT_BRACKET { $value = new MatchingRuleDefinition($string.contents, NotEmptyMatcher.INSTANCE, null); }
//       | 'eachKey' LEFT_BRACKET e=matchingDefinitionExp RIGHT_BRACKET { if ($e.value != null) { $value = new MatchingRuleDefinition(null, new EachKeyMatcher($e.value), null); } }
//       | 'eachValue' LEFT_BRACKET e=matchingDefinitionExp RIGHT_BRACKET {
//         if ($e.value != null) {
//           $value = new MatchingRuleDefinition(null, ValueType.Unknown, List.of((Either<MatchingRule, MatchingReference>) new Either.A(new EachValueMatcher($e.value))), null);
//         }
//       }
//     )
//     ;
fn matching_definition_exp(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<MatchingRuleDefinition> {
  let next = lex.next();
  if let Some(token) = next {
    if token == MatcherDefinitionToken::Matching {
      let (value, value_type, matching_rule, generator, reference) = parse_matching(lex)?;
      if let Some(reference) = reference {
        Ok(MatchingRuleDefinition {
          value: value.clone(),
          value_type: ValueType::Unknown,
          rules: vec![ Either::Right(reference) ],
          generator: generator.clone()
        })
      } else {
        Ok(MatchingRuleDefinition {
          value: value.clone(),
          value_type,
          rules: vec![ Either::Left(matching_rule.unwrap()) ],
          generator: generator.clone()
        })
      }
    } else if token == MatcherDefinitionToken::NotEmpty {
      let value = parse_not_empty(lex)?;
      Ok(MatchingRuleDefinition {
        value: value.clone(),
        value_type: ValueType::Unknown,
        rules: vec![ Either::Left(NotEmpty) ],
        generator: None
      })
    } else {
      Err(anyhow!("expected type of matching rule (matching, notEmpty, eachKey, eachValue) but got '{}'", lex.slice()))
    }
  } else {
    Err(anyhow!("expected type of matching rule (matching, notEmpty, eachKey, eachValue) but got EOS"))
  }
}

// LEFT_BRACKET string RIGHT_BRACKET
fn parse_not_empty(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<String> {
  let next = lex.next().ok_or(anyhow!("expected '('"))?;
  if next == MatcherDefinitionToken::LeftBracket {
    let value = parse_string(lex)?;
    let next = lex.next().ok_or(anyhow!("expected ')'"))?;
    if next == MatcherDefinitionToken::RightBracket {
      Ok(value)
    } else {
      Err(anyhow!("expected closing bracket, got '{}'", lex.slice()))
    }
  } else {
    Err(anyhow!("expected '(', got '{}'", lex.remainder()))
  }
}

// LEFT_BRACKET matchingRule RIGHT_BRACKET
fn parse_matching(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  let next = lex.next().ok_or(anyhow!("expected '('"))?;
  if next == MatcherDefinitionToken::LeftBracket {
    let result = parse_matching_rule(lex)?;
    let next = lex.next().ok_or(anyhow!("expected ')'"))?;
    if next == MatcherDefinitionToken::RightBracket {
      Ok(result)
    } else {
      Err(anyhow!("expected closing bracket, got '{}'", lex.slice()))
    }
  } else {
    Err(anyhow!("expected '(', got '{}'", lex.remainder()))
  }
}

// matchingRule returns [ String value, ValueType type, MatchingRule rule, Generator generator, MatchingReference reference ] :
//   (
//     ( 'equalTo' { $rule = EqualsMatcher.INSTANCE; }
//     | 'type'  { $rule = TypeMatcher.INSTANCE; } )
//     COMMA v=primitiveValue { $value = $v.value; $type = $v.type; } )
//   | 'number' { $rule = new NumberTypeMatcher(NumberTypeMatcher.NumberType.NUMBER); } COMMA val=( DECIMAL_LITERAL | INTEGER_LITERAL ) { $value = $val.getText(); $type = ValueType.Number; }
//   | 'integer' { $rule = new NumberTypeMatcher(NumberTypeMatcher.NumberType.INTEGER); } COMMA val=INTEGER_LITERAL { $value = $val.getText(); $type = ValueType.Integer; }
//   | 'decimal' { $rule = new NumberTypeMatcher(NumberTypeMatcher.NumberType.DECIMAL); } COMMA val=DECIMAL_LITERAL { $value = $val.getText(); $type = ValueType.Decimal; }
//   | matcherType=( 'datetime' | 'date' | 'time' ) COMMA format=string {
//     if ($matcherType.getText().equals("datetime")) { $rule = new TimestampMatcher($format.contents); }
//     if ($matcherType.getText().equals("date")) { $rule = new DateMatcher($format.contents); }
//     if ($matcherType.getText().equals("time")) { $rule = new TimeMatcher($format.contents); }
//     } COMMA s=string { $value = $s.contents; $type = ValueType.String; }
//   | 'regex' COMMA r=string COMMA s=string { $rule = new RegexMatcher($r.contents); $value = $s.contents; $type = ValueType.String; }
//   | 'include' COMMA s=string { $rule = new IncludeMatcher($s.contents); $value = $s.contents; $type = ValueType.String; }
//   | 'boolean' COMMA BOOLEAN_LITERAL { $rule = BooleanMatcher.INSTANCE; $value = $BOOLEAN_LITERAL.getText(); $type = ValueType.Boolean; }
//   | 'semver' COMMA s=string { $rule = SemverMatcher.INSTANCE; $value = $s.contents; $type = ValueType.String; }
//   | 'contentType' COMMA ct=string COMMA s=string { $rule = new ContentTypeMatcher($ct.contents); $value = $s.contents; $type = ValueType.Unknown; }
//   | DOLLAR ref=string { $reference = new MatchingReference($ref.contents); $type = ValueType.Unknown; }
//   ;
fn parse_matching_rule(lex: &mut logos::Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  let next = lex.next()
    .ok_or(anyhow!("expected a matcher type"))?;
  if next == MatcherDefinitionToken::Id {
    match lex.slice() {
      "equalTo" => parse_equality(lex),
      "regex" => parse_regex(lex),
      "type" => parse_type(lex),
      "datetime" => parse_datetime(lex),
      "date" => parse_date(lex),
      "time" => parse_time(lex),
      "include" => parse_include(lex),
      "number" => parse_number(lex),
      "integer" => parse_integer(lex),
      "decimal" => parse_decimal(lex),
      "boolean" => parse_boolean(lex),
      "contentType" => parse_content_type(lex),
      _ => Err(anyhow!("expected the type of matcher, got '{}'", lex.slice()))
    }
  } else {
    Err(anyhow!("expected the type of matcher, got '{}'", lex.slice()))
  }
}

//     COMMA v=primitiveValue { $value = $v.value; $type = $v.type; } )
fn parse_equality(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex)?;
  let (value, value_type) = parse_primitive_value(lex)?;
  Ok((value, value_type, Some(MatchingRule::Equality), None, None))
}

// COMMA r=string COMMA s=string { $rule = new RegexMatcher($r.contents); $value = $s.contents; $type = ValueType.String; }
fn parse_regex(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex)?;
  let regex = parse_string(lex)?;
  parse_comma(lex)?;
  let value = parse_string(lex)?;
  Ok((value, ValueType::String, Some(MatchingRule::Regex(regex)), None, None))
}

// COMMA v=primitiveValue { $value = $v.value; $type = $v.type; } )
fn parse_type(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex)?;
  let (value, value_type) = parse_primitive_value(lex)?;
  Ok((value, value_type, Some(MatchingRule::Type), None, None))
}

// COMMA format=string COMMA s=string { $value = $s.contents; $type = ValueType.String; }
fn parse_datetime(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex)?;
  let format = parse_string(lex)?;
  parse_comma(lex)?;
  let value = parse_string(lex)?;
  Ok((value, ValueType::String, Some(MatchingRule::Timestamp(format.clone())), Some(Generator::DateTime(Some(format.clone()))), None))
}

// COMMA format=string COMMA s=string { $value = $s.contents; $type = ValueType.String; }
fn parse_date(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex)?;
  let format = parse_string(lex)?;
  parse_comma(lex)?;
  let value = parse_string(lex)?;
  Ok((value, ValueType::String, Some(MatchingRule::Date(format.clone())), Some(Generator::Date(Some(format.clone()))), None))
}

// COMMA format=string COMMA s=string { $value = $s.contents; $type = ValueType.String; }
fn parse_time(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex)?;
  let format = parse_string(lex)?;
  parse_comma(lex)?;
  let value = parse_string(lex)?;
  Ok((value, ValueType::String, Some(MatchingRule::Time(format.clone())), Some(Generator::Time(Some(format.clone()))), None))
}

// COMMA s=string { $rule = new IncludeMatcher($s.contents); $value = $s.contents; $type = ValueType.String; }
fn parse_include(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex)?;
  let value = parse_string(lex)?;
  Ok((value.clone(), ValueType::String, Some(MatchingRule::Include(value.clone())), None, None))
}

// COMMA ct=string COMMA s=string { $rule = new ContentTypeMatcher($ct.contents); $value = $s.contents; $type = ValueType.Unknown; }
fn parse_content_type(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex)?;
  let ct = parse_string(lex)?;
  parse_comma(lex)?;
  let value = parse_string(lex)?;
  Ok((value.clone(), ValueType::Unknown, Some(MatchingRule::ContentType(ct.clone())), None, None))
}

// primitiveValue returns [ String value, ValueType type ] :
//   string { $value = $string.contents; $type = ValueType.String; }
//   | v=DECIMAL_LITERAL { $value = $v.getText(); $type = ValueType.Decimal; }
//   | v=INTEGER_LITERAL { $value = $v.getText(); $type = ValueType.Integer; }
//   | v=BOOLEAN_LITERAL { $value = $v.getText(); $type = ValueType.Boolean; }
//   ;
// string returns [ String contents ] :
//   STRING_LITERAL {
//     String contents = $STRING_LITERAL.getText();
//     $contents = contents.substring(1, contents.length() - 1);
//   }
//   | 'null'
//   ;
fn parse_primitive_value(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, ValueType)> {
  let next = lex.next().ok_or(anyhow!("expected a primitive value"))?;
  match next {
    MatcherDefinitionToken::String => Ok((lex.slice().trim_matches('\'').to_string(), ValueType::String)),
    MatcherDefinitionToken::Null => Ok((String::new(), ValueType::String)),
    MatcherDefinitionToken::Int(_) => Ok((lex.slice().to_string(), ValueType::Integer)),
    MatcherDefinitionToken::Decimal => Ok((lex.slice().to_string(), ValueType::Decimal)),
    MatcherDefinitionToken::Boolean => Ok((lex.slice().to_string(), ValueType::Boolean)),
    _ => Err(anyhow!("expected a primitive value, got '{}'", lex.slice()))
  }
}

// COMMA val=( DECIMAL_LITERAL | INTEGER_LITERAL ) { $value = $val.getText(); $type = ValueType.Number; }
fn parse_number(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex)?;
  let next = lex.next().ok_or(anyhow!("expected a number"))?;
  if let MatcherDefinitionToken::Int(_) = next {
    Ok((lex.slice().to_string(), ValueType::Number,  Some(MatchingRule::Number), None, None))
  } else if MatcherDefinitionToken::Decimal == next {
    Ok((lex.slice().to_string(), ValueType::Number,  Some(MatchingRule::Number), None, None))
  } else {
    Err(anyhow!("expected a number, got '{}'", lex.slice()))
  }
}

// COMMA val=INTEGER_LITERAL { $value = $val.getText(); $type = ValueType.Integer; }
fn parse_integer(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex)?;
  let next = lex.next().ok_or(anyhow!("expected an integer"))?;
  if let MatcherDefinitionToken::Int(_) = next {
    Ok((lex.slice().to_string(), ValueType::Integer, Some(MatchingRule::Integer), None, None))
  } else {
    Err(anyhow!("expected an integer, got '{}'", lex.slice()))
  }
}

// COMMA val=DECIMAL_LITERAL { $value = $val.getText(); $type = ValueType.Decimal; }
fn parse_decimal(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex)?;
  let next = lex.next().ok_or(anyhow!("expected a decimal number"))?;
  if let MatcherDefinitionToken::Int(_) = next {
    Ok((lex.slice().to_string(), ValueType::Decimal, Some(MatchingRule::Decimal), None, None))
  } else if MatcherDefinitionToken::Decimal == next {
    Ok((lex.slice().to_string(), ValueType::Decimal, Some(MatchingRule::Decimal), None, None))
  } else {
    Err(anyhow!("expected a decimal number, got '{}'", lex.slice()))
  }
}

// COMMA BOOLEAN_LITERAL { $rule = BooleanMatcher.INSTANCE; $value = $BOOLEAN_LITERAL.getText(); $type = ValueType.Boolean; }
fn parse_boolean(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, ValueType, Option<MatchingRule>, Option<Generator>, Option<MatchingReference>)> {
  parse_comma(lex)?;
  let next = lex.next().ok_or(anyhow!("expected a boolean"))?;
  if MatcherDefinitionToken::Boolean == next {
    Ok((lex.slice().to_string(), ValueType::Boolean, Some(MatchingRule::Boolean), None, None))
  } else {
    Err(anyhow!("expected a boolean, got '{}'", lex.slice()))
  }
}

fn parse_string(lex: &mut logos::Lexer<MatcherDefinitionToken>) -> anyhow::Result<String> {
  let next = lex.next().ok_or(anyhow!("expected a starting quote"))?;
  if next == MatcherDefinitionToken::String {
    Ok(lex.slice().trim_matches('\'').to_string())
  } else {
    Err(anyhow!("expected a starting quote, got '{}'", lex.slice()))
  }
}

fn parse_comma(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<()> {
  let next = lex.next().ok_or(anyhow!("expected a ','"))?;
  if next == MatcherDefinitionToken::Comma {
    Ok(())
  } else {
    Err(anyhow!("expected a comma, got '{}'", lex.slice()))
  }
}

#[cfg(test)]
mod test {
  use expectest::prelude::*;

  use crate::generators::Generator::{Date, DateTime, Time};
  use crate::matchingrules::MatchingRule;

  use super::*;

  #[test]
  fn does_not_start_with_matching() {
    expect!(super::parse_matcher_def("")).to(be_err());
    expect!(super::parse_matcher_def("a, b, c")).to(be_err());
    expect!(super::parse_matcher_def("matching some other text")).to(be_err());
  }

  #[test]
  fn parse_type_matcher() {
    expect!(super::parse_matcher_def("matching(type,'Name')").unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("Name".to_string(), ValueType::String, MatchingRule::Type, None)));
    expect!(super::parse_matcher_def("matching( type, 'Name' )").unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("Name".to_string(), ValueType::String, MatchingRule::Type, None)));
  }

  #[test]
  fn parse_number_matcher() {
    expect!(super::parse_matcher_def("matching(number,100)").unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("100".to_string(), ValueType::Number, MatchingRule::Number, None)));
    expect!(super::parse_matcher_def("matching(integer,100)").unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("100".to_string(), ValueType::Integer, MatchingRule::Integer, None)));
    expect!(super::parse_matcher_def("matching(decimal,100)").unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("100".to_string(), ValueType::Decimal, MatchingRule::Decimal, None)));
  }

  #[test]
  fn parse_datetime_matcher() {
    expect!(super::parse_matcher_def("matching(datetime, 'yyyy-MM-dd','2000-01-01')").unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("2000-01-01".to_string(),
                   ValueType::String,
                   MatchingRule::Timestamp("yyyy-MM-dd".to_string()),
                   Some(DateTime(Some("yyyy-MM-dd".to_string()))))));
    expect!(super::parse_matcher_def("matching(date, 'yyyy-MM-dd','2000-01-01')").unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("2000-01-01".to_string(),
                   ValueType::String,
                   MatchingRule::Date("yyyy-MM-dd".to_string()),
                   Some(Date(Some("yyyy-MM-dd".to_string()))))));
    expect!(super::parse_matcher_def("matching(time, 'HH:mm:ss','12:00:00')").unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("12:00:00".to_string(),
                   ValueType::String,
                   MatchingRule::Time("HH:mm:ss".to_string()),
                   Some(Time(Some("HH:mm:ss".to_string()))))));
  }

  #[test]
  fn parse_regex_matcher() {
    expect!(super::parse_matcher_def("matching(regex,'\\w+', 'Fred')").unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("Fred".to_string(),
                                              ValueType::String,
                                              MatchingRule::Regex("\\w+".to_string()),
                                              None)));
  }

  #[test]
  fn parse_boolean_matcher() {
    expect!(super::parse_matcher_def("matching(boolean,true)").unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("true".to_string(),
                                              ValueType::Boolean,
                                              MatchingRule::Boolean,
                                              None)));
  }

  #[test]
  fn parse_include_matcher() {
    expect!(super::parse_matcher_def("matching(include,'Name')").unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("Name".to_string(),
                                              ValueType::String,
                                              MatchingRule::Include("Name".to_string()),
                                              None)));
  }

  #[test]
  fn parse_equals_matcher() {
    expect!(super::parse_matcher_def("matching(equalTo,'Name')").unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("Name".to_string(),
                                              ValueType::String,
                                              MatchingRule::Equality,
                                              None)));
  }

  #[test]
  fn parse_content_type_matcher() {
    expect!(super::parse_matcher_def("matching(contentType,'Name', 'Value')").unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("Value".to_string(),
                                              ValueType::Unknown,
                                              MatchingRule::ContentType("Name".to_string()),
                                              None)));
  }

  #[test]
  fn parse_not_empty() {
    expect!(super::parse_matcher_def("notEmpty('Value')").unwrap()).to(
      be_equal_to(MatchingRuleDefinition::new("Value".to_string(),
                                              ValueType::Unknown,
                                              MatchingRule::NotEmpty,
                                              None)));
  }

  #[test]
  fn merging_types() {
    expect!(ValueType::String.merge(ValueType::Unknown)).to(be_equal_to(ValueType::String));
    expect!(ValueType::Unknown.merge(ValueType::String )).to(be_equal_to(ValueType::String));
    expect!(ValueType::Unknown.merge(ValueType::Number )).to(be_equal_to(ValueType::Number));
    expect!(ValueType::Number .merge(ValueType::Unknown)).to(be_equal_to(ValueType::Number));
    expect!(ValueType::Unknown.merge(ValueType::Integer)).to(be_equal_to(ValueType::Integer));
    expect!(ValueType::Integer.merge(ValueType::Unknown)).to(be_equal_to(ValueType::Integer));
    expect!(ValueType::Unknown.merge(ValueType::Decimal)).to(be_equal_to(ValueType::Decimal));
    expect!(ValueType::Decimal.merge(ValueType::Unknown)).to(be_equal_to(ValueType::Decimal));
    expect!(ValueType::Unknown.merge(ValueType::Boolean)).to(be_equal_to(ValueType::Boolean));
    expect!(ValueType::Boolean.merge(ValueType::Unknown)).to(be_equal_to(ValueType::Boolean));
    expect!(ValueType::Unknown.merge(ValueType::Unknown)).to(be_equal_to(ValueType::Unknown));
    expect!(ValueType::String .merge(ValueType::String )).to(be_equal_to(ValueType::String));
    expect!(ValueType::Number .merge(ValueType::Number )).to(be_equal_to(ValueType::Number));
    expect!(ValueType::Integer.merge(ValueType::Integer)).to(be_equal_to(ValueType::Integer));
    expect!(ValueType::Decimal.merge(ValueType::Decimal)).to(be_equal_to(ValueType::Decimal));
    expect!(ValueType::Boolean.merge(ValueType::Boolean)).to(be_equal_to(ValueType::Boolean));
    expect!(ValueType::Number .merge(ValueType::String )).to(be_equal_to(ValueType::String));
    expect!(ValueType::Integer.merge(ValueType::String )).to(be_equal_to(ValueType::String));
    expect!(ValueType::Decimal.merge(ValueType::String )).to(be_equal_to(ValueType::String));
    expect!(ValueType::Boolean.merge(ValueType::String )).to(be_equal_to(ValueType::String));
    expect!(ValueType::String .merge(ValueType::Number )).to(be_equal_to(ValueType::String));
    expect!(ValueType::String .merge(ValueType::Integer)).to(be_equal_to(ValueType::String));
    expect!(ValueType::String .merge(ValueType::Decimal)).to(be_equal_to(ValueType::String));
    expect!(ValueType::String .merge(ValueType::Boolean)).to(be_equal_to(ValueType::String));
    expect!(ValueType::Number .merge(ValueType::Integer)).to(be_equal_to(ValueType::Integer));
    expect!(ValueType::Number .merge(ValueType::Decimal)).to(be_equal_to(ValueType::Decimal));
    expect!(ValueType::Number .merge(ValueType::Boolean)).to(be_equal_to(ValueType::Number));
    expect!(ValueType::Integer.merge(ValueType::Number )).to(be_equal_to(ValueType::Integer));
    expect!(ValueType::Integer.merge(ValueType::Decimal)).to(be_equal_to(ValueType::Decimal));
    expect!(ValueType::Integer.merge(ValueType::Boolean)).to(be_equal_to(ValueType::Integer));
    expect!(ValueType::Decimal.merge(ValueType::Number )).to(be_equal_to(ValueType::Decimal));
    expect!(ValueType::Decimal.merge(ValueType::Integer)).to(be_equal_to(ValueType::Decimal));
    expect!(ValueType::Decimal.merge(ValueType::Boolean)).to(be_equal_to(ValueType::Decimal));
    expect!(ValueType::Boolean.merge(ValueType::Number )).to(be_equal_to(ValueType::Number));
    expect!(ValueType::Boolean.merge(ValueType::Integer)).to(be_equal_to(ValueType::Integer));
    expect!(ValueType::Boolean.merge(ValueType::Decimal)).to(be_equal_to(ValueType::Decimal));
  }
}
