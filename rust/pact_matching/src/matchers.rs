use models::matchingrules::*;
use itertools::Itertools;
use onig::Regex;
use time_utils::validate_datetime;

pub trait Matches<A> {
    fn matches(&self, actual: &A, matcher: &MatchingRule) -> Result<(), String>;
}

impl Matches<String> for String {
    fn matches(&self, actual: &String, matcher: &MatchingRule) -> Result<(), String> {
        debug!("String -> String: comparing '{}' to '{}' using {:?}", self, actual, matcher);
        match *matcher {
          MatchingRule::Regex(ref regex) => {
            match Regex::new(regex) {
              Ok(re) => {
                if re.is_match(actual) {
                  Ok(())
                } else {
                  Err(format!("Expected '{}' to match '{}'", actual, regex))
                }
              },
              Err(err) => Err(format!("'{}' is not a valid regular expression - {}", regex, err))
            }
          },
          MatchingRule::Equality => {
            if self == actual {
              Ok(())
            } else {
              Err(format!("Expected '{}' to be equal to '{}'", self, actual))
            }
          },
          MatchingRule::Type |
          MatchingRule::MinType(_) |
          MatchingRule::MaxType(_)|
          MatchingRule::MinMaxType(_, _) => Ok(()),
          MatchingRule::Include(ref substr) => {
            if actual.contains(substr) {
              Ok(())
            } else {
              Err(format!("Expected '{}' to include '{}'", actual, substr))
            }
          },
          MatchingRule::Number | MatchingRule::Decimal => {
            match actual.parse::<f64>() {
              Ok(_) => Ok(()),
              Err(_) => Err(format!("Expected '{}' to match a number", actual))
            }
          },
          MatchingRule::Integer => {
            match actual.parse::<u64>() {
              Ok(_) => Ok(()),
              Err(_) => Err(format!("Expected '{}' to match an integer number", actual))
            }
          },
          MatchingRule::Date(ref s) => {
            match validate_datetime(actual, s) {
              Ok(_) => Ok(()),
              Err(_) => Err(format!("Expected '{}' to match a date format of '{}'", actual, s))
            }
          },
          MatchingRule::Time(ref s) => {
            match validate_datetime(actual, s) {
              Ok(_) => Ok(()),
              Err(_) => Err(format!("Expected '{}' to match a time format of '{}'", actual, s))
            }
          },
          MatchingRule::Timestamp(ref s) => {
            match validate_datetime(actual, s) {
              Ok(_) => Ok(()),
              Err(_) => Err(format!("Expected '{}' to match a timestamp format of '{}'", actual, s))
            }
          },
          _ => Err(format!("Unable to match '{}' using {:?}", self, matcher))
       }
    }
}

impl Matches<u64> for String {
    fn matches(&self, actual: &u64, matcher: &MatchingRule) -> Result<(), String> {
        debug!("String -> u64: comparing '{}' to {} using {:?}", self, actual, matcher);
        match *matcher {
          MatchingRule::Regex(ref regex) => {
            match Regex::new(regex) {
              Ok(re) => {
                if re.is_match(&actual.to_string()) {
                  Ok(())
                } else {
                  Err(format!("Expected {} to match '{}'", actual, regex))
                }
              },
              Err(err) => Err(format!("'{}' is not a valid regular expression - {}", regex, err))
            }
           },
          MatchingRule::Type |
          MatchingRule::MinType(_) |
          MatchingRule::MaxType(_) |
          MatchingRule::MinMaxType(_, _) =>
            Err(format!("Expected '{}' (String) to be the same type as {} (Number)", self, actual)),
          MatchingRule::Equality => Err(format!("Expected '{}' (String) to be equal to {} (Number)", self, actual)),
          MatchingRule::Include(ref substr) => {
            if actual.to_string().contains(substr) {
              Ok(())
            } else {
              Err(format!("Expected {} to include '{}'", actual, substr))
            }
          },
          MatchingRule::Number | MatchingRule::Integer => Ok(()),
          MatchingRule::Decimal => Err(format!("Expected {} to match a decimal number", actual)),
          _ => Err(format!("String: Unable to match {} using {:?}", self, matcher))
       }
    }
}

