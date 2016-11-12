use models::matchingrules::*;
use path_exp::*;
use itertools::Itertools;
use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub enum Matcher {
    EqualityMatcher,
    RegexMatcher(Regex),
    TypeMatcher,
    MinTypeMatcher(usize),
    MaxTypeMatcher(usize)
}

pub trait Matches<A> {
    fn matches(&self, actual: &A, matcher: &Matcher) -> Result<(), String>;
}

impl Matches<String> for String {
    fn matches(&self, actual: &String, matcher: &Matcher) -> Result<(), String> {
        debug!("comparing '{}' to '{}' using {:?}", self, actual, matcher);
        match *matcher {
           Matcher::RegexMatcher(ref regex) => {
               if regex.is_match(actual) {
                   Ok(())
               } else {
                   Err(format!("Expected '{}' to match '{}'", actual, regex))
               }
           },
           Matcher::TypeMatcher | Matcher::MinTypeMatcher(_) | Matcher::MaxTypeMatcher(_) => Ok(()),
           Matcher::EqualityMatcher => {
               if self == actual {
                   Ok(())
               } else {
                   Err(format!("Expected '{}' to be equal to '{}'", self, actual))
               }
           }
       }
    }
}

impl Matches<u64> for String {
    fn matches(&self, actual: &u64, matcher: &Matcher) -> Result<(), String> {
        debug!("comparing '{}' to {} using {:?}", self, actual, matcher);
        match *matcher {
           Matcher::RegexMatcher(ref regex) => {
               if regex.is_match(&actual.to_string()) {
                   Ok(())
               } else {
                   Err(format!("Expected '{}' to match '{}'", actual, regex))
               }
           },
           Matcher::TypeMatcher | Matcher::MinTypeMatcher(_) | Matcher::MaxTypeMatcher(_) => Err(
               format!("Expected '{}' (String) to be the same type as '{}' (Number)", self, actual)),
           Matcher::EqualityMatcher => Err(format!("Expected '{}' (String) to be equal to '{}' (Number)", self, actual))
       }
    }
}

impl Matches<u64> for u64 {
    fn matches(&self, actual: &u64, matcher: &Matcher) -> Result<(), String> {
        debug!("comparing '{}' to {} using {:?}", self, actual, matcher);
        match *matcher {
           Matcher::RegexMatcher(ref regex) => {
               if regex.is_match(&actual.to_string()) {
                   Ok(())
               } else {
                   Err(format!("Expected '{}' to match '{}'", actual, regex))
               }
           },
           Matcher::TypeMatcher | Matcher::MinTypeMatcher(_) | Matcher::MaxTypeMatcher(_) => Ok(()),
           Matcher::EqualityMatcher => {
               if self == actual {
                   Ok(())
               } else {
                   Err(format!("Expected '{}' to be equal to '{}'", self, actual))
               }
           }
       }
    }
}

impl Matches<f64> for u64 {
    fn matches(&self, actual: &f64, matcher: &Matcher) -> Result<(), String> {
        debug!("comparing '{}' to {} using {:?}", self, actual, matcher);
        match *matcher {
           Matcher::RegexMatcher(ref regex) => {
               if regex.is_match(&actual.to_string()) {
                   Ok(())
               } else {
                   Err(format!("Expected '{}' to match '{}'", actual, regex))
               }
           },
           Matcher::TypeMatcher | Matcher::MinTypeMatcher(_) | Matcher::MaxTypeMatcher(_) => Err(
               format!("Expected '{}' (Integer) to be the same type as '{}' (Decimal)", self, actual)),
           Matcher::EqualityMatcher => Err(format!("Expected '{}' (Integer) to be equal to '{}' (Decimal)", self, actual))
       }
    }
}

impl Matches<f64> for f64 {
    fn matches(&self, actual: &f64, matcher: &Matcher) -> Result<(), String> {
        debug!("comparing '{}' to {} using {:?}", self, actual, matcher);
        match *matcher {
           Matcher::RegexMatcher(ref regex) => {
               if regex.is_match(&actual.to_string()) {
                   Ok(())
               } else {
                   Err(format!("Expected '{}' to match '{}'", actual, regex))
               }
           },
           Matcher::TypeMatcher | Matcher::MinTypeMatcher(_) | Matcher::MaxTypeMatcher(_) => Ok(()),
           Matcher::EqualityMatcher => {
               if self == actual {
                   Ok(())
               } else {
                   Err(format!("Expected '{}' to be equal to '{}'", self, actual))
               }
           }
       }
    }
}

