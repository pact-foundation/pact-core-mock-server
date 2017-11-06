//! `generators` module includes all the classes to deal with V3 format generators

use std::collections::HashMap;
use serde_json::{self, Value};
use super::PactSpecification;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use rand::{self, Rng};
use uuid::Uuid;

/// Trait to represent a generator
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Eq, Hash)]
pub enum Generator {
  /// Generates a random integer between the min and max values
  RandomInt(i32, i32),
  /// Generates a random UUID value
  Uuid
}

//  /// Convert this generator to a JSON struct
//  fn toJson(pactSpecVersion: PactSpecVersion): Map<String, Any>

pub trait GenerateValue<T> {
  fn generate_value(&self, value: &T) -> Option<T>;
}

impl GenerateValue<u16> for Generator {
  fn generate_value(&self, value: &u16) -> Option<u16> {
    match self {
      &Generator::RandomInt(min, max) => Some(rand::thread_rng().gen_range(min as u16, max as u16 + 1)),
      _ => None
    }
  }
}

impl GenerateValue<String> for Generator {
  fn generate_value(&self, value: &String) -> Option<String> {
    match self {
      &Generator::RandomInt(min, max) => Some(format!("{}", rand::thread_rng().gen_range(min, max + 1))),
      &Generator::Uuid => Some(Uuid::new_v4().simple().to_string()),
      _ => None
    }
  }
}

/// Category that the generator is applied to
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Eq, Hash)]
pub enum GeneratorCategory {
  /// Request Method
  METHOD,
  /// Request Path
  PATH,
  /// Request/Response Header
  HEADER,
  /// Request Query Parameter
  QUERY,
  /// Body
  BODY,
  /// Response Status
  STATUS
}

impl FromStr for GeneratorCategory {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "METHOD" => Ok(GeneratorCategory::METHOD),
      "PATH" => Ok(GeneratorCategory::PATH),
      "HEADER" => Ok(GeneratorCategory::HEADER),
      "QUERY" => Ok(GeneratorCategory::QUERY),
      "BODY" => Ok(GeneratorCategory::BODY),
      "STATUS" => Ok(GeneratorCategory::STATUS),
      _ => Err(format!("'{}' is not a valid GeneratorCategory", s))
    }
  }
}

/// Data structure for representing a collection of generators
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Eq)]
pub struct Generators {
  /// Map of generator categories to maps of generators
  pub categories: HashMap<GeneratorCategory, HashMap<String, Generator>>
}

impl Generators {
  /// Create a empty set of generators
  pub fn default() -> Generators {
    Generators {
      categories: hashmap!{}
    }
  }

  /// If the generators are empty (that is there are no rules assigned to any categories)
  pub fn is_empty(&self) -> bool {
    self.categories.values().all(|category| category.is_empty())
  }

  /// If the generators are not empty (that is there is at least one rule assigned to a category)
  pub fn is_not_empty(&self) -> bool {
    self.categories.values().any(|category| !category.is_empty())
  }

  pub fn load_from_map(&mut self, map: &serde_json::Map<String, Value>) {
    for (k, v) in map {

    }
  }

  fn to_json(&self) -> Value {
    Value::Object(self.categories.iter().fold(serde_json::Map::new(), |map, (name, category)| {
//      map.insert(name.clone(), category.to_v3_json());
      map
    }))
  }

  pub fn add_generator(&mut self, category: GeneratorCategory, generator: Generator) {
    self.add_generator_with_subcategory(category, "", generator);
  }

  pub fn add_generator_with_subcategory<S: Into<String>>(&mut self, category: GeneratorCategory,
                                                         subcategory: S, generator: Generator) {
    let category_map = self.categories.entry(category).or_insert(HashMap::new());
    category_map.insert(subcategory.into(), generator.clone());
  }

  pub fn apply_generator<F>(&self, category: GeneratorCategory, mut closure: F)
    where F: FnMut(&String, &Generator) {
    if self.categories.contains_key(&category) && !self.categories[&category].is_empty() {
      for (key, value) in self.categories[&category].clone() {
        closure(&key, &value)
      }
    }
  }
}

impl Hash for Generators {
  fn hash<H: Hasher>(&self, state: &mut H) {
    for (k, v) in self.categories.iter() {
      k.hash(state);
      for (k2, v2) in v.iter() {
        k2.hash(state);
        v2.hash(state);
      }
    }
  }
}

/// Parses the generators from the Value structure
pub fn generators_from_json(value: &Value) -> Generators {
  let mut generators = Generators::default();
  match value {
    &Value::Object(ref m) => {
      generators.load_from_map(m)
    },
    _ => ()
  }
  generators
}

/// Generates a Value structure for the provided generators
pub fn generators_to_json(generators: &Generators, spec_version: &PactSpecification) -> Value {
  match spec_version {
    &PactSpecification::V3 => generators.to_json(),
    _ => Value::Null
  }
}

#[macro_export]
macro_rules! generators {
  (
    $( $category:expr => {
      $( $subname:expr => $generator:expr ), *
    }), *
  ) => {{
    let mut _generators = $crate::models::generators::Generators::default();

  $(
    let _cat = $crate::models::generators::GeneratorCategory::from_str($category).unwrap();
    $(
      _generators.add_generator_with_subcategory(_cat, $subname, $generator);
    )*
  )*

    _generators
  }};

  (
    $( $category:expr => $generator:expr ), *
  ) => {{
    let mut _generators = $crate::models::generators::Generators::default();
    $(
      let _cat = $crate::models::generators::GeneratorCategory::from_str($category).unwrap();
      _generators.add_generator(_cat, $generator);
    )*
    _generators
  }};
}

#[cfg(test)]
mod tests {
  use super::*;
  use expectest::prelude::*;
  use super::Generator;
  use std::str::FromStr;

  #[test]
  fn rules_are_empty_when_there_are_no_categories() {
    expect!(Generators::default().is_empty()).to(be_true());
  }

  #[test]
  fn rules_are_empty_when_there_are_only_empty_categories() {
    expect!(Generators {
            categories: hashmap!{
                GeneratorCategory::BODY => hashmap!{},
                GeneratorCategory::HEADER => hashmap!{},
                GeneratorCategory::QUERY => hashmap!{}
            }
        }.is_empty()).to(be_true());
  }

  #[test]
  fn rules_are_not_empty_when_there_is_a_nonempty_category() {
    expect!(Generators {
            categories: hashmap!{
                GeneratorCategory::BODY => hashmap!{},
                GeneratorCategory::HEADER => hashmap!{},
                GeneratorCategory::QUERY => hashmap! {
                    "a".to_string() => Generator::RandomInt(1, 10)
                }
            }
        }.is_empty()).to(be_false());
  }

  #[test]
  fn matchers_from_json_test() {
    expect!(generators_from_json(&Value::Null).categories.iter()).to(be_empty());
  }

  #[test]
  fn generators_macro_test() {
    expect!(generators!{}).to(be_equal_to(Generators::default()));

    let mut expected = Generators::default();
    expected.add_generator(GeneratorCategory::STATUS,
                                                       Generator::RandomInt(400, 499));
    expect!(generators!{
      "STATUS" => Generator::RandomInt(400, 499)
    }).to(be_equal_to(expected));

    expected = Generators::default();
    expected.add_generator_with_subcategory(GeneratorCategory::BODY, "$.a.b",
                           Generator::RandomInt(1, 10));
    expect!(generators!{
      "BODY" => {
        "$.a.b" => Generator::RandomInt(1, 10)
      }
    }).to(be_equal_to(expected));
  }
}