impl Matches<u64> for u64 {
    fn matches(&self, actual: &u64, matcher: &MatchingRule) -> Result<(), String> {
        debug!("u64 -> u64: comparing {} to {} using {:?}", self, actual, matcher);
        match *matcher {
          MatchingRule::Regex(ref regex) => {
            match Regex::new(regex) {
              Ok(re) => {
                if re.is_match(&actual.to_string()) {
                  Ok(())
                } else {
                  Err(format!("Expected {} to match '{}'", actual, regex))
                }
              },
              Err(err) => Err(format!("'{}' is not a valid regular expression - {}", regex, err))
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
                 Err(format!("Expected {} to be equal to {}", self, actual))
             }
          },
          MatchingRule::Include(ref substr) => {
            if actual.to_string().contains(substr) {
              Ok(())
            } else {
              Err(format!("Expected {} to include '{}'", actual, substr))
            }
          },
          MatchingRule::Number | MatchingRule::Integer => Ok(()),
          MatchingRule::Decimal => Err(format!("Expected {} to match a decimal number", actual)),
          _ => Err(format!("Unable to match {} using {:?}", self, matcher))
       }
    }
}

impl Matches<f64> for u64 {
    fn matches(&self, actual: &f64, matcher: &MatchingRule) -> Result<(), String> {
        debug!("u64 -> f64: comparing {} to {} using {:?}", self, actual, matcher);
        match *matcher {
          MatchingRule::Regex(ref regex) => {
            match Regex::new(regex) {
              Ok(re) => {
                if re.is_match(&actual.to_string()) {
                  Ok(())
                } else {
                  Err(format!("Expected {} to match '{}'", actual, regex))
                }
              },
              Err(err) => Err(format!("'{}' is not a valid regular expression - {}", regex, err))
            }
          },
          MatchingRule::Type |
          MatchingRule::MinType(_) |
          MatchingRule::MaxType(_) |
          MatchingRule::MinMaxType(_, _) =>
            Err(format!("Expected {} (Integer) to be the same type as {} (Decimal)", self, actual)),
          MatchingRule::Equality => Err(format!("Expected {} (Integer) to be equal to {} (Decimal)", self, actual)),
          MatchingRule::Include(ref substr) => {
            if actual.to_string().contains(substr) {
              Ok(())
            } else {
              Err(format!("Expected {} to include '{}'", actual, substr))
            }
          },
          MatchingRule::Number | MatchingRule::Decimal => Ok(()),
          MatchingRule::Integer => Err(format!("Expected {} to match an integer number", actual)),
          _ => Err(format!("Unable to match {} using {:?}", self, matcher))
       }
    }
}

impl Matches<f64> for f64 {
    fn matches(&self, actual: &f64, matcher: &MatchingRule) -> Result<(), String> {
        debug!("f64 -> f64: comparing {} to {} using {:?}", self, actual, matcher);
        match *matcher {
          MatchingRule::Regex(ref regex) => {
            match Regex::new(regex) {
              Ok(re) => {
                if re.is_match(&actual.to_string()) {
                  Ok(())
                } else {
                  Err(format!("Expected {} to match '{}'", actual, regex))
                }
              },
              Err(err) => Err(format!("'{}' is not a valid regular expression - {}", regex, err))
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
                 Err(format!("Expected {} to be equal to {}", self, actual))
             }
          },
          MatchingRule::Include(ref substr) => {
            if actual.to_string().contains(substr) {
              Ok(())
            } else {
              Err(format!("Expected {} to include '{}'", actual, substr))
            }
          },
          MatchingRule::Number | MatchingRule::Decimal => Ok(()),
          MatchingRule::Integer => Err(format!("Expected {} to match an integer number", actual)),
          _ => Err(format!("Unable to match {} using {:?}", self, matcher))
       }
    }
}

