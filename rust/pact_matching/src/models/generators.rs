//! `generators` module includes all the classes to deal with V3 format generators

use std::collections::HashMap;
use serde_json::{self, Value};
use super::PactSpecification;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use rand::{self, Rng};
use uuid::Uuid;
use models::{OptionalBody, DetectedContentType};
use models::json_utils::{JsonToNum, json_to_string};
use models::xml_utils::parse_bytes;
use sxd_document::dom::Document;
use path_exp::*;
use std::slice::Iter;
use std::rc::Rc;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::ops::Deref;

/// Trait to represent a generator
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Eq, Hash)]
pub enum Generator {
  /// Generates a random integer between the min and max values
  RandomInt(i32, i32),
  /// Generates a random UUID value
  Uuid,
  /// Generates a random sequence of digits
  RandomDecimal(u16),
  /// Generates a random sequence of hexadecimal digits
  RandomHexadecimal(u16),
  /// Generates a random string of the provided size
  RandomString(u16),
  /// Generates a random string that matches the provided regex
  Regex(String),
  /// Generates a random date that matches either the provided format or the ISO format
  Date(Option<String>),
  /// Generates a random time that matches either the provided format or the ISO format
  Time(Option<String>),
  /// Generates a random timestamp that matches either the provided format or the ISO format
  Timestamp(Option<String>),
  /// Generates a random boolean value
  RandomBoolean
}

impl Generator {
  /// Convert this generator to a JSON struct
  pub fn to_json(&self) -> Value {
    match self {
      &Generator::RandomInt(min, max) => json!({ "type": "RandomInt", "min": min, "max": max }),
      &Generator::Uuid => json!({ "type": "Uuid" }),
      &Generator::RandomDecimal(digits) => json!({ "type": "RandomDecimal", "digits": digits }),
      &Generator::RandomHexadecimal(digits) => json!({ "type": "RandomHexadecimal", "digits": digits }),
      &Generator::RandomString(size) => json!({ "type": "RandomString", "size": size }),
      &Generator::Regex(ref regex) => json!({ "type": "Regex", "regex": regex }),
      &Generator::Date(ref format) => match format {
        &Some(ref format) => json!({ "type": "Date", "format": format }),
        &None => json!({ "type": "Date" })
      },
      &Generator::Time(ref format) => match format {
        &Some(ref format) => json!({ "type": "Time", "format": format }),
        &None => json!({ "type": "Time" })
      },
      &Generator::Timestamp(ref format) => match format {
        &Some(ref format) => json!({ "type": "Timestamp", "format": format }),
        &None => json!({ "type": "Timestamp" })
      },
      &Generator::RandomBoolean => json!({ "type": "RandomBoolean" })
    }
  }

  pub fn from_map(gen_type: &String, map: &serde_json::Map<String, Value>) -> Option<Generator> {
    match gen_type.as_str() {
      "RandomInt" => {
        let min = <i32>::json_to_number(map, "min", 0);
        let max = <i32>::json_to_number(map, "max", 10);
        Some(Generator::RandomInt(min, max))
      },
      "Uuid" => Some(Generator::Uuid),
      "RandomDecimal" => Some(Generator::RandomDecimal(<u16>::json_to_number(map, "digits", 10))),
      "RandomHexadecimal" => Some(Generator::RandomHexadecimal(<u16>::json_to_number(map, "digits", 10))),
      "RandomString" => Some(Generator::RandomString(<u16>::json_to_number(map, "size", 10))),
      "Regex" => map.get("regex").map(|val| Generator::Regex(json_to_string(val))),
      "Date" => Some(Generator::Date(map.get("format").map(|f| json_to_string(f)))),
      "Time" => Some(Generator::Time(map.get("format").map(|f| json_to_string(f)))),
      "Timestamp" => Some(Generator::Timestamp(map.get("format").map(|f| json_to_string(f)))),
      "RandomBoolean" => Some(Generator::RandomBoolean),
      _ => {
        warn!("'{}' is not a valid generator type", gen_type);
        None
      }
    }
  }
}

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

