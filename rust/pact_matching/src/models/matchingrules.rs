//! `matchingrules` module includes all the classes to deal with V3 format matchers

use std::{
  collections::{BTreeSet, HashMap, HashSet},
  hash::{Hash, Hasher}
};
use std::fmt::{Debug, Display};
#[allow(unused_imports)] // FromStr is actually used
use std::str::FromStr;

use log::*;
use maplit::*;
use onig::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{self, json, Value};
use serde_json::map::Map;

use crate::{MatchingContext, merge_result, Mismatch, DiffConfig};
use crate::matchers::{match_values, Matches};
use crate::models::json_utils::{json_to_num, json_to_string};
use crate::path_exp::*;

use super::PactSpecification;

fn matches_token(path_fragment: &str, path_token: &PathToken) -> usize {
  match path_token {
    PathToken::Root if path_fragment == "$" => 2,
    PathToken::Field(name) if path_fragment == name => 2,
    PathToken::Index(index) => match path_fragment.parse::<usize>() {
      Ok(i) if *index == i => 2,
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

pub(crate) fn calc_path_weight(path_exp: &str, path: &Vec<&str>) -> (usize, usize) {
  let weight = match parse_path_exp(path_exp) {
    Ok(path_tokens) => {
      trace!("Calculating weight for path tokens '{:?}' and path '{:?}'", path_tokens, path);
      if path.len() >= path_tokens.len() {
        (
          path_tokens.iter().zip(path.iter())
          .fold(1, |acc, (token, fragment)| acc * matches_token(fragment, token)),
         path_tokens.len()
        )
      } else {
        (0, path_tokens.len())
      }
    },
    Err(err) => {
      warn!("Failed to parse path expression - {}", err);
      (0, 0)
    }
  };
  trace!("Calculated weight {:?} for path '{}' and '{:?}'", weight, path_exp, path);
  weight
}

pub(crate) fn path_length(path_exp: &str) -> usize {
  match parse_path_exp(path_exp) {
    Ok(path_tokens) => path_tokens.len(),
    Err(err) => {
      warn!("Failed to parse path expression - {}", err);
      0
    }
  }
}

impl <T: Debug + Display + PartialEq> Matches<Vec<T>> for Vec<T> {
  fn matches(&self, actual: &Vec<T>, matcher: &MatchingRule) -> Result<(), String> {
    let result = match *matcher {
      MatchingRule::Regex(ref regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            let text: String = actual.iter().map(|v| v.to_string()).collect();
            if re.is_match(text.as_str()) {
              Ok(())
            } else {
              Err(format!("Expected '{}' to match '{}'", text, regex))
            }
          }
          Err(err) => Err(format!("'{}' is not a valid regular expression - {}", regex, err))
        }
      }
      MatchingRule::Type => Ok(()),
      MatchingRule::MinType(min) => {
        if actual.len() < min {
          Err(format!("Expected list with length {} to have a minimum length of {}", actual.len(), min))
        } else {
          Ok(())
        }
      }
      MatchingRule::MaxType(max) => {
        if actual.len() > max {
          Err(format!("Expected list with length {} to have a maximum length of {}", actual.len(), max))
        } else {
          Ok(())
        }
      }
      MatchingRule::MinMaxType(min, max) => {
        if actual.len() < min {
          Err(format!("Expected list with length {} to have a minimum length of {}", actual.len(), min))
        } else if actual.len() > max {
          Err(format!("Expected list with length {} to have a maximum length of {}", actual.len(), max))
        } else {
          Ok(())
        }
      }
      MatchingRule::Equality => {
        if self == actual {
          Ok(())
        } else {
          Err(format!("Expected {:?} to be equal to {:?}", actual, self))
        }
      }
      _ => Err(format!("Unable to match {:?} using {:?}", self, matcher))
    };
    log::debug!("Comparing '{:?}' to '{:?}' using {:?} -> {:?}", self, actual, matcher, result);
    result
  }
}

pub trait DisplayForMismatch {
  fn for_mismatch(&self) -> String;
}

impl <T: Display> DisplayForMismatch for HashMap<String, T> {
  fn for_mismatch(&self) -> String {
    Value::Object(self.iter().map(|(k, v)| (k.clone(), json!(v.to_string()))).collect()).to_string()
  }
}

impl <T: Display> DisplayForMismatch for Vec<T> {
  fn for_mismatch(&self) -> String {
    Value::Array(self.iter().map(|v| json!(v.to_string())).collect()).to_string()
  }
}

/// Set of all matching rules
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Eq, Hash, PartialOrd, Ord)]
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
  Decimal,
  /// Match if the value is a null value (this is content specific, for JSON will match a JSON null)
  Null,
  /// Match binary data by its content type (magic file check)
  ContentType(String)
}

impl MatchingRule {

