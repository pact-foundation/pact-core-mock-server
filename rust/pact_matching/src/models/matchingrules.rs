//! `matchingrules` module includes all the classes to deal with V3 format matchers

use rustc_serialize::json::Json;
use std::collections::{HashMap, BTreeMap, HashSet};
use std::hash::{Hash, Hasher};
use std::str::FromStr;

fn json_to_string(json: &Json) -> String {
  match json {
    &Json::String(ref s) => s.clone(),
    _ => json.to_string()
  }
}

fn json_to_num(json: Option<Json>) -> Option<usize> {
  if let Some(json) = json {
    match json {
      Json::I64(i) => if i > 0 { Some(i as usize) } else { None },
      Json::F64(f) => Some(f as usize),
      Json::U64(u) => Some(u as usize),
      Json::String(ref s) => usize::from_str(&s.clone()).ok(),
      _ => None
    }
  } else {
    None
  }
}

/// Set of all matching rules
#[derive(PartialEq, Debug, Clone, Eq)]
pub enum MatchingRule {
  /// Matcher using equals
  Equality,
  /// Match using a regular expression
  Regex(String),
  /// Match using the type of the value
  Type,
  /// Match using the type of the value and a minimum length for collections
  MinType(usize),
  /// Match using the type of the value and a maximum length for collections
  MaxType(usize),
  /// Match using the type of the value and a minimum and maximum length for collections
  MinMaxType(usize, usize),
  /// Match the value using a timestamp pattern
  Timestamp(String),
  /// Match the value using a time pattern
  Time(String),
  /// Match the value using a date pattern
  Date(String),
  /// Match if the value includes the given value
  Include(String),
  /// Match if the value is a number
  Number,
  /// Match if the value is an integer number
  Integer,
  /// Match if the value is a decimal number
  Decimal
}

impl MatchingRule {

  pub fn from_json(json: &Json) -> Option<MatchingRule> {
    match json {
      &Json::Object(ref m) => match m.get("match") {
        Some(json) => {
          let val = json_to_string(json);
          match val.as_str() {
            "regex" => match m.get(&val) {
              Some(s) => Some(MatchingRule::Regex(json_to_string(s))),
              None => None
            },
            "equality" => Some(MatchingRule::Equality),
            "include" => match m.get("value") {
              Some(s) => Some(MatchingRule::Include(json_to_string(s))),
              None => None
            },
            "type" => match (json_to_num(m.get("min").cloned()), json_to_num(m.get("max").cloned())) {
              (Some(min), Some(max)) => Some(MatchingRule::MinMaxType(min, max)),
              (Some(min), None) => Some(MatchingRule::MinType(min)),
              (None, Some(max)) => Some(MatchingRule::MaxType(max)),
              _ => Some(MatchingRule::Type)
            },
            "number" => Some(MatchingRule::Number),
            "integer" => Some(MatchingRule::Integer),
            "decimal" => Some(MatchingRule::Decimal),
            "real" => Some(MatchingRule::Decimal),
            "min" => match json_to_num(m.get(&val).cloned()) {
              Some(min) => Some(MatchingRule::MinType(min)),
              None => None
            },
            "max" => match json_to_num(m.get(&val).cloned()) {
              Some(max) => Some(MatchingRule::MaxType(max)),
              None => None
            },
            "timestamp" => match m.get(&val) {
              Some(s) => Some(MatchingRule::Timestamp(json_to_string(s))),
              None => None
            },
            "date" => match m.get(&val) {
              Some(s) => Some(MatchingRule::Date(json_to_string(s))),
              None => None
            },
            "time" => match m.get(&val) {
              Some(s) => Some(MatchingRule::Time(json_to_string(s))),
              None => None
            },
            _ => None
          }
        },
        None => if let Some(val) = m.get("regex") {
            Some(MatchingRule::Regex(json_to_string(val)))
          } else if let Some(val) = json_to_num(m.get("min").cloned()) {
            Some(MatchingRule::MinType(val))
          } else if let Some(val) = json_to_num(m.get("max").cloned()) {
            Some(MatchingRule::MaxType(val))
          } else if let Some(val) = m.get("timestamp") {
            Some(MatchingRule::Timestamp(json_to_string(val)))
          } else if let Some(val) = m.get("time") {
            Some(MatchingRule::Time(json_to_string(val)))
          } else if let Some(val) = m.get("date") {
            Some(MatchingRule::Date(json_to_string(val)))
          } else {
            None
          }
      },
      _ => None
    }
  }

}

