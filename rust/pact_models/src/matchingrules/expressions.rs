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
use log::debug;
use logos::{Lexer, Logos};
use crate::generators::Generator;
use crate::matchingrules::MatchingRule;

#[derive(Logos, Debug, PartialEq)]
enum MatcherDefinitionToken {
  #[token("matching")]
  Matching,

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

  #[error]
  #[regex(r"[ \t\n\f]+", logos::skip)]
  Error
}

/// Parse a matcher definition into a tuple containing the example value, matching rule and any
/// generator.
/// The following are examples of matching rule definitions:
/// * `matching(type,'Name')` - type matcher
/// * `matching(number,100)` - number matcher
/// * `matching(datetime, 'yyyy-MM-dd','2000-01-01')` - datetime matcher with format string
pub fn parse_matcher_def(v: &str) -> anyhow::Result<(String, Option<MatchingRule>, Option<Generator>)> {
  let mut lex = MatcherDefinitionToken::lexer(v);
  let next = lex.next();
  debug!("First Token: {:?}", next);
  if let Some(token) = next {
    if token == MatcherDefinitionToken::Matching {
      let next = lex.next().ok_or(anyhow!("'{}' is not a valid value definition, expected '('", v))?;
      if next == MatcherDefinitionToken::LeftBracket {
        let result = parse_matching_def(&mut lex)?;
        let next = lex.next().ok_or(anyhow!("'{}' is not a valid value definition, expected ')'", v))?;
        if next == MatcherDefinitionToken::RightBracket {
          let next = lex.next();
          if next.is_none() {
            Ok(result)
          } else {
            Err(anyhow!("'{}' is not a valid value definition, got '{}' after the closing bracket", v, lex.remainder()))
          }
        } else {
          Err(anyhow!("'{}' is not a valid value definition, expected closing bracket, got '{}'", v, lex.slice()))
        }
      } else {
        Err(anyhow!("'{}' is not a valid value definition, expected '(', got '{}'", v, lex.remainder()))
      }
    } else {
      Ok((v.to_string(), None, None))
    }
  } else {
    Ok((v.to_string(), None, None))
  }
}

fn parse_matching_def(lex: &mut logos::Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, Option<MatchingRule>, Option<Generator>)> {
  let next = lex.next()
    .ok_or(anyhow!("Not a valid matcher definition, expected a matcher type"))?;
  if next == MatcherDefinitionToken::Id {
    match lex.slice() {
      "equality" => parse_equality(lex),
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
      _ => Err(anyhow!("Not a valid matcher definition, expected the type of matcher, got '{}'", lex.slice()))
    }
  } else {
    Err(anyhow!("Not a valid matcher definition, expected the type of matcher, got '{}'", lex.slice()))
  }
}

fn parse_equality(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, Option<MatchingRule>, Option<Generator>)> {
  parse_comma(lex)?;
  let value = parse_string(lex)?;
  Ok((value, Some(MatchingRule::Equality), None))
}

fn parse_regex(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, Option<MatchingRule>, Option<Generator>)> {
  parse_comma(lex)?;
  let regex = parse_string(lex)?;
  parse_comma(lex)?;
  let value = parse_string(lex)?;
  Ok((value, Some(MatchingRule::Regex(regex)), None))
}

fn parse_type(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, Option<MatchingRule>, Option<Generator>)> {
  parse_comma(lex)?;
  let value = parse_string(lex)?;
  Ok((value, Some(MatchingRule::Type), None))
}

fn parse_datetime(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, Option<MatchingRule>, Option<Generator>)> {
  parse_comma(lex)?;
  let format = parse_string(lex)?;
  parse_comma(lex)?;
  let value = parse_string(lex)?;
  Ok((value, Some(MatchingRule::Timestamp(format.clone())), Some(Generator::DateTime(Some(format.clone())))))
}

fn parse_date(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, Option<MatchingRule>, Option<Generator>)> {
  parse_comma(lex)?;
  let format = parse_string(lex)?;
  parse_comma(lex)?;
  let value = parse_string(lex)?;
  Ok((value, Some(MatchingRule::Date(format.clone())), Some(Generator::Date(Some(format.clone())))))
}

fn parse_time(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, Option<MatchingRule>, Option<Generator>)> {
  parse_comma(lex)?;
  let format = parse_string(lex)?;
  parse_comma(lex)?;
  let value = parse_string(lex)?;
  Ok((value, Some(MatchingRule::Time(format.clone())), Some(Generator::Time(Some(format.clone())))))
}

fn parse_include(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, Option<MatchingRule>, Option<Generator>)> {
  parse_comma(lex)?;
  let value = parse_string(lex)?;
  Ok((value.clone(), Some(MatchingRule::Include(value.clone())), None))
}

fn parse_number(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, Option<MatchingRule>, Option<Generator>)> {
  parse_comma(lex)?;
  let next = lex.next()
    .ok_or(anyhow!("Not a valid matcher definition, expected a number"))?;
  if let MatcherDefinitionToken::Int(_) = next {
    Ok((lex.slice().to_string(), Some(MatchingRule::Number), None))
  } else if MatcherDefinitionToken::Decimal == next {
    Ok((lex.slice().to_string(), Some(MatchingRule::Number), None))
  } else {
    Err(anyhow!("Not a valid matcher definition, expected a number, got '{}'", lex.slice()))
  }
}

fn parse_integer(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, Option<MatchingRule>, Option<Generator>)> {
  parse_comma(lex)?;
  let next = lex.next()
    .ok_or(anyhow!("Not a valid matcher definition, expected an integer"))?;
  if let MatcherDefinitionToken::Int(_) = next {
    Ok((lex.slice().to_string(), Some(MatchingRule::Integer), None))
  } else {
    Err(anyhow!("Not a valid matcher definition, expected an integer, got '{}'", lex.slice()))
  }
}