  /// Builds a `MatchingRule` from a `Value` struct
  pub fn from_json(value: &Value) -> Option<MatchingRule> {
    match value {
      &Value::Object(ref m) => match m.get("match") {
        Some(value) => {
          let val = json_to_string(value);
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
            "timestamp" => match m.get("format").or_else(|| m.get(&val)) {
              Some(s) => Some(MatchingRule::Timestamp(json_to_string(s))),
              None => None
            },
            "date" => match m.get("format").or_else(|| m.get(&val)) {
              Some(s) => Some(MatchingRule::Date(json_to_string(s))),
              None => None
            },
            "time" => match m.get("format").or_else(|| m.get(&val)) {
              Some(s) => Some(MatchingRule::Time(json_to_string(s))),
              None => None
            },
            "null" => Some(MatchingRule::Null),
            "contentType" => match m.get("value") {
              Some(s) => Some(MatchingRule::ContentType(json_to_string(s))),
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

  /// Builds a `MatchingRule` from a `Value` struct used by language integrations
  pub fn from_integration_json(m: &Map<String, Value>) -> Option<MatchingRule> {
    match m.get("pact:matcher:type") {
      Some(value) => {
        let val = json_to_string(value);
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
          "timestamp" => match m.get("format").or_else(|| m.get(&val)) {
            Some(s) => Some(MatchingRule::Timestamp(json_to_string(s))),
            None => None
          },
          "date" => match m.get("format").or_else(|| m.get(&val)) {
            Some(s) => Some(MatchingRule::Date(json_to_string(s))),
            None => None
          },
          "time" => match m.get("format").or_else(|| m.get(&val)) {
            Some(s) => Some(MatchingRule::Time(json_to_string(s))),
            None => None
          },
          "null" => Some(MatchingRule::Null),
          "contentType" => match m.get("value") {
            Some(s) => Some(MatchingRule::ContentType(json_to_string(s))),
            None => None
          }
          _ => None
        }
      },
      _ => None
    }
  }

  /// Converts this `MatchingRule` to a `Value` struct
  pub fn to_json(&self) -> Value {
    match *self {
      MatchingRule::Equality => json!({ "match": Value::String(s!("equality")) }),
      MatchingRule::Regex(ref r) => json!({ "match": Value::String(s!("regex")),
        "regex": Value::String(r.clone()) }),
      MatchingRule::Type => json!({ "match": Value::String(s!("type")) }),
      MatchingRule::MinType(min) => json!({ "match": Value::String(s!("type")),
        "min": json!(min as u64) }),
      MatchingRule::MaxType(max) => json!({ "match": Value::String(s!("type")),
        "max": json!(max as u64) }),
      MatchingRule::MinMaxType(min, max) => json!({ "match": Value::String(s!("type")),
        "min": json!(min as u64), "max": json!(max as u64) }),
      MatchingRule::Timestamp(ref t) => json!({ "match": Value::String(s!("timestamp")),
        "timestamp": Value::String(t.clone()) }),
      MatchingRule::Time(ref t) => json!({ s!("match"): Value::String(s!("time")),
        s!("time"): Value::String(t.clone()) }),
      MatchingRule::Date(ref d) => json!({ s!("match"): Value::String(s!("date")),
        s!("date"): Value::String(d.clone()) }),
      MatchingRule::Include(ref s) => json!({ "match": Value::String(s!("include")),
        "value": Value::String(s.clone()) }),
      MatchingRule::Number => json!({ "match": Value::String(s!("number")) }),
      MatchingRule::Integer => json!({ "match": Value::String(s!("integer")) }),
      MatchingRule::Decimal => json!({ "match": Value::String(s!("decimal")) }),
      MatchingRule::Null => json!({ "match": Value::String(s!("null")) }),
      MatchingRule::ContentType(ref r) => json!({ "match": Value::String(s!("contentType")),
        "value": Value::String(r.clone()) })
    }
  }

  /// Create a new matching context for the matching rule defined at the given path. May just return
  /// a clone of the current context.
  pub fn matcher_context(&self, path: &Vec<&str>, current_context: &MatchingContext) -> MatchingContext {
    current_context.clone()
  }

  /// Delegate to the matching rule define at the given path to compare the key/value maps.
  pub fn compare_maps<T: Display + Debug>(&self, path: &Vec<&str>, expected: &HashMap<String, T>, actual: &HashMap<String, T>,
                                  context: &MatchingContext,
                                  callback: &mut dyn FnMut(&Vec<&str>, &T, &T) -> Result<(), Vec<Mismatch>>) -> Result<(), Vec<Mismatch>> {
    let mut p = path.to_vec();
    p.push("any");
    let mut result = Ok(());
    if context.wildcard_matcher_is_defined(&p) {
      for (key, value) in actual.iter() {
        let mut p = path.to_vec();
        p.push(key);
        if expected.contains_key(key) {
          result = merge_result(result, callback(&p, &expected[key], value));
        } else if !expected.is_empty() {
          result = merge_result(result, callback(&p, &expected.values().next().unwrap(), value));
        }
      }
    } else {
      result = merge_result(result, context.match_keys(path, &expected, &actual));
      for (key, value) in expected.iter() {
        if actual.contains_key(key) {
          let mut p = path.to_vec();
          p.push(key);
          result = merge_result(result, callback(&p, value, &actual[key]));
        } else {
          result = merge_result(result, Err(vec![ Mismatch::BodyMismatch { path: path.join("."),
            expected: Some(expected.for_mismatch().into()),
            actual: Some(actual.for_mismatch().into()),
            mismatch: format!("Expected entry {}={} but was missing", key, value.to_string())} ]));
        }
      }
    }
    result
  }

  pub fn compare_lists<T: Display + Debug + PartialEq>(
    &self,
    path: &Vec<&str>,
    expected: &Vec<T>,
    actual: &Vec<T>,
    context: &MatchingContext,
    context_stack: &mut Vec<MatchingContext>,
    callback: &dyn Fn(&Vec<&str>, &T, &T, &mut Vec<MatchingContext>) -> Result<(), Vec<Mismatch>>
  ) -> Result<(), Vec<Mismatch>> {
    let mut result = Ok(());

    if let Err(messages) = match_values(path, context, expected, actual) {
      for message in messages {
        result = merge_result(result,Err(vec![ Mismatch::BodyMismatch {
          path: path.join("."),
          expected: Some(expected.for_mismatch().into()),
          actual: Some(actual.for_mismatch().into()),
          mismatch: message.clone()
        } ]));
      }
    }
    let expected_example = expected.first().unwrap().clone();
    let mut expected_list = Vec::new();
    expected_list.resize(actual.len(), expected_example);

    for (index, value) in expected_list.iter().enumerate() {
      let ps = index.to_string();
      log::debug!("Comparing list item {} with value '{:?}' to '{:?}'", index, actual.get(index), value);
      let mut p = path.to_vec();
      p.push(ps.as_str());
      if index < actual.len() {
        result = merge_result(result, callback(&p, value, &actual[index], context_stack));
      } else if !context.matcher_is_defined(&p) {
        result = merge_result(result,Err(vec![ Mismatch::BodyMismatch { path: path.join("."),
          expected: Some(expected.for_mismatch().into()),
          actual: Some(actual.for_mismatch().into()),
          mismatch: format!("Expected {} but was missing", value) } ]))
      }
    }
    result
  }
}

/// Enumeration to define how to combine rules
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Eq, Hash)]
pub enum RuleLogic {
  /// All rules must match
  And,
  /// At least one rule must match
  Or
}

impl RuleLogic {

  fn to_json(&self) -> Value {
    Value::String(match self {
      &RuleLogic::And => s!("AND"),
      &RuleLogic::Or => s!("OR")
    })
  }

}

/// Data structure for representing a list of rules and the logic needed to combine them
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Eq, Hash)]
pub struct RuleList {
  /// List of rules to apply
  pub rules: BTreeSet<MatchingRule>,
  /// Rule logic to use to evaluate multiple rules
  pub rule_logic: RuleLogic
}