impl Matches<u64> for f64 {
    fn matches(&self, actual: &u64, matcher: &MatchingRule) -> Result<(), String> {
        debug!("f64 -> u64: comparing {} to {} using {:?}", self, actual, matcher);
        match *matcher {
          MatchingRule::Regex(ref regex) => {
            match Regex::new(regex) {
              Ok(re) => {
                if re.is_match(&actual.to_string()) {
                  Ok(())
                } else {
                  Err(format!("Expected '{}' to match '{}'", actual, regex))
                }
              },
              Err(err) => Err(format!("'{}' is not a valid regular expression - {}", regex, err))
            }
          },
          MatchingRule::Type |
          MatchingRule::MinType(_) |
          MatchingRule::MaxType(_) |
          MatchingRule::MinMaxType(_, _) =>
            Err(format!("Expected {} (Decimal) to be the same type as {} (Integer)", self, actual)),
          MatchingRule::Equality => Err(format!("Expected {} (Decimal) to be equal to {} (Integer)", self, actual)),
          MatchingRule::Include(ref substr) => {
            if actual.to_string().contains(substr) {
              Ok(())
            } else {
              Err(format!("Expected {} to include '{}'", actual, substr))
            }
          },
          MatchingRule::Number | MatchingRule::Integer => Ok(()),
          MatchingRule::Decimal => Err(format!("Expected {} to match a decimal number", actual)),
          _ => Err(format!("Unable to match '{}' using {:?}", self, matcher))
       }
    }
}

impl Matches<Vec<u8>> for String {
  fn matches(&self, actual: &Vec<u8>, matcher: &MatchingRule) -> Result<(), String> {
    self.matches(&s!(std::str::from_utf8(actual).unwrap_or("")), matcher)
  }
}

impl Matches<Vec<u8>> for Vec<u8> {
  fn matches(&self, actual: &Vec<u8>, matcher: &MatchingRule) -> Result<(), String> {
    let self_str: String = s!(std::str::from_utf8(self).unwrap_or(""));
    self_str.matches(&s!(std::str::from_utf8(actual).unwrap_or("")), matcher)
  }
}

fn select_best_matcher(category: &str, path: &Vec<String>, matchers: &MatchingRules) -> Option<RuleList> {
  if category == "body" {
    matchers.resolve_body_matchers_by_path(path)
  } else {
    match matchers.resolve_matchers(category, path) {
      Some(category) => category.rules.values().next().cloned(),
      None => None
    }
  }
}