fn parse_decimal(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, Option<MatchingRule>, Option<Generator>)> {
  parse_comma(lex)?;
  let next = lex.next()
    .ok_or(anyhow!("Not a valid matcher definition, expected a decimal number"))?;
  if let MatcherDefinitionToken::Int(_) = next {
    Ok((lex.slice().to_string(), Some(MatchingRule::Decimal), None))
  } else if MatcherDefinitionToken::Decimal == next {
    Ok((lex.slice().to_string(), Some(MatchingRule::Decimal), None))
  } else {
    Err(anyhow!("Not a valid matcher definition, expected a decimal number, got '{}'", lex.slice()))
  }
}

fn parse_boolean(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<(String, Option<MatchingRule>, Option<Generator>)> {
  parse_comma(lex)?;
  let next = lex.next()
    .ok_or(anyhow!("Not a valid matcher definition, expected a boolean"))?;
  if MatcherDefinitionToken::Boolean == next {
    Ok((lex.slice().to_string(), Some(MatchingRule::Boolean), None))
  } else {
    Err(anyhow!("Not a valid matcher definition, expected a boolean, got '{}'", lex.slice()))
  }
}

fn parse_string(lex: &mut logos::Lexer<MatcherDefinitionToken>) -> anyhow::Result<String> {
  let next = lex.next()
    .ok_or(anyhow!("Not a valid matcher definition, expected a starting quote"))?;
  if next == MatcherDefinitionToken::String {
    Ok(lex.slice().trim_matches('\'').to_string())
  } else {
    Err(anyhow!("Not a valid matcher definition, expected a starting quote, got '{}'", lex.slice()))
  }
}

fn parse_comma(lex: &mut Lexer<MatcherDefinitionToken>) -> anyhow::Result<()> {
  let next = lex.next()
    .ok_or(anyhow!("Not a valid matcher definition, expected a ','"))?;
  if next == MatcherDefinitionToken::Comma {
    Ok(())
  } else {
    Err(anyhow!("Not a valid matcher definition, expected a comma, got '{}'", lex.slice()))
  }
}

#[cfg(test)]
mod test {
  use expectest::prelude::*;
  use crate::matchingrules::MatchingRule;
  use crate::generators::Generator::{DateTime, Date, Time};

  #[test]
  fn does_not_start_with_matching() {
    expect!(super::parse_matcher_def("").unwrap()).to(be_equal_to(("".to_string(), None, None)));
    expect!(super::parse_matcher_def("a, b, c").unwrap()).to(be_equal_to(("a, b, c".to_string(), None, None)));
    expect!(super::parse_matcher_def("matching some other text")).to(be_err());
  }

  #[test]
  fn parse_type_matcher() {
    expect!(super::parse_matcher_def("matching(type,'Name')").unwrap()).to(
      be_equal_to(("Name".to_string(), Some(MatchingRule::Type), None)));
    expect!(super::parse_matcher_def("matching( type, 'Name' )").unwrap()).to(
      be_equal_to(("Name".to_string(), Some(MatchingRule::Type), None)));
  }

  #[test]
  fn parse_number_matcher() {
    expect!(super::parse_matcher_def("matching(number,100)").unwrap()).to(
      be_equal_to(("100".to_string(), Some(MatchingRule::Number), None)));
    expect!(super::parse_matcher_def("matching(integer,100)").unwrap()).to(
      be_equal_to(("100".to_string(), Some(MatchingRule::Integer), None)));
    expect!(super::parse_matcher_def("matching(decimal,100)").unwrap()).to(
      be_equal_to(("100".to_string(), Some(MatchingRule::Decimal), None)));
  }

  #[test]
  fn parse_datetime_matcher() {
    expect!(super::parse_matcher_def("matching(datetime, 'yyyy-MM-dd','2000-01-01')").unwrap()).to(
      be_equal_to(("2000-01-01".to_string(),
                   Some(MatchingRule::Timestamp("yyyy-MM-dd".to_string())),
                   Some(DateTime(Some("yyyy-MM-dd".to_string()))))));
    expect!(super::parse_matcher_def("matching(date, 'yyyy-MM-dd','2000-01-01')").unwrap()).to(
      be_equal_to(("2000-01-01".to_string(),
                   Some(MatchingRule::Date("yyyy-MM-dd".to_string())),
                   Some(Date(Some("yyyy-MM-dd".to_string()))))));
    expect!(super::parse_matcher_def("matching(time, 'HH:mm:ss','12:00:00')").unwrap()).to(
      be_equal_to(("12:00:00".to_string(),
                   Some(MatchingRule::Time("HH:mm:ss".to_string())),
                   Some(Time(Some("HH:mm:ss".to_string()))))));
  }

  #[test]
  fn parse_regex_matcher() {
    expect!(super::parse_matcher_def("matching(regex,'\\w+', 'Fred')").unwrap()).to(
      be_equal_to(("Fred".to_string(), Some(MatchingRule::Regex("\\w+".to_string())), None)));
  }

  #[test]
  fn parse_boolean_matcher() {
    expect!(super::parse_matcher_def("matching(boolean,true)").unwrap()).to(
      be_equal_to(("true".to_string(), Some(MatchingRule::Boolean), None)));
  }

  #[test]
  fn parse_include_matcher() {
    expect!(super::parse_matcher_def("matching(include,'Name')").unwrap()).to(
      be_equal_to(("Name".to_string(), Some(MatchingRule::Include("Name".to_string())), None)));
  }
}