impl RuleList {

  /// Creates a new empty rule list
  pub fn default(rule_logic: &RuleLogic) -> RuleList {
    RuleList {
      rules: BTreeSet::new(),
      rule_logic: rule_logic.clone()
    }
  }

  /// Creates a new rule list with the single matching rule
  pub fn new(rule: MatchingRule) -> RuleList {
    RuleList {
      rules: btreeset![ rule ],
      rule_logic: RuleLogic::And
    }
  }

  fn to_v3_json(&self) -> Value {
    json!({
      s!("combine"): self.rule_logic.to_json(),
      s!("matchers"): Value::Array(self.rules.iter().map(|matcher| matcher.to_json()).collect())
    })
  }

  fn to_v2_json(&self) -> Value {
    match self.rules.iter().next() {
      Some(rule) => rule.to_json(),
      None => json!({})
    }
  }

  /// If there is a type matcher defined for the rule list
  pub fn type_matcher_defined(&self) -> bool {
    self.rules.iter().any(|rule| match rule {
      MatchingRule::Type => true,
      MatchingRule::MinType(_) => true,
      MatchingRule::MaxType(_) => true,
      MatchingRule::MinMaxType(_, _) => true,
      _ => false
    })
  }
}

/// Data structure for representing a category of matching rules
#[derive(Serialize, Deserialize, Debug, Clone, Eq, Default)]
pub struct MatchingRuleCategory {
  /// Name of the category
  pub name: String,
  /// Matching rules for this category
  pub rules: HashMap<String, RuleList>
}

impl MatchingRuleCategory {
  /// Creates a default empty category
  pub fn default<S>(name: S) -> MatchingRuleCategory
    where S: Into<String>
  {
    MatchingRuleCategory {
      name: name.into(),
      rules: hashmap! {},
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

  /// Adds a rule from the Value representation
  pub fn rule_from_json(&mut self, key: &String, matcher_json: &Value, rule_logic: &RuleLogic) {
    match MatchingRule::from_json(matcher_json) {
      Some(matching_rule) => {
        let rules = self.rules.entry(key.clone()).or_insert(RuleList::default(rule_logic));
        rules.rules.insert(matching_rule);
      },
      None => log::warn!("Could not parse matcher {:?}", matcher_json)
    }
  }

  /// Adds a rule to this category
  pub fn add_rule<S>(&mut self, key: S, matcher: MatchingRule, rule_logic: &RuleLogic)
    where S: Into<String> {
    let rules = self.rules.entry(key.into()).or_insert(RuleList::default(rule_logic));
    rules.rules.insert(matcher);
  }

  /// Filters the matchers in the category by the predicate, and returns a new category
  pub fn filter<F>(&self, predicate: F) -> MatchingRuleCategory
    where F : Fn(&(&String, &RuleList)) -> bool {
    MatchingRuleCategory {
      name: self.name.clone(),
      rules: self.rules.iter().filter(predicate)
        .map(|(path, rules)| (path.clone(), rules.clone())).collect()
    }
  }

  fn max_by_path(&self, path: &Vec<&str>) -> Option<RuleList> {
    self.rules.iter().map(|(k, v)| (k, v, calc_path_weight(k.as_str(), path)))
      .filter(|&(_, _, w)| w.0 > 0)
      .max_by_key(|&(_, _, w)| w.0 * w.1)
      .map(|(_, v, _)| v.clone())
  }

  /// Returns a JSON Value representation in V3 format
  pub fn to_v3_json(&self) -> Value {
    Value::Object(self.rules.iter().fold(serde_json::Map::new(), |mut map, (category, rulelist)| {
      map.insert(category.clone(), rulelist.to_v3_json());
      map
    }))
  }

  /// Returns a JSON Value representation in V2 format
  pub fn to_v2_json(&self) -> HashMap<String, Value> {
    let mut map = hashmap!{};

    if self.name == "body" {
      for (k, v) in self.rules.clone() {
        map.insert(k.replace("$", "$.body"), v.to_v2_json());
      }
    } else {
      for (k, v) in self.rules.clone() {
        map.insert(format!("$.{}.{}", self.name, k), v.to_v2_json());
      }
    }

    map
  }

  /// If there is a type matcher defined for the category
  pub fn type_matcher_defined(&self) -> bool {
    self.rules.values().any(|rule_list| rule_list.type_matcher_defined())
  }

  /// If there is a matcher defined for the path
  pub fn matcher_is_defined(&self, path: &Vec<&str>) -> bool {
    let result = !self.resolve_matchers_for_path(path).is_empty();
    trace!("matcher_is_defined for category {} and path {:?} -> {}", self.name, path, result);
    result
  }

  /// filters this category with all rules that match the given path for categories that contain
  /// collections (bodies, headers, query parameters). Returns self otherwise.
  pub fn resolve_matchers_for_path(&self, path: &Vec<&str>) -> MatchingRuleCategory {
    if self.name == "body" || self.name == "header" || self.name == "query" {
      self.filter(|(val, _)| {
        calc_path_weight(val, path).0 > 0
      })
    } else {
      self.clone()
    }
  }

  /// Selects the best matcher for the given path by calculating a weighting for each one
  pub fn select_best_matcher(&self, path: &Vec<&str>) -> Option<RuleList> {
    if self.name == "body" {
      self.max_by_path(path)
    } else {
      self.resolve_matchers_for_path(path).as_rule_list()
    }
  }

  /// Returns this category as a matching rule list. Returns a None if there are no rules
  pub fn as_rule_list(&self) -> Option<RuleList> {
    self.rules.values().next().cloned()
  }
}

impl Hash for MatchingRuleCategory {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.name.hash(state);
    for (k, v) in self.rules.clone() {
      k.hash(state);
      v.hash(state);
    }
  }
}

impl PartialEq for MatchingRuleCategory {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name && self.rules == other.rules
  }

