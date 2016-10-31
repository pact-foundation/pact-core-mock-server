use models::matchingrules::*;
use path_exp::*;
use itertools::Itertools;
use regex::Regex;

fn matches_token(path_fragment: &String, path_token: &PathToken) -> u32 {
    match *path_token {
        PathToken::Root if path_fragment == "$" => 2,
        PathToken::Field(ref name) if *path_fragment == name.clone() => 2,
        PathToken::Index(ref index) => match path_fragment.parse() {
            Ok(ref i) if index == i => 2,
            _ => 0
        },
        PathToken::StarIndex => match path_fragment.parse::<usize>() {
            Ok(_) => 1,
            _ => 0
        },
        PathToken::Star => 1,
        _ => 0
    }
}

fn calc_path_weight(path_exp: String, path: &Vec<String>) -> u32 {
    let weight = match parse_path_exp(path_exp.clone()) {
        Ok(path_tokens) => {
            debug!("Calculatint weight for path tokens '{:?}' and path '{:?}'", path_tokens, path);
            if path.len() >= path_tokens.len() {
                path_tokens.iter().zip(path.iter())
                    .fold(1, |acc, (token, fragment)| acc * matches_token(fragment, token))
            } else {
                0
            }
        },
        Err(err) => {
            warn!("Failed to parse path expression - {}", err);
            0
        }
    };
    debug!("Calculated weight {} for path '{}' and '{:?}'", weight, path_exp, path);
    weight
}