/// Enumeration to define how to combine rules
#[derive(PartialEq, Debug, Clone, Eq)]
pub enum RuleLogic {
  /// All rules must match
  And,
  /// At least one rule must match
  Or
}

/// Data structure for representing a list of rules and the logic needed to combine them
#[derive(PartialEq, Debug, Clone, Eq)]
pub struct RuleList {
  /// List of rules to apply
  rules: Vec<MatchingRule>,
  /// Rule logic to use to evaluate multiple rules
  rule_logic: RuleLogic
}

impl RuleList {

  /// Creates a new rule list
  pub fn new(rule_logic: &RuleLogic) -> RuleList {
    RuleList {
      rules: Vec::new(),
      rule_logic: rule_logic.clone()
    }
  }

}

/// Data structure for representing a category of matching rules
#[derive(PartialEq, Debug, Clone, Eq)]
pub struct Category {
    /// Name of the category
    name: String,
    /// Matching rules for this category
    rules: HashMap<String, RuleList>
}

impl Category {

  /// Creates a default empty category
  pub fn default(name: String) -> Category {
      Category {
          name: name.clone(),
          rules: hashmap!{}
      }
  }

  /// If the matching rules in the category are empty
  pub fn is_empty(&self) -> bool {
    self.rules.is_empty()
  }

  /// If the matching rules in the category are not empty
  pub fn is_not_empty(&self) -> bool {
    !self.rules.is_empty()
  }

  /// Adds a rule from the JSON representation
  pub fn add_rule(&mut self, key: &String, matcher_json: &Json, rule_logic: &RuleLogic) {
    match MatchingRule::from_json(matcher_json) {
      Some(matching_rule) => {
        let mut rules = self.rules.entry(key.clone()).or_insert(RuleList::new(rule_logic));
        rules.rules.push(matching_rule);
      },
      None => warn!("Could not parse matcher {:?}", matcher_json)
    }
  }

}

/// Data structure for representing a collection of matchers
#[derive(PartialEq, Debug, Clone, Eq)]
pub struct MatchingRules {
    /// Categories of matching rules
    pub rules: HashMap<String, Category>
}

impl MatchingRules {

    /// Create a empty set of matching rules
    pub fn default() -> MatchingRules {
        MatchingRules {
            rules: hashmap!{}
        }
    }

    /// If the matching rules are empty (that is there are no rules assigned to any categories)
    pub fn is_empty(&self) -> bool {
        self.rules.values().all(|category| category.is_empty())
    }

    /// If the matching rules are not empty (that is there is at least one rule assigned to a category)
    pub fn is_not_empty(&self) -> bool {
      self.rules.values().any(|category| category.is_not_empty())
    }

    /// Adds the category to the map of rules
    pub fn add_category(&mut self, category: &String) -> &mut Category {
        if !self.rules.contains_key(category) {
            self.rules.insert(category.clone(), Category::default(category.clone()));
        }
        self.rules.get_mut(category).unwrap()
    }

    /// Returns all the category names in this rule set
    pub fn categories(&self) -> HashSet<String> {
      self.rules.keys().cloned().collect()
    }

    /// Returns the category of rules for a given category name
    pub fn rules_for_category(&self, category: &String) -> Option<Category> {
      self.rules.get(category).cloned()
    }

    /// If there is a matcher defined for the category and path
    pub fn matcher_is_defined(&self, category: &str, path: &Vec<String>) -> bool {
    //   match *matchers {
    //     Some(ref m) => !resolve_matchers(path, m).is_empty(),
    //     None => false
    //   }
    false
    }

    /// If there is a wildcard matcher defined for the category and path
    pub fn wildcard_matcher_is_defined(&self, category: &str, path: &Vec<String>) -> bool {
    //   match *matchers {
    //     Some(ref m) => m.iter().map(|(k, _)| k.clone())
    //       .filter(|k| calc_path_weight(k.clone(), path) > 0 && path_length(k.clone()) == path.len())
    //       .any(|k| k.ends_with(".*")),
    //     None => false
    //   }
    false
    }