  fn ne(&self, other: &Self) -> bool {
    self.name != other.name || self.rules != other.rules
  }
}

/// Data structure for representing a collection of matchers
#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
#[serde(transparent)]
pub struct MatchingRules {
    /// Categories of matching rules
    pub rules: HashMap<String, MatchingRuleCategory>
}

impl MatchingRules {

    /// If the matching rules are empty (that is there are no rules assigned to any categories)
    pub fn is_empty(&self) -> bool {
        self.rules.values().all(|category| category.is_empty())
    }

    /// If the matching rules are not empty (that is there is at least one rule assigned to a category)
    pub fn is_not_empty(&self) -> bool {
      self.rules.values().any(|category| category.is_not_empty())
    }

    /// Adds the category to the map of rules
    pub fn add_category<S>(&mut self, category: S) -> &mut MatchingRuleCategory
      where S: Into<String>
    {
      let category = category.into();
      if !self.rules.contains_key(&category) {
          self.rules.insert(category.clone(), MatchingRuleCategory::default(category.clone()));
      }
      self.rules.get_mut(&category).unwrap()
    }

    /// Returns all the category names in this rule set
    pub fn categories(&self) -> HashSet<String> {
      self.rules.keys().cloned().collect()
    }

    /// Returns the category of rules for a given category name
    pub fn rules_for_category(&self, category: &str) -> Option<MatchingRuleCategory> {
      self.rules.get(category).cloned()
    }

    /// If there is a matcher defined for the category and path
    pub fn matcher_is_defined(&self, category: &str, path: &Vec<&str>) -> bool {
      let result = match self.resolve_matchers(category, path) {
        Some(ref category) => !category.is_empty(),
        None => false
      };
      trace!("matcher_is_defined for category {} and path {:?} -> {}", category, path, result);
      result
    }

    /// If there is a wildcard matcher defined for the category and path
    pub fn wildcard_matcher_is_defined(&self, category: &str, path: &Vec<&str>) -> bool {
      match self.resolve_wildcard_matchers(category, path) {
        Some(ref category) => !category.filter(|&(val, _)| val.ends_with(".*")).is_empty(),
        None => false
      }
    }

  /// If there is a type matcher defined for the category and path
  pub fn type_matcher_defined(&self, category: &str, path: &Vec<&str>) -> bool {
    let result = match self.resolve_matchers(category, path) {
      Some(ref category) => category.type_matcher_defined(),
      None => false
    };
    trace!("type_matcher_defined for category {} and path {:?} -> {}", category, path, result);
    result
  }

  /// Returns a `Category` filtered with all rules that match the given path.
  pub fn resolve_matchers(&self, category: &str, path: &Vec<&str>) -> Option<MatchingRuleCategory> {
    self.rules_for_category(category)
      .map(|category| category.resolve_matchers_for_path(path))
  }

    /// Returns a list of rules from the body category that match the given path
    pub fn resolve_body_matchers_by_path(&self, path: &Vec<&str>) -> Option<RuleList> {
      match self.rules_for_category(&s!("body")) {
        Some(category) => category.max_by_path(path),
        None => None
      }
    }

    fn resolve_wildcard_matchers(&self, category: &str, path: &Vec<&str>) -> Option<MatchingRuleCategory> {
      if category == "body" {
        self.rules_for_category(category).map(|category| category.filter(|&(val, _)| {
          calc_path_weight(val, path).0 > 0 && path_length(val) == path.len()
        }))
      } else if category == "header" || category == "query" {
        self.rules_for_category(category).map(|category| category.filter(|&(val, _)| {
          path.len() == 1 && path[0] == *val
        }))
      } else {
        self.rules_for_category(category)
      }
    }