impl GenerateValue<Value> for Generator {
  fn generate_value(&self, value: &Value) -> Option<Value> {
    match self {
      &Generator::RandomInt(min, max) => {
        let rand_int = rand::thread_rng().gen_range(min, max + 1);
        match value {
          &Value::String(_) => Some(json!(format!("{}", rand_int))),
          &Value::Number(_) => Some(json!(rand_int)),
          _ => None
        }
      },
      &Generator::Uuid => match value {
        &Value::String(_) => Some(json!(Uuid::new_v4().simple().to_string())),
        _ => None
      },
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
    match s.to_lowercase().as_str() {
      "method" => Ok(GeneratorCategory::METHOD),
      "path" => Ok(GeneratorCategory::PATH),
      "header" => Ok(GeneratorCategory::HEADER),
      "query" => Ok(GeneratorCategory::QUERY),
      "body" => Ok(GeneratorCategory::BODY),
      "status" => Ok(GeneratorCategory::STATUS),
      _ => Err(format!("'{}' is not a valid GeneratorCategory", s))
    }
  }
}

impl <'a> Into<&'a str> for GeneratorCategory {
  fn into(self) -> &'a str {
    match self {
      GeneratorCategory::METHOD => "method",
      GeneratorCategory::PATH => "path",
      GeneratorCategory::HEADER => "header",
      GeneratorCategory::QUERY => "query",
      GeneratorCategory::BODY => "body",
      GeneratorCategory::STATUS => "status"
    }
  }
}

impl Into<String> for GeneratorCategory {
  fn into(self) -> String {
    let s: &str = self.into();
    s.to_string()
  }
}

#[derive(PartialEq, Debug, Eq, Clone)]
pub struct QueryResult<T> {
  pub value: T,
  pub key: Option<String>,
  pub parent: Option<T>
}

impl <T> QueryResult<T> where T: Clone {

  pub fn default(value: T) -> QueryResult<T> {
    QueryResult { value, key: None, parent: None }
  }

  pub fn update_to(&mut self, new_value: &mut T, new_key: Option<String>, new_parent: Option<T>) {
    self.value = new_value.clone();
    self.key = new_key;
    self.parent = new_parent;
  }

  pub fn update_value(&mut self, new_value: T) {
    self.value = new_value.clone();
  }

}

pub trait ContentTypeHandler<T> {
  fn process_body(&self) -> OptionalBody;
  fn apply_key(&self, mut body: QueryResult<T>, key: &String, generator: &Generator);
  fn new_cursor(&self) -> QueryResult<T>;
}

pub struct JsonHandler {
  pub value: Value
}

impl JsonHandler {
  fn query_object_graph(&self, path_exp: Iter<PathToken>, mut body: QueryResult<Value>, callback: &mut FnMut(QueryResult<Value>)) {
    let mut body_cursor = body.clone();
    for token in path_exp {
      match token {
        &PathToken::Field(ref name) => {
          let parent = body.value.clone();
          match body.value.as_object_mut() {
            Some(map) => match map.get_mut(name) {
              Some(new_value) => {
                body_cursor.update_to(new_value, Some(name.clone()), Some(parent));
              },
              None => return
            },
            None => return
          }
        }
        /*
        is PathToken.Field -> if (bodyCursor.value is Map<*, *> &&
          (bodyCursor.value as Map<*, *>).containsKey(token.name)) {
          val map = bodyCursor.value as Map<*, *>
          bodyCursor = QueryResult(map[token.name]!!, token.name, bodyCursor.value)
        } else {
          return
        }
        is PathToken.Index -> if (bodyCursor.value is List<*> && (bodyCursor.value as List<*>).size > token.index) {
          val list = bodyCursor.value as List<*>
          bodyCursor = QueryResult(list[token.index]!!, token.index, bodyCursor.value)
        } else {
          return
        }
        is PathToken.Star -> if (bodyCursor.value is MutableMap<*, *>) {
          val map = bodyCursor.value as MutableMap<*, *>
          val pathIterator = IteratorUtils.toList(pathExp)
          map.forEach { (key, value) ->
            queryObjectGraph(pathIterator.iterator(), QueryResult(value!!, key, map), fn)
          }
          return
        } else {
          return
        }
        is PathToken.StarIndex -> if (bodyCursor.value is List<*>) {
          val list = bodyCursor.value as List<*>
          val pathIterator = IteratorUtils.toList(pathExp)
          list.forEachIndexed { index, item ->
            queryObjectGraph(pathIterator.iterator(), QueryResult(item!!, index, list), fn)
          }
          return
        } else {
          return
        }
  */
        _ => ()
      }
    }

    callback(body_cursor);
  }
}