    fn load_from_v2_map(&mut self, map: &BTreeMap<String, Json>) {
      for (key, v) in map {
        let path = key.split('.').map(|p| s!(p)).collect::<Vec<String>>();
        if key.starts_with("$.body") {
          if key == "$.body" {
            self.add_v2_rule(s!("body"), s!("$"), v);
          } else {
            self.add_v2_rule(s!("body"), format!("${}", s!(key[6..])), v);
          }
        } else if key.starts_with("$.headers") {
          self.add_v2_rule(s!("header"), path[2].clone(), v);
        } else {
          self.add_v2_rule(path[1].clone(), if path.len() > 2 { path[2].clone() } else { s!("") }, v);
        }
      }
    }

    fn load_from_v3_map(&mut self, map: &BTreeMap<String, Json>) {
      for (k, v) in map {
        self.add_rules(k, v);
      }
    }

    fn add_rules(&mut self, category_name: &String, rules: &Json) {
      let mut category = self.add_category(category_name);
      if category_name == "path" {
        let rule_logic = match rules.find("combine") {
          Some(val) => if json_to_string(val).to_uppercase() == "OR" {
              RuleLogic::Or
            } else {
              RuleLogic::And
            },
          None => RuleLogic::And
        };
        match rules.find("matchers") {
          Some(matchers) => match matchers {
            &Json::Array(ref array) => for matcher in array {
              category.add_rule(&s!(""), &matcher, &rule_logic)
            },
            _ => ()
          },
          None => ()
        }
      } else {
        match rules {
          &Json::Object(ref m) => {
            for (k, v) in m {
              let rule_logic = match v.find("combine") {
                Some(val) => if json_to_string(val).to_uppercase() == "OR" {
                  RuleLogic::Or
                } else {
                  RuleLogic::And
                },
                None => RuleLogic::And
              };
              match v.find("matchers") {
                Some(matchers) => match matchers {
                  &Json::Array(ref array) => for matcher in array {
                    category.add_rule(k, &matcher, &rule_logic)
                  },
                  _ => ()
                },
                None => ()
              }
            }
          },
          _ => ()
        }
      }
    }

  fn add_v2_rule(&mut self, category_name: String, sub_category: String, rule: &Json) {
    let mut category = self.add_category(&category_name);
    category.add_rule(&sub_category, rule, &RuleLogic::And);
  }
}

impl Hash for MatchingRules {
    fn hash<H: Hasher>(&self, state: &mut H) {

    }
}

/// Parses the matching rules from the Json structure
pub fn matchers_from_json(json: &Json, deprecated_name: &Option<String>) -> MatchingRules {
  let matchers_json = match (json.find("matchingRules"), deprecated_name.clone().and_then(|name| json.find(&name))) {
    (Some(v), _) => Some(v),
    (None, Some(v)) => Some(v),
    (None, None) => None
  };

  let mut matching_rules = MatchingRules::default();
  match matchers_json {
      Some(json) => match json {
        &Json::Object(ref m) => {
            if m.keys().next().unwrap_or(&s!("")).starts_with("$") {
                matching_rules.load_from_v2_map(m)
            } else {
                matching_rules.load_from_v3_map(m)
            }
        },
        _ => ()
      },
      None => ()
  }
  matching_rules
}

/// Generates a JSON structure for the provided matching rules
pub fn matchers_to_json(matchers: &MatchingRules) -> Json {
  // Json::Object(matchers.iter().fold(BTreeMap::new(), |mut map, kv| {
  //   map.insert(kv.0.clone(), Json::Object(kv.1.clone().iter().fold(BTreeMap::new(), |mut map, kv| {
  //     map.insert(kv.0.clone(), Json::String(kv.1.clone()));
  //     map
  //   })));
  //   map
  // }))
  Json::Null
}

// fn resolve_matchers(path: &Vec<String>, matchers: &MatchingRules) -> Matchers {
//   matchers.iter().map(|(k, v)| (k.clone(), v.clone()))
//     .filter(|kv| calc_path_weight(kv.0.clone(), path) > 0).collect()
// }