fn path_length(path_exp: String) -> usize {
    match parse_path_exp(path_exp.clone()) {
        Ok(path_tokens) => path_tokens.len(),
        Err(err) => {
            warn!("Failed to parse path expression - {}", err);
            0
        }
    }
}

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
    use super::{calc_path_weight, matches_token, select_best_matcher};
    use expectest::prelude::*;
    use path_exp::*;
    use regex::Regex;

    #[test]
    fn matcher_is_defined_returns_false_when_there_are_no_matchers() {
        let matchers = matchingrules!{};
        expect!(matchers.matcher_is_defined("body", &vec![s!("$"), s!("a"), s!("b")])).to(be_false());
    }

    #[test]
    fn matcher_is_defined_returns_false_when_the_path_does_not_have_a_matcher_entry() {
        let matchers = matchingrules!{
            "body" => {
            }
        };
        expect!(matchers.matcher_is_defined("body", &vec![s!("$"), s!("a"), s!("b")])).to(be_false());
    }

    #[test]
    fn matcher_is_defined_returns_true_when_the_path_does_have_a_matcher_entry() {
        let matchers = matchingrules!{
            "body" => {
                "$.a.b" => [ matchtype!() ]
            }
        };
        expect!(matchers.matcher_is_defined("body", &vec![s!("$"), s!("a"), s!("b")])).to(be_true());
    }

    #[test]
    fn matcher_is_defined_returns_true_when_the_parent_of_the_path_does_have_a_matcher_entry() {
        let matchers = matchingrules!{
            "body" => {
                "$.a.b" => [ matchtype!() ]
            }
        };
        expect!(matchers.matcher_is_defined("body", &vec![s!("$"), s!("a"), s!("b"), s!("c")])).to(be_true());
    }

    #[test]
    fn wildcard_matcher_is_defined_returns_false_when_there_are_no_matchers() {
        let matchers = matchingrules!{};
        expect!(matchers.wildcard_matcher_is_defined("body", &vec![s!("$"), s!("a"), s!("b")])).to(be_false());
    }

    #[test]
    fn wildcard_matcher_is_defined_returns_false_when_the_path_does_not_have_a_matcher_entry() {
        let matchers = matchingrules!{
            "body" => {

            }
        };
        expect!(matchers.wildcard_matcher_is_defined("body", &vec![s!("$"), s!("a"), s!("b")])).to(be_false());
    }

    #[test]
    fn wildcard_matcher_is_defined_returns_false_when_the_path_does_have_a_matcher_entry_and_it_is_not_a_wildcard() {
        let matchers = matchingrules!{
            "body" => {
                "$.a.b" => [ matchtype!() ],
                "$.*" => [ matchtype!() ]
            }
        };
        expect!(matchers.wildcard_matcher_is_defined("body", &vec![s!("$"), s!("a"), s!("b")])).to(be_false());
    }

    #[test]
    fn wildcard_matcher_is_defined_returns_true_when_the_path_does_have_a_matcher_entry_and_it_is_a_widcard() {
        let matchers = matchingrules!{
            "body" => {
                "$.a.*" => [ matchtype!() ]
            }
        };
        expect!(matchers.wildcard_matcher_is_defined("body", &vec![s!("$"), s!("a"), s!("b")])).to(be_true());
    }

    #[test]
    fn wildcard_matcher_is_defined_returns_false_when_the_parent_of_the_path_does_have_a_matcher_entry() {
        let matchers = matchingrules!{
            "body" => {
                "$.a.*" => [ matchtype!() ]
            }
        };
        expect!(matchers.wildcard_matcher_is_defined("body", &vec![s!("$"), s!("a"), s!("b"), s!("c")])).to(be_false());
    }

    #[test]
    fn matches_token_test_with_root() {
        expect!(matches_token(&s!("$"), &PathToken::Root)).to(be_equal_to(2));
        expect!(matches_token(&s!("path"), &PathToken::Root)).to(be_equal_to(0));
        expect!(matches_token(&s!("*"), &PathToken::Root)).to(be_equal_to(0));
    }

    #[test]
    fn matches_token_test_with_field() {
        expect!(matches_token(&s!("$"), &PathToken::Field(s!("path")))).to(be_equal_to(0));
        expect!(matches_token(&s!("path"), &PathToken::Field(s!("path")))).to(be_equal_to(2));
    }

    #[test]
    fn matches_token_test_with_index() {
        expect!(matches_token(&s!("$"), &PathToken::Index(2))).to(be_equal_to(0));
        expect!(matches_token(&s!("path"), &PathToken::Index(2))).to(be_equal_to(0));
        expect!(matches_token(&s!("*"), &PathToken::Index(2))).to(be_equal_to(0));
        expect!(matches_token(&s!("1"), &PathToken::Index(2))).to(be_equal_to(0));
        expect!(matches_token(&s!("2"), &PathToken::Index(2))).to(be_equal_to(2));
    }

    #[test]
    fn matches_token_test_with_index_wildcard() {
        expect!(matches_token(&s!("$"), &PathToken::StarIndex)).to(be_equal_to(0));
        expect!(matches_token(&s!("path"), &PathToken::StarIndex)).to(be_equal_to(0));
        expect!(matches_token(&s!("*"), &PathToken::StarIndex)).to(be_equal_to(0));
        expect!(matches_token(&s!("1"), &PathToken::StarIndex)).to(be_equal_to(1));
    }

    #[test]
    fn matches_token_test_with_wildcard() {
        expect!(matches_token(&s!("$"), &PathToken::Star)).to(be_equal_to(1));
        expect!(matches_token(&s!("path"), &PathToken::Star)).to(be_equal_to(1));
        expect!(matches_token(&s!("*"), &PathToken::Star)).to(be_equal_to(1));
        expect!(matches_token(&s!("1"), &PathToken::Star)).to(be_equal_to(1));
    }

    #[test]
    fn matches_path_matches_root_path_element() {
        expect!(calc_path_weight(s!("$"), &vec![s!("$")]) > 0).to(be_true());
        expect!(calc_path_weight(s!("$"), &vec![]) > 0).to(be_false());
    }

    #[test]
    fn matches_path_matches_field_name() {
        expect!(calc_path_weight(s!("$.name"), &vec![s!("$"), s!("name")]) > 0).to(be_true());
        expect!(calc_path_weight(s!("$['name']"), &vec![s!("$"), s!("name")]) > 0).to(be_true());
        expect!(calc_path_weight(s!("$.name.other"), &vec![s!("$"), s!("name"), s!("other")]) > 0).to(be_true());
        expect!(calc_path_weight(s!("$['name'].other"), &vec![s!("$"), s!("name"), s!("other")]) > 0).to(be_true());
        expect!(calc_path_weight(s!("$.name"), &vec![s!("$"), s!("other")]) > 0).to(be_false());
        expect!(calc_path_weight(s!("$.name"), &vec![s!("$"), s!("name"), s!("other")]) > 0).to(be_true());
        expect!(calc_path_weight(s!("$.other"), &vec![s!("$"), s!("name"), s!("other")]) > 0).to(be_false());
        expect!(calc_path_weight(s!("$.name.other"), &vec![s!("$"), s!("name")]) > 0).to(be_false());
    }

    #[test]
    fn matches_path_matches_array_indices() {
        expect!(calc_path_weight(s!("$[0]"), &vec![s!("$"), s!("0")]) > 0).to(be_true());
        expect!(calc_path_weight(s!("$.name[1]"), &vec![s!("$"), s!("name"), s!("1")]) > 0).to(be_true());
        expect!(calc_path_weight(s!("$.name"), &vec![s!("$"), s!("0")]) > 0).to(be_false());
        expect!(calc_path_weight(s!("$.name[1]"), &vec![s!("$"), s!("name"), s!("0")]) > 0).to(be_false());
        expect!(calc_path_weight(s!("$[1].name"), &vec![s!("$"), s!("name"), s!("1")]) > 0).to(be_false());
    }

    #[test]
    fn matches_path_matches_with_wildcard() {
        expect!(calc_path_weight(s!("$[*]"), &vec![s!("$"), s!("0")]) > 0).to(be_true());
        expect!(calc_path_weight(s!("$.*"), &vec![s!("$"), s!("name")]) > 0).to(be_true());
        expect!(calc_path_weight(s!("$.*.name"), &vec![s!("$"), s!("some"), s!("name")]) > 0).to(be_true());
        expect!(calc_path_weight(s!("$.name[*]"), &vec![s!("$"), s!("name"), s!("0")]) > 0).to(be_true());
        expect!(calc_path_weight(s!("$.name[*].name"), &vec![s!("$"), s!("name"), s!("1"), s!("name")]) > 0).to(be_true());
        expect!(calc_path_weight(s!("$[*]"), &vec![s!("$"), s!("name")]) > 0).to(be_false());
    }

    #[test]
    fn select_best_matcher_selects_most_appropriate_by_weight() {
        let matchers = matchingrules!{
            "body" => {
                "$" => [ matchregex!("1") ],
                "$.item1" => [ matchregex!("3") ],
                "$.item2" => [ matchregex!("4") ],
                "$.item1.level" => [ matchregex!("6") ],
                "$.item1.level[1]" => [ matchregex!("7") ],
                "$.item1.level[1].id" => [ matchregex!("8") ],
                "$.item1.level[1].name" => [ matchregex!("9") ],
                "$.item1.level[2]" => [ matchregex!("10") ],
                "$.item1.level[2].id" => [ matchregex!("11") ],
                "$.item1.level[*].id" => [ matchregex!("12") ],
                "$.*.level[*].id" => [ matchregex!("13") ]
            },
            "header" => {
                "item1" => [ matchregex!("5") ]
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
                "$.item1" => [ matchregex!("3") ],
                "$.item2" => [ matchmin!("4") ],
                "$.item3" => [ matchmax!("4") ],
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
