use std::str::from_utf8;

use anyhow::anyhow;
use bytes::Bytes;
use itertools::Itertools;
use log::*;
use onig::Regex;

use pact_models::HttpStatus;

use crate::binary_utils::match_content_type;
use crate::MatchingContext;
use crate::models::matchingrules::*;
use crate::time_utils::validate_datetime;

pub trait Matches<A> {
  fn matches(&self, actual: &A, matcher: &MatchingRule) -> anyhow::Result<()>;
}

impl Matches<String> for String {
  fn matches(&self, actual: &String, matcher: &MatchingRule) -> anyhow::Result<()> {
    self.matches(&actual.as_str(), matcher)
  }
}

impl Matches<&str> for &str {
  fn matches(&self, actual: &&str, matcher: &MatchingRule) -> anyhow::Result<()> {
    self.to_string().matches(actual, matcher)
  }
}

impl Matches<&str> for String {
  fn matches(&self, actual: &&str, matcher: &MatchingRule) -> anyhow::Result<()> {
    debug!("String -> String: comparing '{}' to '{}' using {:?}", self, actual, matcher);
    match matcher {
      MatchingRule::Regex(regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            if re.is_match(actual) {
              Ok(())
            } else {
              Err(anyhow!("Expected '{}' to match '{}'", actual, regex))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Equality => {
        if self == actual {
          Ok(())
        } else {
          Err(anyhow!("Expected '{}' to be equal to '{}'", self, actual))
        }
      },
      MatchingRule::Type |
      MatchingRule::MinType(_) |
      MatchingRule::MaxType(_) |
      MatchingRule::MinMaxType(_, _) => Ok(()),
      MatchingRule::Include(substr) => {
        if actual.contains(substr) {
          Ok(())
        } else {
          Err(anyhow!("Expected '{}' to include '{}'", actual, substr))
        }
      },
      MatchingRule::Number | MatchingRule::Decimal => {
        match actual.parse::<f64>() {
          Ok(_) => Ok(()),
          Err(_) => Err(anyhow!("Expected '{}' to match a number", actual))
        }
      },
      MatchingRule::Integer => {
        match actual.parse::<u64>() {
          Ok(_) => Ok(()),
          Err(_) => Err(anyhow!("Expected '{}' to match an integer number", actual))
        }
      },
      MatchingRule::Date(s) => {
        match validate_datetime(&actual.to_string(), s) {
          Ok(_) => Ok(()),
          Err(_) => Err(anyhow!("Expected '{}' to match a date format of '{}'", actual, s))
        }
      },
      MatchingRule::Time(s) => {
        match validate_datetime(&actual.to_string(), s) {
          Ok(_) => Ok(()),
          Err(_) => Err(anyhow!("Expected '{}' to match a time format of '{}'", actual, s))
        }
      },
      MatchingRule::Timestamp(s) => {
        match validate_datetime(&actual.to_string(), s) {
          Ok(_) => Ok(()),
          Err(_) => Err(anyhow!("Expected '{}' to match a timestamp format of '{}'", actual, s))
        }
      },
      MatchingRule::Boolean => {
        if *actual == "true" || *actual == "false" {
          Ok(())
        } else {
          Err(anyhow!("Expected '{}' to match a boolean", actual))
        }
      }
      MatchingRule::StatusCode(status) => {
        match actual.parse::<u16>() {
          Ok(status_code) => match_status_code(status_code, status),
          Err(err) => Err(anyhow!("Unable to match '{}' using {:?} - {}", self, matcher, err))
        }
      }
      _ => Err(anyhow!("Unable to match '{}' using {:?}", self, matcher))
    }
  }
}

impl Matches<u64> for String {
  fn matches(&self, actual: &u64, matcher: &MatchingRule) -> anyhow::Result<()> {
    log::debug!("String -> u64: comparing '{}' to {} using {:?}", self, actual, matcher);
    match matcher {
      MatchingRule::Regex(regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            if re.is_match(&actual.to_string()) {
              Ok(())
            } else {
              Err(anyhow!("Expected {} to match '{}'", actual, regex))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Type |
      MatchingRule::MinType(_) |
      MatchingRule::MaxType(_) |
      MatchingRule::MinMaxType(_, _) =>
        Err(anyhow!("Expected '{}' (String) to be the same type as {} (Number)", self, actual)),
      MatchingRule::Equality => Err(anyhow!("Expected '{}' (String) to be equal to {} (Number)", self, actual)),
      MatchingRule::Include(substr) => {
        if actual.to_string().contains(substr) {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to include '{}'", actual, substr))
        }
      },
      MatchingRule::Number | MatchingRule::Integer => Ok(()),
      MatchingRule::Decimal => Err(anyhow!("Expected {} to match a decimal number", actual)),
      MatchingRule::StatusCode(status) => match_status_code(*actual as u16, status),
      _ => Err(anyhow!("String: Unable to match {} using {:?}", self, matcher))
    }
  }
}

impl Matches<u64> for u64 {
  fn matches(&self, actual: &u64, matcher: &MatchingRule) -> anyhow::Result<()> {
    debug!("u64 -> u64: comparing {} to {} using {:?}", self, actual, matcher);
    match matcher {
      MatchingRule::Regex(regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            if re.is_match(&actual.to_string()) {
              Ok(())
            } else {
              Err(anyhow!("Expected {} to match '{}'", actual, regex))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Type |
      MatchingRule::MinType(_) |
      MatchingRule::MaxType(_) |
      MatchingRule::MinMaxType(_, _) => Ok(()),
      MatchingRule::Equality => {
        if self == actual {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to be equal to {}", self, actual))
        }
      },
      MatchingRule::Include(substr) => {
        if actual.to_string().contains(substr) {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to include '{}'", actual, substr))
        }
      },
      MatchingRule::Number | MatchingRule::Integer => Ok(()),
      MatchingRule::Decimal => Err(anyhow!("Expected {} to match a decimal number", actual)),
      MatchingRule::StatusCode(status) => match_status_code(*actual as u16, status),
      _ => Err(anyhow!("Unable to match {} using {:?}", self, matcher))
    }
  }
}

impl Matches<f64> for u64 {
  fn matches(&self, actual: &f64, matcher: &MatchingRule) -> anyhow::Result<()> {
    debug!("u64 -> f64: comparing {} to {} using {:?}", self, actual, matcher);
    match matcher {
      MatchingRule::Regex(regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            if re.is_match(&actual.to_string()) {
              Ok(())
            } else {
              Err(anyhow!("Expected {} to match '{}'", actual, regex))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Type |
      MatchingRule::MinType(_) |
      MatchingRule::MaxType(_) |
      MatchingRule::MinMaxType(_, _) =>
        Err(anyhow!("Expected {} (Integer) to be the same type as {} (Decimal)", self, actual)),
      MatchingRule::Equality => Err(anyhow!("Expected {} (Integer) to be equal to {} (Decimal)", self, actual)),
      MatchingRule::Include(substr) => {
        if actual.to_string().contains(substr) {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to include '{}'", actual, substr))
        }
      },
      MatchingRule::Number | MatchingRule::Decimal => Ok(()),
      MatchingRule::Integer => Err(anyhow!("Expected {} to match an integer number", actual)),
      _ => Err(anyhow!("Unable to match {} using {:?}", self, matcher))
    }
  }
}

impl Matches<f64> for f64 {
  #[allow(clippy::float_cmp)]
  fn matches(&self, actual: &f64, matcher: &MatchingRule) -> anyhow::Result<()> {
    debug!("f64 -> f64: comparing {} to {} using {:?}", self, actual, matcher);
    match matcher {
      MatchingRule::Regex(regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            if re.is_match(&actual.to_string()) {
              Ok(())
            } else {
              Err(anyhow!("Expected {} to match '{}'", actual, regex))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Type |
      MatchingRule::MinType(_) |
      MatchingRule::MaxType(_) |
      MatchingRule::MinMaxType(_, _) => Ok(()),
      MatchingRule::Equality => {
        if self == actual {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to be equal to {}", self, actual))
        }
      },
      MatchingRule::Include(substr) => {
        if actual.to_string().contains(substr) {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to include '{}'", actual, substr))
        }
      },
      MatchingRule::Number | MatchingRule::Decimal => Ok(()),
      MatchingRule::Integer => Err(anyhow!("Expected {} to match an integer number", actual)),
      _ => Err(anyhow!("Unable to match {} using {:?}", self, matcher))
    }
  }
}

impl Matches<u64> for f64 {
  fn matches(&self, actual: &u64, matcher: &MatchingRule) -> anyhow::Result<()> {
    debug!("f64 -> u64: comparing {} to {} using {:?}", self, actual, matcher);
    match matcher {
      MatchingRule::Regex(ref regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            if re.is_match(&actual.to_string()) {
              Ok(())
            } else {
              Err(anyhow!("Expected '{}' to match '{}'", actual, regex))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Type |
      MatchingRule::MinType(_) |
      MatchingRule::MaxType(_) |
      MatchingRule::MinMaxType(_, _) =>
        Err(anyhow!("Expected {} (Decimal) to be the same type as {} (Integer)", self, actual)),
      MatchingRule::Equality => Err(anyhow!("Expected {} (Decimal) to be equal to {} (Integer)", self, actual)),
      MatchingRule::Include(substr) => {
        if actual.to_string().contains(substr) {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to include '{}'", actual, substr))
        }
      },
      MatchingRule::Number | MatchingRule::Integer => Ok(()),
      MatchingRule::Decimal => Err(anyhow!("Expected {} to match a decimal number", actual)),
      _ => Err(anyhow!("Unable to match '{}' using {:?}", self, matcher))
    }
  }
}

impl Matches<u16> for String {
  fn matches(&self, actual: &u16, matcher: &MatchingRule) -> anyhow::Result<()> {
    debug!("String -> u16: comparing '{}' to {} using {:?}", self, actual, matcher);
    self.matches(&(*actual as u64), matcher)
  }
}

impl Matches<u16> for u16 {
  fn matches(&self, actual: &u16, matcher: &MatchingRule) -> anyhow::Result<()> {
    debug!("u16 -> u16: comparing {} to {} using {:?}", self, actual, matcher);
    (*self as u64).matches(&(*actual as u64), matcher)
  }
}

impl Matches<i32> for String {
  fn matches(&self, actual: &i32, matcher: &MatchingRule) -> anyhow::Result<()> {
    debug!("String -> i32: comparing '{}' to {} using {:?}", self, actual, matcher);
    match matcher {
      MatchingRule::Regex(regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            if re.is_match(&actual.to_string()) {
              Ok(())
            } else {
              Err(anyhow!("Expected {} to match '{}'", actual, regex))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Type |
      MatchingRule::MinType(_) |
      MatchingRule::MaxType(_) |
      MatchingRule::MinMaxType(_, _) => Ok(()),
      MatchingRule::Equality => Err(anyhow!("Expected '{}' (String) to be equal to {} (Number)", self, actual)),
      MatchingRule::Include(substr) => {
        if actual.to_string().contains(substr) {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to include '{}'", actual, substr))
        }
      },
      MatchingRule::Number | MatchingRule::Integer => Ok(()),
      MatchingRule::Decimal => Err(anyhow!("Expected {} to match a decimal number", actual)),
      _ => Err(anyhow!("Unable to match {} using {:?}", self, matcher))
    }
  }
}

impl Matches<i32> for i32 {
  fn matches(&self, actual: &i32, matcher: &MatchingRule) -> anyhow::Result<()> {
    debug!("u16 -> u16: comparing {} to {} using {:?}", self, actual, matcher);
    match matcher {
      MatchingRule::Regex(regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            if re.is_match(&actual.to_string()) {
              Ok(())
            } else {
              Err(anyhow!("Expected {} to match '{}'", actual, regex))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Type |
      MatchingRule::MinType(_) |
      MatchingRule::MaxType(_) |
      MatchingRule::MinMaxType(_, _) => Ok(()),
      MatchingRule::Equality => {
        if self == actual {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to be equal to {}", self, actual))
        }
      },
      MatchingRule::Include(substr) => {
        if actual.to_string().contains(substr) {
          Ok(())
        } else {
          Err(anyhow!("Expected {} to include '{}'", actual, substr))
        }
      },
      MatchingRule::Number | MatchingRule::Integer => Ok(()),
      MatchingRule::Decimal => Err(anyhow!("Expected {} to match a decimal number", actual)),
      _ => Err(anyhow!("Unable to match {} using {:?}", self, matcher))
    }
  }
}

impl Matches<bool> for bool {
  fn matches(&self, actual: &bool, matcher: &MatchingRule) -> anyhow::Result<()> {
    debug!("bool -> bool: comparing '{}' to {} using {:?}", self, actual, matcher);
    match matcher {
      MatchingRule::Regex(regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            if re.is_match(&actual.to_string()) {
              Ok(())
            } else {
              Err(anyhow!("Expected {} to match '{}'", actual, regex))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Type |
      MatchingRule::MinType(_) |
      MatchingRule::MaxType(_) |
      MatchingRule::MinMaxType(_, _) => Ok(()),
      MatchingRule::Equality => if actual == self {
        Ok(())
      } else {
        Err(anyhow!("Expected {} (Boolean) to be equal to {} (Boolean)", self, actual))
      },
      MatchingRule::Boolean => Ok(()),
      _ => Err(anyhow!("Boolean: Unable to match {} using {:?}", self, matcher))
    }
  }
}

impl Matches<Bytes> for Bytes {
  fn matches(&self, actual: &Bytes, matcher: &MatchingRule) -> anyhow::Result<()> {
    debug!("Bytes -> Bytes: comparing {} bytes to {} bytes using {:?}", self.len(), actual.len(), matcher);
    match matcher {
      MatchingRule::Regex(regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            match from_utf8(actual) {
              Ok(s) => if re.is_match(s) {
                Ok(())
              } else {
                Err(anyhow!("Expected '{}' to match '{}'", s, regex))
              }
              Err(err) => Err(anyhow!("Could not convert actual bytes into a UTF-8 string - {}", err))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Equality => {
        if self == actual {
          Ok(())
        } else {
          Err(anyhow!("Expected '{:?}...' ({} bytes) to be equal to '{:?}...' ({} bytes)",
                      self.split_at(10).0, self.len(), actual.split_at(10).0, actual.len()))
        }
      },
      MatchingRule::Type |
      MatchingRule::MinType(_) |
      MatchingRule::MaxType(_) |
      MatchingRule::MinMaxType(_, _) => Ok(()),
      MatchingRule::Include(substr) => {
        match from_utf8(actual) {
          Ok(s) => if s.contains(substr) {
            Ok(())
          } else {
            Err(anyhow!("Expected '{}' to include '{}'", s, substr))
          }
          Err(err) => Err(anyhow!("Could not convert actual bytes into a UTF-8 string - {}", err))
        }
      },
      MatchingRule::ContentType(content_type) => match_content_type(&actual, content_type),
      _ => Err(anyhow!("Unable to match '{:?}...' ({} bytes) using {:?}",
                       actual.split_at(10).0, actual.len(), matcher))
    }
  }
}

pub fn match_values<E, A>(path: &[&str], context: &MatchingContext, expected: &E, actual: &A) -> Result<(), Vec<String>>
    where E: Matches<A> {
    let matching_rules = context.select_best_matcher(path);
    match matching_rules {
        None => Err(vec![format!("No matcher found for path '{}'", path.iter().join("."))]),
        Some(ref rulelist) => {
          let results = rulelist.rules.iter().map(|rule| {
            expected.matches(actual, rule)
          }).collect::<Vec<anyhow::Result<()>>>();
          match rulelist.rule_logic {
            RuleLogic::And => {
              if results.iter().all(|result| result.is_ok()) {
                Ok(())
              } else {
                Err(results.iter().filter(|result| result.is_err())
                  .map(|result| result.as_ref().unwrap_err().to_string()).collect())
              }
            },
            RuleLogic::Or => {
              if results.iter().any(|result| result.is_ok()) {
                Ok(())
              } else {
                Err(results.iter().filter(|result| result.is_err())
                  .map(|result| result.as_ref().unwrap_err().to_string()).collect())
              }
            }
          }
        }
    }
}

fn match_status_code(status_code: u16, status: &HttpStatus) -> anyhow::Result<()> {
  let matches = match status {
    HttpStatus::Information => (100..=199).contains(&status_code),
    HttpStatus::Success => (200..=299).contains(&status_code),
    HttpStatus::Redirect => (300..=399).contains(&status_code),
    HttpStatus::ClientError => (400..=499).contains(&status_code),
    HttpStatus::ServerError => (500..=599).contains(&status_code),
    HttpStatus::StatusCodes(status_codes) => status_codes.contains(&status_code),
    HttpStatus::NonError => status_code < 400,
    HttpStatus::Error => status_code >= 400
  };
  if matches {
    Ok(())
  } else {
    Err(anyhow!("Expected status code {} to be a {}", status_code, status))
  }
}

#[cfg(test)]
mod tests {
  use expectest::expect;
  use expectest::prelude::*;

  use super::*;

  #[test]
  fn select_best_matcher_selects_most_appropriate_by_weight() {
    let matchers = matchingrules! {
      "body" => {
        "$" => [ MatchingRule::Regex(s!("1")) ],
        "$.item1" => [ MatchingRule::Regex(s!("3")) ],
        "$.item2" => [ MatchingRule::Regex(s!("4")) ],
        "$.item1.level" => [ MatchingRule::Regex(s!("6")) ],
        "$.item1.level[1]" => [ MatchingRule::Regex(s!("7")) ],
        "$.item1.level[1].id" => [ MatchingRule::Regex(s!("8")) ],
        "$.item1.level[1].name" => [ MatchingRule::Regex(s!("9")) ],
        "$.item1.level[2]" => [ MatchingRule::Regex(s!("10")) ],
        "$.item1.level[2].id" => [ MatchingRule::Regex(s!("11")) ],
        "$.item1.level[*].id" => [ MatchingRule::Regex(s!("12")) ],
        "$.*.level[*].id" => [ MatchingRule::Regex(s!("13")) ]
      },
      "header" => {
        "item1" => [ MatchingRule::Regex(s!("5")) ]
      }
    };
    let body_matchers = matchers.rules_for_category("body").unwrap();
    let header_matchers = matchers.rules_for_category("header").unwrap();

    expect!(body_matchers.select_best_matcher(&vec!["$"])).to(
      be_some().value(RuleList::new(MatchingRule::Regex(s!("1")))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "a"])).to(
      be_some().value(RuleList::new(MatchingRule::Regex(s!("1")))));

    expect!(body_matchers.select_best_matcher(&vec!["$", "item1"])).to(
      be_some().value(RuleList::new(MatchingRule::Regex(s!("3")))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "item2"])).to(
      be_some().value(RuleList::new(MatchingRule::Regex(s!("4")))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "item3"])).to(
      be_some().value(RuleList::new(MatchingRule::Regex(s!("1")))));

    expect!(header_matchers.select_best_matcher(&vec!["$", "item1"])).to(
      be_some().value(RuleList::new(MatchingRule::Regex(s!("5")))));

    expect!(body_matchers.select_best_matcher(&vec!["$", "item1", "level"])).to(
      be_some().value(RuleList::new(MatchingRule::Regex(s!("6")))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "item1", "level", "1"])).to(
      be_some().value(RuleList::new(MatchingRule::Regex(s!("7")))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "item1", "level", "2"])).to(
      be_some().value(RuleList::new(MatchingRule::Regex(s!("10")))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "item1", "level", "1", "id"])).to(
      be_some().value(RuleList::new(MatchingRule::Regex(s!("8")))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "item1", "level", "1", "name"])).to(
      be_some().value(RuleList::new(MatchingRule::Regex(s!("9")))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "item1", "level", "1", "other"])).to(
      be_some().value(RuleList::new(MatchingRule::Regex(s!("7")))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "item1", "level", "2", "id"])).to(
      be_some().value(RuleList::new(MatchingRule::Regex(s!("11")))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "item1", "level", "3", "id"])).to(
      be_some().value(RuleList::new(MatchingRule::Regex(s!("12")))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "item2", "level", "1", "id"])).to(
      be_some().value(RuleList::new(MatchingRule::Regex(s!("13")))));
    expect!(body_matchers.select_best_matcher(&vec!["$", "item2", "level", "3", "id"])).to(
      be_some().value(RuleList::new(MatchingRule::Regex(s!("13")))));
  }

  #[test]
  fn select_best_matcher_selects_most_appropriate_when_weight_is_equal() {
    let matchers = matchingrules!{
      "body" => {
          "$.animals" => [ MatchingRule::Regex(s!("1")) ],
          "$.animals.*" => [ MatchingRule::Regex(s!("2")) ],
          "$.animals.*.alligator['@phoneNumber']" => [ MatchingRule::Regex(s!("3")) ]
      },
      "header" => {
          "item1" => [ MatchingRule::Regex(s!("5")) ]
      }
    };
    let body_matchers = matchers.rules_for_category("body").unwrap();

    expect!(body_matchers.select_best_matcher(&vec!["$", "animals", "0"])).to(
      be_some().value(RuleList::new(MatchingRule::Regex(s!("2")))));
  }

    #[test]
    fn select_best_matcher_selects_handles_missing_type_attribute() {
      let matchers = matchingrules_list! {
        "body";
        "$.item1" => [ MatchingRule::Regex(s!("3")) ],
        "$.item2" => [ MatchingRule::MinType(4) ],
        "$.item3" => [ MatchingRule::MaxType(4) ],
        "$.item4" => [ ]
      };

      expect!(matchers.select_best_matcher(&vec!["$", "item1"])).to(
        be_some().value(RuleList::new(MatchingRule::Regex(s!("3")))));
      expect!(matchers.select_best_matcher(&vec!["$", "item2"])).to(
        be_some().value(RuleList::new(MatchingRule::MinType(4))));
      expect!(matchers.select_best_matcher(&vec!["$", "item3"])).to(
        be_some().value(RuleList::new(MatchingRule::MaxType(4))));
      expect!(matchers.select_best_matcher(&vec!["$", "item4"])).to(be_none());
    }

    #[test]
    fn equality_matcher_test() {
        let matcher = MatchingRule::Equality;
        expect!(s!("100").matches(&s!("100"), &matcher)).to(be_ok());
        expect!(s!("100").matches(&s!("101"), &matcher)).to(be_err());
        expect!(s!("100").matches(&100, &matcher)).to(be_err());
        expect!(100.matches(&100, &matcher)).to(be_ok());
        expect!(100.matches(&100.0, &matcher)).to(be_err());
        expect!(100.1f64.matches(&100.0, &matcher)).to(be_err());
    }

    #[test]
    fn regex_matcher_test() {
        let matcher = MatchingRule::Regex(s!("^\\d+$"));
        expect!(s!("100").matches(&s!("100"), &matcher)).to(be_ok());
        expect!(s!("100").matches(&s!("10a"), &matcher)).to(be_err());
        expect!(s!("100").matches(&100, &matcher)).to(be_ok());
        expect!(100.matches(&100, &matcher)).to(be_ok());
        expect!(100.matches(&100.01f64, &matcher)).to(be_err());
        expect!(100.1f64.matches(&100.02f64, &matcher)).to(be_err());
    }

    #[test]
    fn type_matcher_test() {
        let matcher = MatchingRule::Type;
        expect!(s!("100").matches(&s!("100"), &matcher)).to(be_ok());
        expect!(s!("100").matches(&s!("10a"), &matcher)).to(be_ok());
        expect!(s!("100").matches(&100, &matcher)).to(be_err());
        expect!(100.matches(&200, &matcher)).to(be_ok());
        expect!(100.matches(&100.1, &matcher)).to(be_err());
        expect!(100.1f64.matches(&100.2, &matcher)).to(be_ok());
    }

    #[test]
    fn min_type_matcher_test() {
        let matcher = MatchingRule::MinType(3);
        expect!(s!("100").matches(&s!("100"), &matcher)).to(be_ok());
        expect!(s!("100").matches(&s!("10a"), &matcher)).to(be_ok());
        expect!(s!("100").matches(&s!("10"), &matcher)).to(be_ok());
        expect!(s!("100").matches(&100, &matcher)).to(be_err());
        expect!(100.matches(&200, &matcher)).to(be_ok());
        expect!(100.matches(&100.1, &matcher)).to(be_err());
        expect!(100.1f64.matches(&100.2, &matcher)).to(be_ok());
    }

    #[test]
    fn max_type_matcher_test() {
        let matcher = MatchingRule::MaxType(3);
        expect!(s!("100").matches(&s!("100"), &matcher)).to(be_ok());
        expect!(s!("100").matches(&s!("10a"), &matcher)).to(be_ok());
        expect!(s!("100").matches(&s!("1000"), &matcher)).to(be_ok());
        expect!(s!("100").matches(&100, &matcher)).to(be_err());
        expect!(100.matches(&200, &matcher)).to(be_ok());
        expect!(100.matches(&100.1, &matcher)).to(be_err());
        expect!(100.1f64.matches(&100.2, &matcher)).to(be_ok());
    }

  #[test]
  fn minmax_type_matcher_test() {
    let matcher = MatchingRule::MinMaxType(3, 6);
    expect!(s!("100").matches(&s!("100"), &matcher)).to(be_ok());
    expect!(s!("100").matches(&s!("10a"), &matcher)).to(be_ok());
    expect!(s!("100").matches(&s!("1000"), &matcher)).to(be_ok());
    expect!(s!("100").matches(&100, &matcher)).to(be_err());
    expect!(100.matches(&200, &matcher)).to(be_ok());
    expect!(100.matches(&100.1, &matcher)).to(be_err());
    expect!(100.1f64.matches(&100.2, &matcher)).to(be_ok());
  }

  #[test]
  fn timestamp_matcher_test() {
    let matcher = MatchingRule::Timestamp("yyyy-MM-dd HH:mm:ssZZZ".into());

    expect!(s!("100").matches(&s!("2013-12-01 14:00:00+10:00"), &matcher)).to(be_err());
    expect!(s!("100").matches(&s!("2013-12-01 14:00:00+1000"), &matcher)).to(be_ok());
    expect!(s!("100").matches(&s!("13-12-01 14:00:00+10:00"), &matcher)).to(be_err());
    expect!(s!("100").matches(&s!("I\'m a timestamp!"), &matcher)).to(be_err());
    expect!(s!("100").matches(&s!("100"), &matcher)).to(be_err());
    expect!(s!("100").matches(&s!("10a"), &matcher)).to(be_err());
    expect!(s!("100").matches(&s!("1000"), &matcher)).to(be_err());
    expect!(s!("100").matches(&100, &matcher)).to(be_err());
    expect!(100.matches(&200, &matcher)).to(be_err());
    expect!(100.matches(&100.1, &matcher)).to(be_err());
    expect!(100.1f64.matches(&100.2, &matcher)).to(be_err());

    let matcher = MatchingRule::Timestamp("yyyy-MM-dd HH:mm:ssXXX".into());
    expect!(s!("2014-01-01 14:00:00+10:00").matches(&s!("2013-12-01 14:00:00+10:00"), &matcher)).to(be_ok());

    let matcher = MatchingRule::Timestamp("yyyy#MM#dd#HH#mm#ss".into());
    expect!(s!("2014-01-01 14:00:00+10:00").matches(&s!("2013#12#01#14#00#00"), &matcher)).to(be_ok());
  }

  #[test]
  fn time_matcher_test() {
    let matcher = MatchingRule::Time("HH:mm:ss".into());

    expect!(s!("00:00:00").matches(&s!("14:00:00"), &matcher)).to(be_ok());
    expect!(s!("00:00:00").matches(&s!("33:00:00"), &matcher)).to(be_err());
    expect!(s!("00:00:00").matches(&s!("100"), &matcher)).to(be_err());
    expect!(s!("00:00:00").matches(&s!("10a"), &matcher)).to(be_err());
    expect!(s!("00:00:00").matches(&s!("1000"), &matcher)).to(be_err());
    expect!(s!("00:00:00").matches(&100, &matcher)).to(be_err());
    expect!(100.matches(&200, &matcher)).to(be_err());
    expect!(100.matches(&100.1, &matcher)).to(be_err());
    expect!(100.1f64.matches(&100.2, &matcher)).to(be_err());

    let matcher = MatchingRule::Time("mm:ss".into());
    expect!(s!("100").matches(&s!("14:01:01"), &matcher)).to(be_err());
    expect!(s!("100").matches(&s!("61:01"), &matcher)).to(be_err());

    let matcher = MatchingRule::Time("ss:mm:HH".into());
    expect!(s!("100").matches(&s!("05:10:14"), &matcher)).to(be_ok());

    let matcher = MatchingRule::Time("".into());
    expect!(s!("100").matches(&s!("14:00:00+10:00"), &matcher)).to(be_err());
  }

  #[test]
  fn date_matcher_test() {
    let matcher = MatchingRule::Date("yyyy-MM-dd".into());
    let matcher2 = MatchingRule::Date("MM/dd/yyyy".into());

    expect!(s!("100").matches(&s!("2001-10-01"), &matcher)).to(be_ok());
    expect!(s!("100").matches(&s!("01/14/2001"), &matcher2)).to(be_ok());
    expect!(s!("100").matches(&s!("01-13-01"), &matcher)).to(be_err());
    expect!(s!("100").matches(&s!("100"), &matcher)).to(be_err());
    expect!(s!("100").matches(&s!("10a"), &matcher)).to(be_err());
    expect!(s!("100").matches(&s!("1000"), &matcher)).to(be_err());
    expect!(s!("100").matches(&100, &matcher)).to(be_err());
    expect!(100.matches(&200, &matcher)).to(be_err());
    expect!(100.matches(&100.1, &matcher)).to(be_err());
    expect!(100.1f64.matches(&100.2, &matcher)).to(be_err());
  }

  #[test]
  fn include_matcher_test() {
    let matcher = MatchingRule::Include("10".into());
    expect!(s!("100").matches(&s!("100"), &matcher)).to(be_ok());
    expect!(s!("100").matches(&s!("10a"), &matcher)).to(be_ok());
    expect!(s!("100").matches(&s!("1000"), &matcher)).to(be_ok());
    expect!(s!("100").matches(&s!("200"), &matcher)).to(be_err());
    expect!(s!("100").matches(&100, &matcher)).to(be_ok());
    expect!(100.matches(&100, &matcher)).to(be_ok());
    expect!(100.matches(&100.1, &matcher)).to(be_ok());
    expect!(100.1f64.matches(&100.2, &matcher)).to(be_ok());
  }

  #[test]
  fn number_matcher_test() {
    let matcher = MatchingRule::Number;
    expect!(s!("100").matches(&s!("100"), &matcher)).to(be_ok());
    expect!(s!("100").matches(&s!("10a"), &matcher)).to(be_err());
    expect!(s!("100").matches(&s!("1000"), &matcher)).to(be_ok());
    expect!(s!("100").matches(&100, &matcher)).to(be_ok());
    expect!(100.matches(&200, &matcher)).to(be_ok());
    expect!(100.matches(&100.1, &matcher)).to(be_ok());
    expect!(100.1f64.matches(&100.2, &matcher)).to(be_ok());
  }

  #[test]
  fn integer_matcher_test() {
    let matcher = MatchingRule::Integer;
    expect!(s!("100").matches(&s!("100"), &matcher)).to(be_ok());
    expect!(s!("100").matches(&s!("10a"), &matcher)).to(be_err());
    expect!(s!("100").matches(&s!("1000"), &matcher)).to(be_ok());
    expect!(s!("100").matches(&100, &matcher)).to(be_ok());
    expect!(100.matches(&200, &matcher)).to(be_ok());
    expect!(100.matches(&100.1, &matcher)).to(be_err());
    expect!(100.1f64.matches(&100.2, &matcher)).to(be_err());
  }

  #[test]
  fn decimal_matcher_test() {
    let matcher = MatchingRule::Decimal;
    expect!(s!("100").matches(&s!("100"), &matcher)).to(be_ok());
    expect!(s!("100").matches(&s!("10a"), &matcher)).to(be_err());
    expect!(s!("100").matches(&s!("1000"), &matcher)).to(be_ok());
    expect!(s!("100").matches(&100, &matcher)).to(be_err());
    expect!(100.matches(&200, &matcher)).to(be_err());
    expect!(100.matches(&100.1, &matcher)).to(be_ok());
    expect!(100.1f64.matches(&100.2, &matcher)).to(be_ok());
  }

  #[test]
  fn null_matcher_test() {
    let matcher = MatchingRule::Null;
    expect!(s!("100").matches(&s!("100"), &matcher)).to(be_err());
    expect!(s!("100").matches(&s!("10a"), &matcher)).to(be_err());
    expect!(s!("100").matches(&s!("1000"), &matcher)).to(be_err());
    expect!(s!("100").matches(&100, &matcher)).to(be_err());
    expect!(100.matches(&200, &matcher)).to(be_err());
    expect!(100.matches(&100.1, &matcher)).to(be_err());
    expect!(100.1f64.matches(&100.2, &matcher)).to(be_err());
  }

  #[test]
  fn regex_matcher_supports_crazy_regexes() {
    let matcher = MatchingRule::Regex(
      r"^([\+-]?\d{4}(?!\d{2}\b))((-?)((0[1-9]|1[0-2])(\3([12]\d|0[1-9]|3[01]))?|W([0-4]\d|5[0-2])(-?[1-7])?|(00[1-9]|0[1-9]\d|[12]\d{2}|3([0-5]\d|6[1-6])))?)$"
        .into());
    expect!(s!("100").matches(&s!("2019-09-27"), &matcher)).to(be_ok());
  }

  #[test]
  fn boolean_matcher_test() {
    let matcher = MatchingRule::Boolean;
    expect!("100".to_string().matches(&"100", &matcher)).to(be_err());
    expect!("100".to_string().matches(&"10a", &matcher)).to(be_err());
    expect!("100".to_string().matches(&100, &matcher)).to(be_err());
    expect!(100.matches(&100.1, &matcher)).to(be_err());
    expect!("100".to_string().matches(&"true", &matcher)).to(be_ok());
    expect!("100".to_string().matches(&"false", &matcher)).to(be_ok());
    expect!(false.matches(&true, &matcher)).to(be_ok());
  }

  #[test]
  fn match_status_code_test() {
    expect!(match_status_code(100, &HttpStatus::Information)).to(be_ok());
    expect!(match_status_code(199, &HttpStatus::Information)).to(be_ok());
    expect!(match_status_code(500, &HttpStatus::Information)).to(be_err());
    expect!(match_status_code(200, &HttpStatus::Success)).to(be_ok());
    expect!(match_status_code(400, &HttpStatus::Success)).to(be_err());
    expect!(match_status_code(301, &HttpStatus::Redirect)).to(be_ok());
    expect!(match_status_code(500, &HttpStatus::Redirect)).to(be_err());
    expect!(match_status_code(404, &HttpStatus::ClientError)).to(be_ok());
    expect!(match_status_code(500, &HttpStatus::ClientError)).to(be_err());
    expect!(match_status_code(503, &HttpStatus::ServerError)).to(be_ok());
    expect!(match_status_code(499, &HttpStatus::ServerError)).to(be_err());
    expect!(match_status_code(200, &HttpStatus::StatusCodes(vec![200, 201, 204]))).to(be_ok());
    expect!(match_status_code(202, &HttpStatus::StatusCodes(vec![200, 201, 204]))).to(be_err());
    expect!(match_status_code(333, &HttpStatus::NonError)).to(be_ok());
    expect!(match_status_code(599, &HttpStatus::NonError)).to(be_err());
    expect!(match_status_code(555, &HttpStatus::Error)).to(be_ok());
    expect!(match_status_code(99, &HttpStatus::Error)).to(be_err());
  }
}