    fn load_from_v2_map(&mut self, map: &serde_json::Map<String, Value>) {
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

    fn load_from_v3_map(&mut self, map: &serde_json::Map<String, Value>) {
      for (k, v) in map {
        self.add_rules(k, v);
      }
    }

    fn add_rules(&mut self, category_name: &String, rules: &Value) {
      let category = self.add_category(category_name.clone());
      if category_name == "path" && rules.get("matchers").is_some() {
        let rule_logic = match rules.get("combine") {
          Some(val) => if json_to_string(val).to_uppercase() == "OR" {
              RuleLogic::Or
            } else {
              RuleLogic::And
            },
          None => RuleLogic::And
        };
        match rules.get("matchers") {
          Some(matchers) => match matchers {
            &Value::Array(ref array) => for matcher in array {
              category.rule_from_json(&s!(""), &matcher, &rule_logic)
            },
            _ => ()
          },
          None => ()
        }
      } else {
        match rules {
          &Value::Object(ref m) => {
            for (k, v) in m {
              let rule_logic = match v.get("combine") {
                Some(val) => if json_to_string(val).to_uppercase() == "OR" {
                  RuleLogic::Or
                } else {
                  RuleLogic::And
                },
                None => RuleLogic::And
              };
              match v.get("matchers") {
                Some(matchers) => match matchers {
                  &Value::Array(ref array) => for matcher in array {
                    category.rule_from_json(k, &matcher, &rule_logic)
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

  fn add_v2_rule(&mut self, category_name: String, sub_category: String, rule: &Value) {
    let category = self.add_category(category_name);
    category.rule_from_json(&sub_category, rule, &RuleLogic::And);
  }

  fn to_v3_json(&self) -> Value {
    Value::Object(self.rules.iter().fold(serde_json::Map::new(), |mut map, (name, sub_category)| {
      if name == "path" {
        if let Some(rules) = sub_category.rules.get("") {
          map.insert(name.clone(), rules.to_v3_json());
        }
      } else {
        map.insert(name.clone(), sub_category.to_v3_json());
      }
      map
    }))
  }

  fn to_v2_json(&self) -> Value {
    Value::Object(self.rules.iter().fold(serde_json::Map::new(), |mut map, (_, category)| {
      for (key, value) in category.to_v2_json() {
        map.insert(key.clone(), value);
      }
      map
    }))
  }
}

impl Hash for MatchingRules {
  fn hash<H: Hasher>(&self, state: &mut H) {
    for (k, v) in self.rules.iter() {
      k.hash(state);
      v.hash(state);
    }
  }
}

impl PartialEq for MatchingRules {
  fn eq(&self, other: &Self) -> bool {
    self.rules == other.rules
  }

  fn ne(&self, other: &Self) -> bool {
    self.rules != other.rules
  }
}

impl Default for MatchingRules {
  fn default() -> Self {
    MatchingRules {
      rules: hashmap!{}
    }
  }
}

/// Parses the matching rules from the Value structure
pub fn matchers_from_json(value: &Value, deprecated_name: &Option<String>) -> MatchingRules {
  let matchers_json = match (value.get("matchingRules"), deprecated_name.clone().and_then(|name| value.get(&name))) {
    (Some(v), _) => Some(v),
    (None, Some(v)) => Some(v),
    (None, None) => None
  };

  let mut matching_rules = MatchingRules::default();
  match matchers_json {
      Some(value) => match value {
        &Value::Object(ref m) => {
            if m.keys().next().unwrap_or(&String::default()).starts_with("$") {
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

/// Generates a Value structure for the provided matching rules
pub fn matchers_to_json(matchers: &MatchingRules, spec_version: &PactSpecification) -> Value {
   match spec_version {
     &PactSpecification::V3 | &PactSpecification::V4 => matchers.to_v3_json(),
     _ => matchers.to_v2_json()
   }
}

/// Macro to ease constructing matching rules
/// Example usage:
/// ```ignore
/// matchingrules! {
///   "query" => { "user_id" => [ MatchingRule::Regex(s!("^[0-9]+$")) ] }
/// }
/// ```
#[macro_export]
macro_rules! matchingrules {
    ( $( $name:expr => {
        $( $subname:expr => [ $( $matcher:expr ), * ] ),*
    }), * ) => {{
        let mut _rules = $crate::models::matchingrules::MatchingRules::default();
        $({
            let mut _category = _rules.add_category($name);
            $({
              $({
                _category.add_rule(&$subname.to_string(), $matcher, &RuleLogic::And);
              })*
            })*
        })*
        _rules
    }};
}

#[cfg(test)]
mod tests {
  use expectest::expect;
  use expectest::prelude::*;
  use serde_json::Value;
  use speculate::speculate;

  use super::*;
  use super::{calc_path_weight, matches_token};

  #[test]
    fn rules_are_empty_when_there_are_no_categories() {
        expect!(MatchingRules::default().is_empty()).to(be_true());
    }

    #[test]
    fn rules_are_empty_when_there_are_only_empty_categories() {
        expect!(MatchingRules {
            rules: hashmap!{
                s!("body") => MatchingRuleCategory::default(s!("body")),
                s!("header") => MatchingRuleCategory::default(s!("header")),
                s!("query") => MatchingRuleCategory::default(s!("query")),
            }
        }.is_empty()).to(be_true());
    }

    #[test]
    fn rules_are_not_empty_when_there_is_a_nonempty_category() {
        expect!(MatchingRules {
            rules: hashmap!{
                s!("body") => MatchingRuleCategory::default(s!("body")),
                s!("header") => MatchingRuleCategory::default(s!("headers")),
                s!("query") => MatchingRuleCategory {
                    name: s!("query"),
                    rules: hashmap!{
                      s!("") => RuleList {
                        rules: btreeset![ MatchingRule::Equality ],
                        rule_logic: RuleLogic::And
                      }
                    }
                },
            }
        }.is_empty()).to(be_false());
    }

  #[test]
  fn matchers_from_json_test() {
      expect!(matchers_from_json(&Value::Null, &None).rules.iter()).to(be_empty());
  }

  #[test]
  fn loads_v2_matching_rules() {
    let matching_rules_json = Value::from_str(r#"{"matchingRules": {
      "$.path": { "match": "regex", "regex": "\\w+" },
      "$.query.Q1": { "match": "regex", "regex": "\\d+" },
      "$.header.HEADERY": {"match": "include", "value": "ValueA"},
      "$.body.animals": {"min": 1, "match": "type"},
      "$.body.animals[*].*": {"match": "type"},
      "$.body.animals[*].children": {"min": 1},
      "$.body.animals[*].children[*].*": {"match": "type"}
    }}"#).unwrap();

    let matching_rules = matchers_from_json(&matching_rules_json, &None);

    expect!(matching_rules.rules.iter()).to_not(be_empty());
    expect!(matching_rules.categories()).to(be_equal_to(hashset!{ s!("path"), s!("query"), s!("header"), s!("body") }));
    expect!(matching_rules.rules_for_category(&s!("path"))).to(be_some().value(MatchingRuleCategory {
      name: s!("path"),
      rules: hashmap! { s!("") => RuleList { rules: btreeset![ MatchingRule::Regex(s!("\\w+")) ], rule_logic: RuleLogic::And } }
    }));
    expect!(matching_rules.rules_for_category(&s!("query"))).to(be_some().value(MatchingRuleCategory {
      name: s!("query"),
      rules: hashmap!{ s!("Q1") => RuleList { rules: btreeset![ MatchingRule::Regex(s!("\\d+")) ], rule_logic: RuleLogic::And } }
    }));
    expect!(matching_rules.rules_for_category(&s!("header"))).to(be_some().value(MatchingRuleCategory {
      name: s!("header"),
      rules: hashmap!{ s!("HEADERY") => RuleList { rules: btreeset![
        MatchingRule::Include(s!("ValueA")) ], rule_logic: RuleLogic::And } }
    }));
    expect!(matching_rules.rules_for_category(&s!("body"))).to(be_some().value(MatchingRuleCategory {
      name: s!("body"),
      rules: hashmap!{
        s!("$.animals") => RuleList { rules: btreeset![ MatchingRule::MinType(1) ], rule_logic: RuleLogic::And },
        s!("$.animals[*].*") => RuleList { rules: btreeset![ MatchingRule::Type ], rule_logic: RuleLogic::And },
        s!("$.animals[*].children") => RuleList { rules: btreeset![ MatchingRule::MinType(1) ], rule_logic: RuleLogic::And },
        s!("$.animals[*].children[*].*") => RuleList { rules: btreeset![ MatchingRule::Type ], rule_logic: RuleLogic::And }
      }
    }));
  }

  #[test]
  fn loads_v3_matching_rules() {
    let matching_rules_json = Value::from_str(r#"{"matchingRules": {
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

    expect!(matching_rules.rules.iter()).to_not(be_empty());
    expect!(matching_rules.categories()).to(be_equal_to(hashset!{ s!("path"), s!("query"), s!("header"), s!("body") }));
    expect!(matching_rules.rules_for_category(&s!("path"))).to(be_some().value(MatchingRuleCategory {
      name: s!("path"),
      rules: hashmap! { s!("") => RuleList { rules: btreeset![ MatchingRule::Regex(s!("\\w+")) ], rule_logic: RuleLogic::And } }
    }));
    expect!(matching_rules.rules_for_category(&s!("query"))).to(be_some().value(MatchingRuleCategory {
      name: s!("query"),
      rules: hashmap!{ s!("Q1") => RuleList { rules: btreeset![ MatchingRule::Regex(s!("\\d+")) ], rule_logic: RuleLogic::And } }
    }));
    expect!(matching_rules.rules_for_category(&s!("header"))).to(be_some().value(MatchingRuleCategory {
      name: s!("header"),
      rules: hashmap!{ s!("HEADERY") => RuleList { rules: btreeset![
        MatchingRule::Include(s!("ValueA")),
        MatchingRule::Include(s!("ValueB")) ], rule_logic: RuleLogic::Or } }
    }));
    expect!(matching_rules.rules_for_category(&s!("body"))).to(be_some().value(MatchingRuleCategory {
      name: s!("body"),
      rules: hashmap!{
        s!("$.animals") => RuleList { rules: btreeset![ MatchingRule::MinType(1) ], rule_logic: RuleLogic::And },
        s!("$.animals[*].*") => RuleList { rules: btreeset![ MatchingRule::Type ], rule_logic: RuleLogic::And },
        s!("$.animals[*].children") => RuleList { rules: btreeset![ MatchingRule::MinType(1) ], rule_logic: RuleLogic::And },
        s!("$.animals[*].children[*].*") => RuleList { rules: btreeset![ MatchingRule::Type ], rule_logic: RuleLogic::And }
      }
    }));
  }

  #[test]
  fn correctly_loads_v3_matching_rules_with_incorrect_path_format() {
    let matching_rules_json = Value::from_str(r#"{"matchingRules": {
      "path": {
        "": {
          "matchers": [
            { "match": "regex", "regex": "\\w+" }
          ]
        }
      }
    }}"#).unwrap();

    let matching_rules = matchers_from_json(&matching_rules_json, &None);

    expect!(matching_rules.rules.iter()).to_not(be_empty());
    expect!(matching_rules.categories()).to(be_equal_to(hashset!{ s!("path") }));
    expect!(matching_rules.rules_for_category(&s!("path"))).to(be_some().value(MatchingRuleCategory {
      name: s!("path"),
      rules: hashmap! { s!("") => RuleList { rules: btreeset![ MatchingRule::Regex(s!("\\w+")) ], rule_logic: RuleLogic::And } }
    }));
  }

  speculate! {
    describe "generating matcher JSON" {
      before {
        let matchers = matchingrules!{
          "body" => {
            "$.a.b" => [ MatchingRule::Type ]
          },
          "path" => { "" => [ MatchingRule::Regex(s!("/path/\\d+")) ] },
          "query" => {
            "a" => [ MatchingRule::Regex(s!("\\w+")) ]
          },
          "header" => {
            "item1" => [ MatchingRule::Regex(s!("5")) ]
          }
        };
      }

      it "generates V2 matcher format" {
        expect!(matchers.to_v2_json().to_string()).to(be_equal_to(
          "{\"$.body.a.b\":{\"match\":\"type\"},\
          \"$.header.item1\":{\"match\":\"regex\",\"regex\":\"5\"},\
          \"$.path.\":{\"match\":\"regex\",\"regex\":\"/path/\\\\d+\"},\
          \"$.query.a\":{\"match\":\"regex\",\"regex\":\"\\\\w+\"}}"
        ));
      }

      it "generates V3 matcher format" {
        expect!(matchers.to_v3_json().to_string()).to(be_equal_to(
          "{\"body\":{\"$.a.b\":{\"combine\":\"AND\",\"matchers\":[{\"match\":\"type\"}]}},\
          \"header\":{\"item1\":{\"combine\":\"AND\",\"matchers\":[{\"match\":\"regex\",\"regex\":\"5\"}]}},\
          \"path\":{\"combine\":\"AND\",\"matchers\":[{\"match\":\"regex\",\"regex\":\"/path/\\\\d+\"}]},\
          \"query\":{\"a\":{\"combine\":\"AND\",\"matchers\":[{\"match\":\"regex\",\"regex\":\"\\\\w+\"}]}}}"
        ));
      }
    }
  }

  #[test]
  fn matching_rule_from_json_test() {
    expect!(MatchingRule::from_json(&Value::from_str("\"test string\"").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Value::from_str("null").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Value::from_str("{}").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Value::from_str("[]").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Value::from_str("true").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Value::from_str("false").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Value::from_str("100").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Value::from_str("100.10").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Value::from_str("{\"stuff\": 100}").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"stuff\"}").unwrap())).to(be_none());

    expect!(MatchingRule::from_json(&Value::from_str("{\"regex\": \"[0-9]\"}").unwrap())).to(
      be_some().value(MatchingRule::Regex(s!("[0-9]"))));
    expect!(MatchingRule::from_json(&Value::from_str("{\"min\": 100}").unwrap())).to(
      be_some().value(MatchingRule::MinType(100)));
    expect!(MatchingRule::from_json(&Value::from_str("{\"max\": 100}").unwrap())).to(
      be_some().value(MatchingRule::MaxType(100)));
    expect!(MatchingRule::from_json(&Value::from_str("{\"timestamp\": \"yyyy\"}").unwrap())).to(
      be_some().value(MatchingRule::Timestamp(s!("yyyy"))));
    expect!(MatchingRule::from_json(&Value::from_str("{\"date\": \"yyyy\"}").unwrap())).to(
      be_some().value(MatchingRule::Date(s!("yyyy"))));
    expect!(MatchingRule::from_json(&Value::from_str("{\"time\": \"hh:mm\"}").unwrap())).to(
      be_some().value(MatchingRule::Time(s!("hh:mm"))));

    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"regex\", \"regex\": \"[0-9]\"}").unwrap())).to(
      be_some().value(MatchingRule::Regex(s!("[0-9]"))));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"regex\"}").unwrap())).to(be_none());

    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"equality\"}").unwrap())).to(
      be_some().value(MatchingRule::Equality));

    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"include\", \"value\": \"A\"}").unwrap())).to(
      be_some().value(MatchingRule::Include(s!("A"))));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"include\"}").unwrap())).to(be_none());

    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"type\", \"min\": 1}").unwrap())).to(
      be_some().value(MatchingRule::MinType(1)));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"type\", \"max\": \"1\"}").unwrap())).to(
      be_some().value(MatchingRule::MaxType(1)));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"type\", \"min\": 1, \"max\": \"1\"}").unwrap())).to(
      be_some().value(MatchingRule::MinMaxType(1, 1)));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"type\"}").unwrap())).to(
      be_some().value(MatchingRule::Type));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"type\", \"value\": 100}").unwrap())).to(
      be_some().value(MatchingRule::Type));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"min\", \"min\": 1}").unwrap())).to(
      be_some().value(MatchingRule::MinType(1)));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"max\", \"max\": \"1\"}").unwrap())).to(
      be_some().value(MatchingRule::MaxType(1)));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"min\"}").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"max\"}").unwrap())).to(be_none());

    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"number\"}").unwrap())).to(
      be_some().value(MatchingRule::Number));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"integer\"}").unwrap())).to(
      be_some().value(MatchingRule::Integer));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"decimal\"}").unwrap())).to(
      be_some().value(MatchingRule::Decimal));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"real\"}").unwrap())).to(
      be_some().value(MatchingRule::Decimal));

    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"timestamp\", \"timestamp\": \"A\"}").unwrap())).to(
      be_some().value(MatchingRule::Timestamp(s!("A"))));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"timestamp\"}").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"time\", \"time\": \"A\"}").unwrap())).to(
      be_some().value(MatchingRule::Time(s!("A"))));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"time\"}").unwrap())).to(be_none());
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"date\", \"date\": \"A\"}").unwrap())).to(
      be_some().value(MatchingRule::Date(s!("A"))));
    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"date\"}").unwrap())).to(be_none());

    expect!(MatchingRule::from_json(&Value::from_str("{\"match\": \"null\"}").unwrap())).to(
      be_some().value(MatchingRule::Null));
  }

  #[test]
  fn matcher_is_defined_returns_false_when_there_are_no_matchers() {
    let matchers = matchingrules!{};
    expect!(matchers.matcher_is_defined("body", &vec!["$", "a", "b"])).to(be_false());
  }

  #[test]
  fn matcher_is_defined_returns_false_when_the_path_does_not_have_a_matcher_entry() {
    let matchers = matchingrules!{
            "body" => {
            }
        };
    expect!(matchers.matcher_is_defined("body", &vec!["$", "a", "b"])).to(be_false());
  }

  #[test]
  fn matcher_is_defined_returns_true_when_the_path_does_have_a_matcher_entry() {
    let matchers = matchingrules! {
      "body" => {
        "$.a.b" => [ MatchingRule::Type ]
      }
    };
    expect!(matchers.matcher_is_defined("body", &vec!["$", "a", "b"])).to(be_true());
  }

  #[test]
  fn matcher_is_defined_returns_true_when_the_parent_of_the_path_does_have_a_matcher_entry() {
    let matchers = matchingrules!{
            "body" => {
                "$.a.b" => [ MatchingRule::Type ]
            }
        };
    expect!(matchers.matcher_is_defined("body", &vec!["$", "a", "b", "c"])).to(be_true());
  }

  #[test]
  fn wildcard_matcher_is_defined_returns_false_when_there_are_no_matchers() {
    let matchers = matchingrules!{};
    expect!(matchers.wildcard_matcher_is_defined("body", &vec!["$", "a", "b"])).to(be_false());
  }

  #[test]
  fn wildcard_matcher_is_defined_returns_false_when_the_path_does_not_have_a_matcher_entry() {
    let matchers = matchingrules!{
            "body" => {

            }
        };
    expect!(matchers.wildcard_matcher_is_defined("body", &vec!["$", "a", "b"])).to(be_false());
  }

  #[test]
  fn wildcard_matcher_is_defined_returns_false_when_the_path_does_have_a_matcher_entry_and_it_is_not_a_wildcard() {
    let matchers = matchingrules!{
            "body" => {
                "$.a.b" => [ MatchingRule::Type ],
                "$.*" => [ MatchingRule::Type ]
            }
        };
    expect!(matchers.wildcard_matcher_is_defined("body", &vec!["$", "a", "b"])).to(be_false());
  }

  #[test]
  fn wildcard_matcher_is_defined_returns_true_when_the_path_does_have_a_matcher_entry_and_it_is_a_widcard() {
    let matchers = matchingrules!{
            "body" => {
                "$.a.*" => [ MatchingRule::Type ]
            }
        };
    expect!(matchers.wildcard_matcher_is_defined("body", &vec!["$", "a", "b"])).to(be_true());
  }

  #[test]
  fn wildcard_matcher_is_defined_returns_false_when_the_parent_of_the_path_does_have_a_matcher_entry() {
    let matchers = matchingrules!{
            "body" => {
                "$.a.*" => [ MatchingRule::Type ]
            }
        };
    expect!(matchers.wildcard_matcher_is_defined("body", &vec!["$", "a", "b", "c"])).to(be_false());
  }

  #[test]
  fn matches_token_test_with_root() {
    expect!(matches_token("$", &PathToken::Root)).to(be_equal_to(2));
    expect!(matches_token("path", &PathToken::Root)).to(be_equal_to(0));
    expect!(matches_token("*", &PathToken::Root)).to(be_equal_to(0));
  }

  #[test]
  fn matches_token_test_with_field() {
    expect!(matches_token("$", &PathToken::Field(s!("path")))).to(be_equal_to(0));
    expect!(matches_token("path", &PathToken::Field(s!("path")))).to(be_equal_to(2));
  }

  #[test]
  fn matches_token_test_with_index() {
    expect!(matches_token("$", &PathToken::Index(2))).to(be_equal_to(0));
    expect!(matches_token("path", &PathToken::Index(2))).to(be_equal_to(0));
    expect!(matches_token("*", &PathToken::Index(2))).to(be_equal_to(0));
    expect!(matches_token("1", &PathToken::Index(2))).to(be_equal_to(0));
    expect!(matches_token("2", &PathToken::Index(2))).to(be_equal_to(2));
  }

  #[test]
  fn matches_token_test_with_index_wildcard() {
    expect!(matches_token("$", &PathToken::StarIndex)).to(be_equal_to(0));
    expect!(matches_token("path", &PathToken::StarIndex)).to(be_equal_to(0));
    expect!(matches_token("*", &PathToken::StarIndex)).to(be_equal_to(0));
    expect!(matches_token("1", &PathToken::StarIndex)).to(be_equal_to(1));
  }

  #[test]
  fn matches_token_test_with_wildcard() {
    expect!(matches_token("$", &PathToken::Star)).to(be_equal_to(1));
    expect!(matches_token("path", &PathToken::Star)).to(be_equal_to(1));
    expect!(matches_token("*", &PathToken::Star)).to(be_equal_to(1));
    expect!(matches_token("1", &PathToken::Star)).to(be_equal_to(1));
  }

  #[test]
  fn matches_path_matches_root_path_element() {
    expect!(calc_path_weight("$", &vec!["$"]).0 > 0).to(be_true());
    expect!(calc_path_weight("$", &vec![]).0 > 0).to(be_false());
  }

  #[test]
  fn matches_path_matches_field_name() {
    expect!(calc_path_weight("$.name", &vec!["$", "name"]).0 > 0).to(be_true());
    expect!(calc_path_weight("$['name']", &vec!["$", "name"]).0 > 0).to(be_true());
    expect!(calc_path_weight("$.name.other", &vec!["$", "name", "other"]).0 > 0).to(be_true());
    expect!(calc_path_weight("$['name'].other", &vec!["$", "name", "other"]).0 > 0).to(be_true());
    expect!(calc_path_weight("$.name", &vec!["$", "other"]).0 > 0).to(be_false());
    expect!(calc_path_weight("$.name", &vec!["$", "name", "other"]).0 > 0).to(be_true());
    expect!(calc_path_weight("$.other", &vec!["$", "name", "other"]).0 > 0).to(be_false());
    expect!(calc_path_weight("$.name.other", &vec!["$", "name"]).0 > 0).to(be_false());
  }

  #[test]
  fn matches_path_matches_array_indices() {
    expect!(calc_path_weight("$[0]", &vec!["$", "0"]).0 > 0).to(be_true());
    expect!(calc_path_weight("$.name[1]", &vec!["$", "name", "1"]).0 > 0).to(be_true());
    expect!(calc_path_weight("$.name", &vec!["$", "0"]).0 > 0).to(be_false());
    expect!(calc_path_weight("$.name[1]", &vec!["$", "name", "0"]).0 > 0).to(be_false());
    expect!(calc_path_weight("$[1].name", &vec!["$", "name", "1"]).0 > 0).to(be_false());
  }

  #[test]
  fn matches_path_matches_with_wildcard() {
    expect!(calc_path_weight("$[*]", &vec!["$", "0"]).0 > 0).to(be_true());
    expect!(calc_path_weight("$.*", &vec!["$", "name"]).0 > 0).to(be_true());
    expect!(calc_path_weight("$.*.name", &vec!["$", "some", "name"]).0 > 0).to(be_true());
    expect!(calc_path_weight("$.name[*]", &vec!["$", "name", "0"]).0 > 0).to(be_true());
    expect!(calc_path_weight("$.name[*].name", &vec!["$", "name", "1", "name"]).0 > 0).to(be_true());
    expect!(calc_path_weight("$[*]", &vec!["$", "name"]).0 > 0).to(be_false());
  }

  #[test]
  fn min_and_max_values_get_serialised_to_json_as_numbers() {
    expect!(MatchingRule::MinType(1).to_json().to_string()).to(be_equal_to("{\"match\":\"type\",\"min\":1}"));
    expect!(MatchingRule::MaxType(1).to_json().to_string()).to(be_equal_to("{\"match\":\"type\",\"max\":1}"));
    expect!(MatchingRule::MinMaxType(1, 10).to_json().to_string()).to(be_equal_to("{\"match\":\"type\",\"max\":10,\"min\":1}"));
  }
}