impl ContentTypeHandler<Value> for JsonHandler {
  fn process_body(&self) -> OptionalBody {
    unimplemented!()
//  self.apply_generator(&GeneratorCategory::BODY, |key, generator| {
//    handler.apply_key(body_result, key, generator);
//  });
  }

  fn apply_key(&self, mut body: QueryResult<Value>, key: &String, generator: &Generator) {
    match parse_path_exp(key.clone()) {
      Ok(mut path_exp) => self.query_object_graph(path_exp.iter(), body, &mut |mut body_val: QueryResult<Value>| {
        p!(body_val);
        let bv = &body_val.value.clone();
        let bk = body_val.key.clone();
        match body_val.parent.clone() {
          Some(ref mut parent_value) => match parent_value {
            &mut Value::Object(ref mut map) => match generator.generate_value(bv) {
              Some(val) => {
                map.insert(bk.unwrap(), val);
              },
              None => ()
            },
            &mut Value::Array(ref mut array) => match generator.generate_value(bv) {
              Some(val) => {
                let index = bk.clone().unwrap().parse::<usize>();
                match index {
                  Ok(index) => array.insert(index, val),
                  Err(err) => warn!("Ignoring generator as {:?} is not a valid array index", bk)
                }
              },
              None => ()
            },
            _ => match generator.generate_value(bv) {
              Some(mut val) => {
                body_val.update_value(val);
              },
              None => ()
            }
          },
          _ => match generator.generate_value(bv) {
            Some(mut val) => {
              body_val.update_value(val);
            },
            None => ()
          }
        }
      }),
      Err(err) => warn!("Generator path '{}' is invalid, ignoring: {}", key, err)
    }
  }

  fn new_cursor(&self) -> QueryResult<Value> {
    unimplemented!()
  }
}

pub struct XmlHandler<'a> {
  pub value: Document<'a>
}