impl Matches<u64> for f64 {
    fn matches(&self, actual: &u64, matcher: &Matcher) -> Result<(), String> {
        debug!("comparing '{}' to {} using {:?}", self, actual, matcher);
        match *matcher {
           Matcher::RegexMatcher(ref regex) => {
               if regex.is_match(&actual.to_string()) {
                   Ok(())
               } else {
                   Err(format!("Expected '{}' to match '{}'", actual, regex))
               }
           },
           Matcher::TypeMatcher | Matcher::MinTypeMatcher(_) | Matcher::MaxTypeMatcher(_) => Err(
               format!("Expected '{}' (Decimal) to be the same type as '{}' (Integer)", self, actual)),
           Matcher::EqualityMatcher => Err(format!("Expected '{}' (Decimal) to be equal to '{}' (Integer)", self, actual))
       }
    }
}

fn select_best_matcher(path: &Vec<String>, matchers: &MatchingRules) -> Result<Matcher, String> {
    // let path_str = path.iter().join(".");
    // let matcher = match matchers.iter().max_by_key(|&(k, _)| calc_path_weight(k.clone(), path)) {
    //     Some(kv) => {
    //         match kv.1.get("match") {
    //             Some(val) => {
    //                 match val.as_str() {
    //                     "regex" => {
    //                         match kv.1.get("regex") {
    //                             Some(regex) => {
    //                                 match Regex::new(regex) {
    //                                     Ok(regex) => Ok(Matcher::RegexMatcher(regex)),
    //                                     Err(err) => {
    //                                         error!("Failed to compile regular expression '{}' provided for regex matcher for path '{}' - {}",
    //                                             regex, path_str, err);
    //                                         Err(format!("Failed to compile regular expression '{}' provided for regex matcher for path '{}' - {}",
    //                                             regex, path_str, err))
    //                                     }
    //                                 }
    //                             },
    //                             None => {
    //                                 error!("No regular expression provided for regex matcher for path '{}'",
    //                                     path_str);
    //                                 Err(format!("No regular expression provided for regex matcher for path '{}'",
    //                                     path_str))
    //                             }
    //                         }
    //                     },
    //                     "type" => if kv.1.contains_key("min") {
    //                         let min = kv.1.get("min").unwrap();
    //                         match min.parse() {
    //                             Ok(min) => Ok(Matcher::MinTypeMatcher(min)),
    //                             Err(err) => {
    //                                 warn!("Failed to parse minimum value '{}', defaulting to type matcher - {}", min, err);
    //                                 Ok(Matcher::TypeMatcher)
    //                             }
    //                         }
    //                     } else if kv.1.contains_key("max") {
    //                         let max = kv.1.get("max").unwrap();
    //                         match max.parse() {
    //                             Ok(max) => Ok(Matcher::MaxTypeMatcher(max)),
    //                             Err(err) => {
    //                                 warn!("Failed to parse maximum value '{}', defaulting to type matcher - {}", max, err);
    //                                 Ok(Matcher::TypeMatcher)
    //                             }
    //                         }
    //                     } else {
    //                         Ok(Matcher::TypeMatcher)
    //                     },
    //                     _ => {
    //                         warn!("Unrecognised matcher type '{}' for path '{}', defaulting to equality",
    //                             val, path_str);
    //                         Ok(Matcher::EqualityMatcher)
    //                     }
    //                 }
    //             },
    //             None => {
    //                 warn!("Matcher defined for path '{}' does not have an explicit 'match' attribute, falling back to equality, type or regular expression matching",
    //                     path_str);
    //                 if kv.1.contains_key("regex") {
    //                     let regex = kv.1.get("regex").unwrap();
    //                     match Regex::new(regex) {
    //                         Ok(regex) => Ok(Matcher::RegexMatcher(regex)),
    //                         Err(err) => {
    //                             error!("Failed to compile regular expression '{}' provided for regex matcher for path '{}' - {}",
    //                                 regex, path_str, err);
    //                             Err(format!("Failed to compile regular expression '{}' provided for regex matcher for path '{}' - {}",
    //                                 regex, path_str, err))
    //                         }
    //                     }
    //                 } else if kv.1.contains_key("min") {
    //                     let min = kv.1.get("min").unwrap();
    //                     match min.parse() {
    //                         Ok(min) => Ok(Matcher::MinTypeMatcher(min)),
    //                         Err(err) => {
    //                             warn!("Failed to parse minimum value '{}', defaulting to type matcher - {}", min, err);
    //                             Ok(Matcher::TypeMatcher)
    //                         }
    //                     }
    //                 } else if kv.1.contains_key("max") {
    //                     let max = kv.1.get("max").unwrap();
    //                     match max.parse() {
    //                         Ok(max) => Ok(Matcher::MaxTypeMatcher(max)),
    //                         Err(err) => {
    //                             warn!("Failed to parse maximum value '{}', defaulting to type matcher - {}", max, err);
    //                             Ok(Matcher::TypeMatcher)
    //                         }
    //                     }
    //                 } else {
    //                     error!("Invalid matcher definition {:?} for path '{}'", kv.1, path_str);
    //                     Err(format!("Invalid matcher definition {:?} for path '{}'", kv.1, path_str))
    //                 }
    //             }
    //         }
    //     },
    //     None => {
    //         warn!("Could not find an appropriate matcher for path '{}', defaulting to equality",
    //             path_str);
    //         Ok(Matcher::EqualityMatcher)
    //     }
    // };
    // debug!("Using Matcher for path '{}': {:?}", path_str, matcher);
    // matcher
    Err(s!("Not Implemented"))
}