#[macro_export]
macro_rules! matchingrules {
    ( $( $name:expr => {
        $( $subname:expr => [ $( $matcher:expr ), * ] ),*
    }), * ) => {{
        let mut rules = $crate::models::matchingrules::MatchingRules::default();
        $({
            let mut category = rules.add_category(&$name.to_string());
        })*
        rules
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::{json_to_string, json_to_num};
    use expectest::prelude::*;
    use expectest::traits::IsEmpty;
    use rustc_serialize::json::Json;

    impl IsEmpty for MatchingRules {
      fn is_empty(&self) -> bool {
        self.is_empty()
      }
    }

    impl<'a> IsEmpty for &'a MatchingRules {
      fn is_empty(&self) -> bool {
        (*self).is_empty()
      }
    }

    #[test]
    fn rules_are_empty_when_there_are_no_categories() {
        expect!(MatchingRules::default()).to(be_empty());
    }

    #[test]
    fn rules_are_empty_when_there_are_only_empty_categories() {
        expect!(MatchingRules {
            rules: hashmap!{
                s!("body") => Category::default(s!("body")),
                s!("header") => Category::default(s!("header")),
                s!("query") => Category::default(s!("query")),
            }
        }).to(be_empty());
    }

    #[test]
    fn rules_are_not_empty_when_there_is_a_nonempty_category() {
        expect!(MatchingRules {
            rules: hashmap!{
                s!("body") => Category::default(s!("body")),
                s!("header") => Category::default(s!("headers")),
                s!("query") => Category {
                    name: s!("query"),
                    rules: hashmap!{
                      s!("") => RuleList {
                        rules: vec![ MatchingRule::Equality ],
                        rule_logic: RuleLogic::And
                      }
                    }
                },
            }
        }).to_not(be_empty());
    }

    #[test]
    fn matchers_from_json_test() {
        expect!(matchers_from_json(&Json::Null, &None)).to(be_empty());
    }

  #[test]
  fn loads_v2_matching_rules() {
    let matching_rules_json = Json::from_str(r#"{"matchingRules": {
      "$.path": { "match": "regex", "regex": "\\w+" },
      "$.query.Q1": { "match": "regex", "regex": "\\d+" },
      "$.header.HEADERY": {"match": "include", "value": "ValueA"},
      "$.body.animals": {"min": 1, "match": "type"},
      "$.body.animals[*].*": {"match": "type"},
      "$.body.animals[*].children": {"min": 1},
      "$.body.animals[*].children[*].*": {"match": "type"}
    }}"#).unwrap();

    let matching_rules = matchers_from_json(&matching_rules_json, &None);

    expect!(&matching_rules).to_not(be_empty());
    expect!(matching_rules.categories()).to(be_equal_to(hashset!{ s!("path"), s!("query"), s!("header"), s!("body") }));
    expect!(matching_rules.rules_for_category(&s!("path"))).to(be_some().value(Category {
      name: s!("path"),
      rules: hashmap! { s!("") => RuleList { rules: vec![ MatchingRule::Regex(s!("\\w+")) ], rule_logic: RuleLogic::And } }
    }));
    expect!(matching_rules.rules_for_category(&s!("query"))).to(be_some().value(Category {
      name: s!("query"),
      rules: hashmap!{ s!("Q1") => RuleList { rules: vec![ MatchingRule::Regex(s!("\\d+")) ], rule_logic: RuleLogic::And } }
    }));
    expect!(matching_rules.rules_for_category(&s!("header"))).to(be_some().value(Category {
      name: s!("header"),
      rules: hashmap!{ s!("HEADERY") => RuleList { rules: vec![
        MatchingRule::Include(s!("ValueA")) ], rule_logic: RuleLogic::And } }
    }));
    expect!(matching_rules.rules_for_category(&s!("body"))).to(be_some().value(Category {
      name: s!("body"),
      rules: hashmap!{
        s!("$.animals") => RuleList { rules: vec![ MatchingRule::MinType(1) ], rule_logic: RuleLogic::And },
        s!("$.animals[*].*") => RuleList { rules: vec![ MatchingRule::Type ], rule_logic: RuleLogic::And },
        s!("$.animals[*].children") => RuleList { rules: vec![ MatchingRule::MinType(1) ], rule_logic: RuleLogic::And },
        s!("$.animals[*].children[*].*") => RuleList { rules: vec![ MatchingRule::Type ], rule_logic: RuleLogic::And }
      }
    }));
  }

  #[test]
  fn loads_v3_matching_rules() {
    let matching_rules_json = Json::from_str(r#"{"matchingRules": {
      "path": {
        "matchers": [
          { "match": "regex", "regex": "\\w+" }
        ]
      },
      "query": {
        "Q1": {
          "matchers": [
            { "match": "regex", "regex": "\\d+" }
          ]
        }
      },
      "header": {
        "HEADERY": {
          "combine": "OR",
          "matchers": [
            {"match": "include", "value": "ValueA"},
            {"match": "include", "value": "ValueB"}
          ]
        }
      },
      "body": {
        "$.animals": {
          "matchers": [{"min": 1, "match": "type"}]
        },
        "$.animals[*].*": {
          "matchers": [{"match": "type"}]
        },
        "$.animals[*].children": {
          "matchers": [{"min": 1}]
        },
        "$.animals[*].children[*].*": {
          "matchers": [{"match": "type"}]
        }
      }
    }}"#).unwrap();

    let matching_rules = matchers_from_json(&matching_rules_json, &None);

    expect!(&matching_rules).to_not(be_empty());
    expect!(matching_rules.categories()).to(be_equal_to(hashset!{ s!("path"), s!("query"), s!("header"), s!("body") }));
    expect!(matching_rules.rules_for_category(&s!("path"))).to(be_some().value(Category {
      name: s!("path"),
      rules: hashmap! { s!("") => RuleList { rules: vec![ MatchingRule::Regex(s!("\\w+")) ], rule_logic: RuleLogic::And } }
    }));
    expect!(matching_rules.rules_for_category(&s!("query"))).to(be_some().value(Category {
      name: s!("query"),
      rules: hashmap!{ s!("Q1") => RuleList { rules: vec![ MatchingRule::Regex(s!("\\d+")) ], rule_logic: RuleLogic::And } }
    }));
    expect!(matching_rules.rules_for_category(&s!("header"))).to(be_some().value(Category {
      name: s!("header"),
      rules: hashmap!{ s!("HEADERY") => RuleList { rules: vec![
        MatchingRule::Include(s!("ValueA")),
        MatchingRule::Include(s!("ValueB")) ], rule_logic: RuleLogic::Or } }
    }));
    expect!(matching_rules.rules_for_category(&s!("body"))).to(be_some().value(Category {
      name: s!("body"),
      rules: hashmap!{
        s!("$.animals") => RuleList { rules: vec![ MatchingRule::MinType(1) ], rule_logic: RuleLogic::And },
        s!("$.animals[*].*") => RuleList { rules: vec![ MatchingRule::Type ], rule_logic: RuleLogic::And },
        s!("$.animals[*].children") => RuleList { rules: vec![ MatchingRule::MinType(1) ], rule_logic: RuleLogic::And },
        s!("$.animals[*].children[*].*") => RuleList { rules: vec![ MatchingRule::Type ], rule_logic: RuleLogic::And }
      }
    }));
  }

  #[test]
  fn json_to_string_test() {
    expect!(json_to_string(&Json::from_str("\"test string\"").unwrap())).to(be_equal_to(s!("test string")));
    expect!(json_to_string(&Json::from_str("null").unwrap())).to(be_equal_to(s!("null")));
    expect!(json_to_string(&Json::from_str("100").unwrap())).to(be_equal_to(s!("100")));
    expect!(json_to_string(&Json::from_str("100.10").unwrap())).to(be_equal_to(s!("100.1")));
    expect!(json_to_string(&Json::from_str("{}").unwrap())).to(be_equal_to(s!("{}")));
    expect!(json_to_string(&Json::from_str("[]").unwrap())).to(be_equal_to(s!("[]")));
    expect!(json_to_string(&Json::from_str("true").unwrap())).to(be_equal_to(s!("true")));
    expect!(json_to_string(&Json::from_str("false").unwrap())).to(be_equal_to(s!("false")));
  }

  #[test]
  fn json_to_num_test() {
    expect!(json_to_num(Json::from_str("\"test string\"").ok())).to(be_none());
    expect!(json_to_num(Json::from_str("null").ok())).to(be_none());
    expect!(json_to_num(Json::from_str("{}").ok())).to(be_none());
    expect!(json_to_num(Json::from_str("[]").ok())).to(be_none());
    expect!(json_to_num(Json::from_str("true").ok())).to(be_none());
    expect!(json_to_num(Json::from_str("false").ok())).to(be_none());
    expect!(json_to_num(Json::from_str("100").ok())).to(be_some().value(100));
    expect!(json_to_num(Json::from_str("-100").ok())).to(be_none());
    expect!(json_to_num(Json::from_str("100.10").ok())).to(be_some().value(100));
  }

  #[test]
  fn matching_rule_from_json_test() {
    expect!(MatchingRule::from_json(&Json::from_str("\"test string\"").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Json::from_str("null").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Json::from_str("{}").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Json::from_str("[]").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Json::from_str("true").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Json::from_str("false").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Json::from_str("100").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Json::from_str("100.10").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Json::from_str("{\"stuff\": 100}").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"stuff\"}").unwrap())).to(be_none());

    expect!(MatchingRule::from_json(&Json::from_str("{\"regex\": \"[0-9]\"}").unwrap())).to(
      be_some().value(MatchingRule::Regex(s!("[0-9]"))));
    expect!(MatchingRule::from_json(&Json::from_str("{\"min\": 100}").unwrap())).to(
      be_some().value(MatchingRule::MinType(100)));
    expect!(MatchingRule::from_json(&Json::from_str("{\"max\": 100}").unwrap())).to(
      be_some().value(MatchingRule::MaxType(100)));
    expect!(MatchingRule::from_json(&Json::from_str("{\"timestamp\": \"yyyy\"}").unwrap())).to(
      be_some().value(MatchingRule::Timestamp(s!("yyyy"))));
    expect!(MatchingRule::from_json(&Json::from_str("{\"date\": \"yyyy\"}").unwrap())).to(
      be_some().value(MatchingRule::Date(s!("yyyy"))));
    expect!(MatchingRule::from_json(&Json::from_str("{\"time\": \"hh:mm\"}").unwrap())).to(
      be_some().value(MatchingRule::Time(s!("hh:mm"))));

    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"regex\", \"regex\": \"[0-9]\"}").unwrap())).to(
      be_some().value(MatchingRule::Regex(s!("[0-9]"))));
    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"regex\"}").unwrap())).to(be_none());

    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"equality\"}").unwrap())).to(
      be_some().value(MatchingRule::Equality));

    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"include\", \"value\": \"A\"}").unwrap())).to(
      be_some().value(MatchingRule::Include(s!("A"))));
    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"include\"}").unwrap())).to(be_none());

    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"type\", \"min\": 1}").unwrap())).to(
      be_some().value(MatchingRule::MinType(1)));
    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"type\", \"max\": \"1\"}").unwrap())).to(
      be_some().value(MatchingRule::MaxType(1)));
    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"type\", \"min\": 1, \"max\": \"1\"}").unwrap())).to(
      be_some().value(MatchingRule::MinMaxType(1, 1)));
    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"type\"}").unwrap())).to(
      be_some().value(MatchingRule::Type));
    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"type\", \"value\": 100}").unwrap())).to(
      be_some().value(MatchingRule::Type));
    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"min\", \"min\": 1}").unwrap())).to(
      be_some().value(MatchingRule::MinType(1)));
    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"max\", \"max\": \"1\"}").unwrap())).to(
      be_some().value(MatchingRule::MaxType(1)));
    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"min\"}").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"max\"}").unwrap())).to(be_none());

    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"number\"}").unwrap())).to(
      be_some().value(MatchingRule::Number));
    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"integer\"}").unwrap())).to(
      be_some().value(MatchingRule::Integer));
    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"decimal\"}").unwrap())).to(
      be_some().value(MatchingRule::Decimal));
    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"real\"}").unwrap())).to(
      be_some().value(MatchingRule::Decimal));

    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"timestamp\", \"timestamp\": \"A\"}").unwrap())).to(
      be_some().value(MatchingRule::Timestamp(s!("A"))));
    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"timestamp\"}").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"time\", \"time\": \"A\"}").unwrap())).to(
      be_some().value(MatchingRule::Time(s!("A"))));
    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"time\"}").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"date\", \"date\": \"A\"}").unwrap())).to(
      be_some().value(MatchingRule::Date(s!("A"))));
    expect!(MatchingRule::from_json(&Json::from_str("{\"match\": \"date\"}").unwrap())).to(be_none());
  }
}