pub fn match_values<E, A>(category: &str, path: &Vec<String>, matchers: MatchingRules, expected: &E, actual: &A) -> Result<(), Vec<String>>
    where E: Matches<A> {
    let matching_rules = select_best_matcher(category, path, &matchers);
    match matching_rules {
        None => Err(vec![format!("No matcher found for category '{}' and path '{}'", category,
                            path.iter().join("."))]),
        Some(ref rulelist) => {
          let results = rulelist.rules.iter().map(|rule| expected.matches(actual, rule)).collect::<Vec<Result<(), String>>>();
          match rulelist.rule_logic {
            RuleLogic::And => {
              if results.iter().all(|result| result.is_ok()) {
                Ok(())
              } else {
                Err(results.iter().filter(|result| result.is_err()).map(|result| result.clone().unwrap_err()).collect())
              }
            },
            RuleLogic::Or => {
              if results.iter().any(|result| result.is_ok()) {
                Ok(())
              } else {
                Err(results.iter().filter(|result| result.is_err()).map(|result| result.clone().unwrap_err()).collect())
              }
            }
          }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::select_best_matcher;
    use expectest::prelude::*;

    #[test]
    fn select_best_matcher_selects_most_appropriate_by_weight() {
        let matchers = matchingrules!{
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

        expect!(select_best_matcher("body", &vec![s!("$")], &matchers)).to(
          be_some().value(RuleList::new(MatchingRule::Regex(s!("1")))));
        expect!(select_best_matcher("body", &vec![s!("$"), s!("a")], &matchers)).to(
          be_some().value(RuleList::new(MatchingRule::Regex(s!("1")))));

        expect!(select_best_matcher("body", &vec![s!("$"), s!("item1")], &matchers)).to(
          be_some().value(RuleList::new(MatchingRule::Regex(s!("3")))));
        expect!(select_best_matcher("body", &vec![s!("$"), s!("item2")], &matchers)).to(
          be_some().value(RuleList::new(MatchingRule::Regex(s!("4")))));
        expect!(select_best_matcher("body", &vec![s!("$"), s!("item3")], &matchers)).to(
          be_some().value(RuleList::new(MatchingRule::Regex(s!("1")))));

        expect!(select_best_matcher("header", &vec![s!("$"), s!("item1")], &matchers)).to(
          be_some().value(RuleList::new(MatchingRule::Regex(s!("5")))));

        expect!(select_best_matcher("body", &vec![s!("$"), s!("item1"), s!("level")], &matchers)).to(
          be_some().value(RuleList::new(MatchingRule::Regex(s!("6")))));
        expect!(select_best_matcher("body", &vec![s!("$"), s!("item1"), s!("level"), s!("1")], &matchers)).to(
          be_some().value(RuleList::new(MatchingRule::Regex(s!("7")))));
        expect!(select_best_matcher("body", &vec![s!("$"), s!("item1"), s!("level"), s!("2")], &matchers)).to(
          be_some().value(RuleList::new(MatchingRule::Regex(s!("10")))));
        expect!(select_best_matcher("body", &vec![s!("$"), s!("item1"), s!("level"), s!("1"), s!("id")], &matchers)).to(
          be_some().value(RuleList::new(MatchingRule::Regex(s!("8")))));
        expect!(select_best_matcher("body", &vec![s!("$"), s!("item1"), s!("level"), s!("1"), s!("name")], &matchers)).to(
          be_some().value(RuleList::new(MatchingRule::Regex(s!("9")))));
        expect!(select_best_matcher("body", &vec![s!("$"), s!("item1"), s!("level"), s!("1"), s!("other")], &matchers)).to(
          be_some().value(RuleList::new(MatchingRule::Regex(s!("7")))));
        expect!(select_best_matcher("body", &vec![s!("$"), s!("item1"), s!("level"), s!("2"), s!("id")], &matchers)).to(
          be_some().value(RuleList::new(MatchingRule::Regex(s!("11")))));
        expect!(select_best_matcher("body", &vec![s!("$"), s!("item1"), s!("level"), s!("3"), s!("id")], &matchers)).to(
          be_some().value(RuleList::new(MatchingRule::Regex(s!("12")))));
        expect!(select_best_matcher("body", &vec![s!("$"), s!("item2"), s!("level"), s!("1"), s!("id")], &matchers)).to(
          be_some().value(RuleList::new(MatchingRule::Regex(s!("13")))));
        expect!(select_best_matcher("body", &vec![s!("$"), s!("item2"), s!("level"), s!("3"), s!("id")], &matchers)).to(
          be_some().value(RuleList::new(MatchingRule::Regex(s!("13")))));
    }

    #[test]
    fn select_best_matcher_selects_handles_missing_type_attribute() {
        let matchers = matchingrules!{
            "body" => {
                "$.item1" => [ MatchingRule::Regex(s!("3")) ],
                "$.item2" => [ MatchingRule::MinType(4) ],
                "$.item3" => [ MatchingRule::MaxType(4) ],
                "$.item4" => [ ]
            }
        };

        expect!(select_best_matcher("body", &vec![s!("$"), s!("item1")], &matchers)).to(
          be_some().value(RuleList::new(MatchingRule::Regex(s!("3")))));
        expect!(select_best_matcher("body", &vec![s!("$"), s!("item2")], &matchers)).to(
          be_some().value(RuleList::new(MatchingRule::MinType(4))));
        expect!(select_best_matcher("body", &vec![s!("$"), s!("item3")], &matchers)).to(
          be_some().value(RuleList::new(MatchingRule::MaxType(4))));
        expect!(select_best_matcher("body", &vec![s!("$"), s!("item4")], &matchers)).to(be_none());
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

    let matcher = MatchingRule::Timestamp("yyyy-MM-dd HH:mm:ssX".into());
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
}