pub fn match_values<E, A>(path: &Vec<String>, matchers: MatchingRules, expected: &E, actual: &A) -> Result<(), String>
    where E: Matches<A> {
    let matcher = select_best_matcher(path, &matchers);
    match matcher {
        Err(err) => Err(format!("Matcher for path '{}' is invalid - {}", path.iter().join("."), err)),
        Ok(ref matcher) => expected.matches(actual, matcher)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::select_best_matcher;
    use expectest::prelude::*;
    use regex::Regex;
    use models::matchingrules::*;

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

        expect!(select_best_matcher(&vec![s!("$")], &matchers)).to(be_ok().value(Matcher::RegexMatcher(Regex::new("1").unwrap())));
        expect!(select_best_matcher(&vec![s!("$"), s!("body")], &matchers)).to(be_ok().value(Matcher::RegexMatcher(Regex::new("2").unwrap())));
        expect!(select_best_matcher(&vec![s!("$"), s!("a")], &matchers)).to(be_ok().value(Matcher::RegexMatcher(Regex::new("1").unwrap())));

        expect!(select_best_matcher(&vec![s!("$"), s!("body"), s!("item1")], &matchers)).to(be_ok().value(Matcher::RegexMatcher(Regex::new("3").unwrap())));
        expect!(select_best_matcher(&vec![s!("$"), s!("body"), s!("item2")], &matchers)).to(be_ok().value(Matcher::RegexMatcher(Regex::new("4").unwrap())));
        expect!(select_best_matcher(&vec![s!("$"), s!("body"), s!("item3")], &matchers)).to(be_ok().value(Matcher::RegexMatcher(Regex::new("2").unwrap())));

        expect!(select_best_matcher(&vec![s!("$"), s!("header"), s!("item1")], &matchers)).to(be_ok().value(Matcher::RegexMatcher(Regex::new("5").unwrap())));

        expect!(select_best_matcher(&vec![s!("$"), s!("body"), s!("item1"), s!("level")], &matchers)).to(be_ok().value(Matcher::RegexMatcher(Regex::new("6").unwrap())));
        expect!(select_best_matcher(&vec![s!("$"), s!("body"), s!("item1"), s!("level"), s!("1")], &matchers)).to(be_ok().value(Matcher::RegexMatcher(Regex::new("7").unwrap())));
        expect!(select_best_matcher(&vec![s!("$"), s!("body"), s!("item1"), s!("level"), s!("2")], &matchers)).to(be_ok().value(Matcher::RegexMatcher(Regex::new("10").unwrap())));
        expect!(select_best_matcher(&vec![s!("$"), s!("body"), s!("item1"), s!("level"), s!("1"), s!("id")], &matchers)).to(be_ok().value(Matcher::RegexMatcher(Regex::new("8").unwrap())));
        expect!(select_best_matcher(&vec![s!("$"), s!("body"), s!("item1"), s!("level"), s!("1"), s!("name")], &matchers)).to(be_ok().value(Matcher::RegexMatcher(Regex::new("9").unwrap())));
        expect!(select_best_matcher(&vec![s!("$"), s!("body"), s!("item1"), s!("level"), s!("1"), s!("other")], &matchers)).to(be_ok().value(Matcher::RegexMatcher(Regex::new("7").unwrap())));
        expect!(select_best_matcher(&vec![s!("$"), s!("body"), s!("item1"), s!("level"), s!("2"), s!("id")], &matchers)).to(be_ok().value(Matcher::RegexMatcher(Regex::new("11").unwrap())));
        expect!(select_best_matcher(&vec![s!("$"), s!("body"), s!("item1"), s!("level"), s!("3"), s!("id")], &matchers)).to(be_ok().value(Matcher::RegexMatcher(Regex::new("12").unwrap())));
        expect!(select_best_matcher(&vec![s!("$"), s!("body"), s!("item2"), s!("level"), s!("1"), s!("id")], &matchers)).to(be_ok().value(Matcher::RegexMatcher(Regex::new("13").unwrap())));
        expect!(select_best_matcher(&vec![s!("$"), s!("body"), s!("item2"), s!("level"), s!("3"), s!("id")], &matchers)).to(be_ok().value(Matcher::RegexMatcher(Regex::new("13").unwrap())));
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

        expect!(select_best_matcher(&vec![s!("$"), s!("body"), s!("item1")], &matchers)).to(be_ok().value(Matcher::RegexMatcher(Regex::new("3").unwrap())));
        expect!(select_best_matcher(&vec![s!("$"), s!("body"), s!("item2")], &matchers)).to(be_ok().value(Matcher::MinTypeMatcher(4)));
        expect!(select_best_matcher(&vec![s!("$"), s!("body"), s!("item3")], &matchers)).to(be_ok().value(Matcher::MaxTypeMatcher(4)));
        expect!(select_best_matcher(&vec![s!("$"), s!("body"), s!("item4")], &matchers)).to(be_err());
    }

    #[test]
    fn equality_matcher_test() {
        let matcher = Matcher::EqualityMatcher;
        expect!(s!("100").matches(&s!("100"), &matcher)).to(be_ok());
        expect!(s!("100").matches(&s!("101"), &matcher)).to(be_err());
        expect!(s!("100").matches(&100, &matcher)).to(be_err());
        expect!(100.matches(&100, &matcher)).to(be_ok());
        expect!(100.matches(&100.0, &matcher)).to(be_err());
        expect!(100.1f64.matches(&100.0, &matcher)).to(be_err());
    }

    #[test]
    fn regex_matcher_test() {
        let matcher = Matcher::RegexMatcher(Regex::new("^\\d+$").unwrap());
        expect!(s!("100").matches(&s!("100"), &matcher)).to(be_ok());
        expect!(s!("100").matches(&s!("10a"), &matcher)).to(be_err());
        expect!(s!("100").matches(&100, &matcher)).to(be_ok());
        expect!(100.matches(&100, &matcher)).to(be_ok());
        expect!(100.matches(&100.01f64, &matcher)).to(be_err());
        expect!(100.1f64.matches(&100.02f64, &matcher)).to(be_err());
    }

    #[test]
    fn type_matcher_test() {
        let matcher = Matcher::TypeMatcher;
        expect!(s!("100").matches(&s!("100"), &matcher)).to(be_ok());
        expect!(s!("100").matches(&s!("10a"), &matcher)).to(be_ok());
        expect!(s!("100").matches(&100, &matcher)).to(be_err());
        expect!(100.matches(&200, &matcher)).to(be_ok());
        expect!(100.matches(&100.1, &matcher)).to(be_err());
        expect!(100.1f64.matches(&100.2, &matcher)).to(be_ok());
    }

    #[test]
    fn min_type_matcher_test() {
        let matcher = Matcher::MinTypeMatcher(3);
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
        let matcher = Matcher::MaxTypeMatcher(3);
        expect!(s!("100").matches(&s!("100"), &matcher)).to(be_ok());
        expect!(s!("100").matches(&s!("10a"), &matcher)).to(be_ok());
        expect!(s!("100").matches(&s!("1000"), &matcher)).to(be_ok());
        expect!(s!("100").matches(&100, &matcher)).to(be_err());
        expect!(100.matches(&200, &matcher)).to(be_ok());
        expect!(100.matches(&100.1, &matcher)).to(be_err());
        expect!(100.1f64.matches(&100.2, &matcher)).to(be_ok());
    }
}