impl <'a> ContentTypeHandler<Document<'a>> for XmlHandler<'a> {
  fn process_body(&self) -> OptionalBody {
    unimplemented!()
  }

  fn apply_key(&self, mut body: QueryResult<Document<'a>>, key: &String, generator: &Generator) {
    unimplemented!()
  }

  fn new_cursor(&self) -> QueryResult<Document<'a>> {
    unimplemented!()
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
      match v {
        &Value::Object(ref map) =>  match GeneratorCategory::from_str(k) {
          Ok(ref category) => match category {
            &GeneratorCategory::PATH | &GeneratorCategory::METHOD | &GeneratorCategory::STATUS => {
              self.parse_generator_from_map(category, map, None);
            },
            _ => for (sub_k, sub_v) in map {
              match sub_v {
                &Value::Object(ref map) => self.parse_generator_from_map(category, map, Some(sub_k.clone())),
                _ => warn!("Ignoring invalid generator JSON '{}' -> {:?}", sub_k, sub_v)
              }
            }
          },
          Err(err) => warn!("Ignoring generator with invalid category '{}' - {}", k, err)
        },
        _ => warn!("Ignoring invalid generator JSON '{}' -> {:?}", k, v)
      }
    }
  }

  fn parse_generator_from_map(&mut self, category: &GeneratorCategory,
                              map: &serde_json::Map<String, Value>, subcat: Option<String>) {
    match map.get("type") {
      Some(gen_type) => match gen_type {
        &Value::String(ref gen_type) => match Generator::from_map(gen_type, map) {
          Some(generator) => match subcat {
            Some(s) => self.add_generator_with_subcategory(category, s, generator),
            None => self.add_generator(category, generator)
          },
          None => warn!("Ignoring invalid generator JSON '{:?}' with invalid type attribute -> {:?}", category, map)
        },
        _ => warn!("Ignoring invalid generator JSON '{:?}' with invalid type attribute -> {:?}", category, map)
      },
      None => warn!("Ignoring invalid generator JSON '{:?}' with no type attribute -> {:?}", category, map)
    }
  }

  fn to_json(&self) -> Value {
    Value::Object(self.categories.iter().fold(serde_json::Map::new(), |mut map, (name, category)| {
      let cat: String = name.clone().into();
      match name {
        &GeneratorCategory::PATH | &GeneratorCategory::METHOD | &GeneratorCategory::STATUS => {
          match category.get("") {
            Some(generator) => {
              map.insert(cat.clone(), category.get("").unwrap().to_json());
            },
            None => ()
          }
        },
        _ => {
          for (key, val) in category {
            map.insert(cat.clone(), json!({ key.as_str(): val.to_json() }));
          }
        }
      }
      map
    }))
  }

  pub fn add_generator(&mut self, category: &GeneratorCategory, generator: Generator) {
    self.add_generator_with_subcategory(category, "", generator);
  }

  pub fn add_generator_with_subcategory<S: Into<String>>(&mut self, category: &GeneratorCategory,
                                                         subcategory: S, generator: Generator) {
    let category_map = self.categories.entry(category.clone()).or_insert(HashMap::new());
    category_map.insert(subcategory.into(), generator.clone());
  }

  pub fn apply_generator<F>(&self, category: &GeneratorCategory, mut closure: F)
    where F: FnMut(&String, &Generator) {
    if self.categories.contains_key(category) && !self.categories[category].is_empty() {
      for (key, value) in self.categories[category].clone() {
        closure(&key, &value)
      }
    }
  }

  pub fn apply_body_generators(&self, body: &OptionalBody, content_type: DetectedContentType) -> OptionalBody {
    if body.is_present() {
      if self.is_not_empty() {
        match content_type {
          DetectedContentType::Json => {
            let result: Result<Value, serde_json::Error> = serde_json::from_slice(&body.value());
            match result {
              Ok(val) => {
                let handler = JsonHandler { value: val };
                handler.process_body()
              },
              Err(err) => {
                error!("Failed to parse the body, so not applying any generators: {}", err);
                body.clone()
              }
            }
          },
          DetectedContentType::Xml => match parse_bytes(&body.value()) {
            Ok(val) => {
              let handler = XmlHandler { value: val.as_document() };
              handler.process_body()
            },
            Err(err) => {
              error!("Failed to parse the body, so not applying any generators: {}", err);
              body.clone()
            }
          },
          _ => body.clone()
        }
      } else {
        body.clone()
      }
    } else {
      body.clone()
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
    &Value::Object(ref m) => match m.get("generators") {
      Some(gen_val) => match gen_val {
        &Value::Object(ref m) => generators.load_from_map(m),
        _ => ()
      },
      None => ()
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
      _generators.add_generator_with_subcategory(&_cat, $subname, $generator);
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
      _generators.add_generator(&_cat, $generator);
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
    expected.add_generator(&GeneratorCategory::STATUS, Generator::RandomInt(400, 499));
    expect!(generators!{
      "STATUS" => Generator::RandomInt(400, 499)
    }).to(be_equal_to(expected));

    expected = Generators::default();
    expected.add_generator_with_subcategory(&GeneratorCategory::BODY, "$.a.b",
                           Generator::RandomInt(1, 10));
    expect!(generators!{
      "BODY" => {
        "$.a.b" => Generator::RandomInt(1, 10)
      }
    }).to(be_equal_to(expected));
  }

  #[test]
  fn generator_from_json_test() {
    expect!(Generator::from_map(&s!(""), &serde_json::Map::new())).to(be_none());
    expect!(Generator::from_map(&s!("Invalid"), &serde_json::Map::new())).to(be_none());
    expect!(Generator::from_map(&s!("uuid"), &serde_json::Map::new())).to(be_none());
    expect!(Generator::from_map(&s!("Uuid"), &serde_json::Map::new())).to(be_some().value(Generator::Uuid));
    expect!(Generator::from_map(&s!("RandomBoolean"), &serde_json::Map::new())).to(be_some().value(Generator::RandomBoolean));
  }

  #[test]
  fn randomint_generator_from_json_test() {
    expect!(Generator::from_map(&s!("RandomInt"), &serde_json::Map::new())).to(be_some().value(Generator::RandomInt(0, 10)));
    expect!(Generator::from_map(&s!("RandomInt"), &json!({ "min": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomInt(5, 10)));
    expect!(Generator::from_map(&s!("RandomInt"), &json!({ "max": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomInt(0, 5)));
    expect!(Generator::from_map(&s!("RandomInt"), &json!({ "min": 5, "max": 6 }).as_object().unwrap())).to(be_some().value(Generator::RandomInt(5, 6)));
  }

  #[test]
  fn random_decimal_generator_from_json_test() {
    expect!(Generator::from_map(&s!("RandomDecimal"), &serde_json::Map::new())).to(be_some().value(Generator::RandomDecimal(10)));
    expect!(Generator::from_map(&s!("RandomDecimal"), &json!({ "min": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomDecimal(10)));
    expect!(Generator::from_map(&s!("RandomDecimal"), &json!({ "digits": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomDecimal(5)));
  }

  #[test]
  fn random_hexadecimal_generator_from_json_test() {
    expect!(Generator::from_map(&s!("RandomHexadecimal"), &serde_json::Map::new())).to(be_some().value(Generator::RandomHexadecimal(10)));
    expect!(Generator::from_map(&s!("RandomHexadecimal"), &json!({ "min": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomHexadecimal(10)));
    expect!(Generator::from_map(&s!("RandomHexadecimal"), &json!({ "digits": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomHexadecimal(5)));
  }

  #[test]
  fn random_string_generator_from_json_test() {
    expect!(Generator::from_map(&s!("RandomString"), &serde_json::Map::new())).to(be_some().value(Generator::RandomString(10)));
    expect!(Generator::from_map(&s!("RandomString"), &json!({ "min": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomString(10)));
    expect!(Generator::from_map(&s!("RandomString"), &json!({ "size": 5 }).as_object().unwrap())).to(be_some().value(Generator::RandomString(5)));
  }

  #[test]
  fn regex_generator_from_json_test() {
    expect!(Generator::from_map(&s!("Regex"), &serde_json::Map::new())).to(be_none());
    expect!(Generator::from_map(&s!("Regex"), &json!({ "min": 5 }).as_object().unwrap())).to(be_none());
    expect!(Generator::from_map(&s!("Regex"), &json!({ "regex": "\\d+" }).as_object().unwrap())).to(be_some().value(Generator::Regex(s!("\\d+"))));
    expect!(Generator::from_map(&s!("Regex"), &json!({ "regex": 5 }).as_object().unwrap())).to(be_some().value(Generator::Regex(s!("5"))));
  }

  #[test]
  fn date_generator_from_json_test() {
    expect!(Generator::from_map(&s!("Date"), &serde_json::Map::new())).to(be_some().value(Generator::Date(None)));
    expect!(Generator::from_map(&s!("Date"), &json!({ "min": 5 }).as_object().unwrap())).to(be_some().value(Generator::Date(None)));
    expect!(Generator::from_map(&s!("Date"), &json!({ "format": "yyyy-MM-dd" }).as_object().unwrap())).to(be_some().value(Generator::Date(Some(s!("yyyy-MM-dd")))));
    expect!(Generator::from_map(&s!("Date"), &json!({ "format": 5 }).as_object().unwrap())).to(be_some().value(Generator::Date(Some(s!("5")))));
  }

  #[test]
  fn time_generator_from_json_test() {
    expect!(Generator::from_map(&s!("Time"), &serde_json::Map::new())).to(be_some().value(Generator::Time(None)));
    expect!(Generator::from_map(&s!("Time"), &json!({ "min": 5 }).as_object().unwrap())).to(be_some().value(Generator::Time(None)));
    expect!(Generator::from_map(&s!("Time"), &json!({ "format": "yyyy-MM-dd" }).as_object().unwrap())).to(be_some().value(Generator::Time(Some(s!("yyyy-MM-dd")))));
    expect!(Generator::from_map(&s!("Time"), &json!({ "format": 5 }).as_object().unwrap())).to(be_some().value(Generator::Time(Some(s!("5")))));
  }

  #[test]
  fn timestamp_generator_from_json_test() {
    expect!(Generator::from_map(&s!("Timestamp"), &serde_json::Map::new())).to(be_some().value(Generator::Timestamp(None)));
    expect!(Generator::from_map(&s!("Timestamp"), &json!({ "min": 5 }).as_object().unwrap())).to(be_some().value(Generator::Timestamp(None)));
    expect!(Generator::from_map(&s!("Timestamp"), &json!({ "format": "yyyy-MM-dd" }).as_object().unwrap())).to(be_some().value(Generator::Timestamp(Some(s!("yyyy-MM-dd")))));
    expect!(Generator::from_map(&s!("Timestamp"), &json!({ "format": 5 }).as_object().unwrap())).to(be_some().value(Generator::Timestamp(Some(s!("5")))));
  }

  #[test]
  fn generator_to_json_test() {
    expect!(Generator::RandomInt(5, 15).to_json()).to(be_equal_to(json!({
      "type": "RandomInt",
      "min": 5,
      "max": 15
    })));
    expect!(Generator::Uuid.to_json()).to(be_equal_to(json!({
      "type": "Uuid"
    })));
    expect!(Generator::RandomDecimal(5).to_json()).to(be_equal_to(json!({
      "type": "RandomDecimal",
      "digits": 5
    })));
    expect!(Generator::RandomHexadecimal(5).to_json()).to(be_equal_to(json!({
      "type": "RandomHexadecimal",
      "digits": 5
    })));
    expect!(Generator::RandomString(5).to_json()).to(be_equal_to(json!({
      "type": "RandomString",
      "size": 5
    })));
    expect!(Generator::Regex(s!("\\d+")).to_json()).to(be_equal_to(json!({
      "type": "Regex",
      "regex": "\\d+"
    })));
    expect!(Generator::RandomBoolean.to_json()).to(be_equal_to(json!({
      "type": "RandomBoolean"
    })));

    expect!(Generator::Date(Some(s!("yyyyMMdd"))).to_json()).to(be_equal_to(json!({
      "type": "Date",
      "format": "yyyyMMdd"
    })));
    expect!(Generator::Date(None).to_json()).to(be_equal_to(json!({
      "type": "Date"
    })));
    expect!(Generator::Time(Some(s!("yyyyMMdd"))).to_json()).to(be_equal_to(json!({
      "type": "Time",
      "format": "yyyyMMdd"
    })));
    expect!(Generator::Time(None).to_json()).to(be_equal_to(json!({
      "type": "Time"
    })));
    expect!(Generator::Timestamp(Some(s!("yyyyMMdd"))).to_json()).to(be_equal_to(json!({
      "type": "Timestamp",
      "format": "yyyyMMdd"
    })));
    expect!(Generator::Timestamp(None).to_json()).to(be_equal_to(json!({
      "type": "Timestamp"
    })));
  }
}
