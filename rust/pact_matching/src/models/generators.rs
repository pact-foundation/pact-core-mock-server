//! `generators` module includes all the classes to deal with V3 format generators

use std::{
  collections::HashMap,
  hash::{Hash, Hasher},
  str::FromStr,
  ops::Index
};
use serde::{Serialize, Deserialize};
use serde_json::{self, Value, json};
use maplit::*;
use super::PactSpecification;
use rand::prelude::*;
use rand::distributions::Alphanumeric;
use rand::seq::SliceRandom;
use uuid::Uuid;
use crate::models::OptionalBody;
use crate::models::json_utils::{JsonToNum, json_to_string};
use crate::models::xml_utils::parse_bytes;
use sxd_document::dom::Document;
use crate::path_exp::*;
use itertools::Itertools;
use indextree::{Arena, NodeId};
use chrono::prelude::*;
use crate::time_utils::{parse_pattern, to_chrono_pattern};
use regex_syntax;
use crate::models::content_types::ContentType;
use crate::models::expression_parser::{contains_expressions, DataType, DataValue, parse_expression, MapValueResolver};
use std::convert::TryFrom;
use log::trace;

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
  DateTime(Option<String>),
  /// Generates a random boolean value
  RandomBoolean,
  /// Generates a value that is looked up from the provider state context
  ProviderStateGenerator(String, Option<DataType>)
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
      &Generator::DateTime(ref format) => match format {
        &Some(ref format) => json!({ "type": "DateTime", "format": format }),
        &None => json!({ "type": "DateTime" })
      },
      &Generator::RandomBoolean => json!({ "type": "RandomBoolean" }),
      &Generator::ProviderStateGenerator(ref expression, ref data_type) => {
        if let Some(data_type) = data_type {
          json!({"type": "ProviderState", "expression": expression, "dataType": data_type})
        } else {
          json!({"type": "ProviderState", "expression": expression})
        }
      }
    }
  }

  /// Converts a JSON map into a `Generator` struct, returning `None` if it can not be converted.
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
      "DateTime" => Some(Generator::DateTime(map.get("format").map(|f| json_to_string(f)))),
      "RandomBoolean" => Some(Generator::RandomBoolean),
      "ProviderState" => map.get("expression").map(|f|
        Generator::ProviderStateGenerator(json_to_string(f), map.get("dataType")
          .map(|dt| DataType::from(dt.clone())))),
      _ => {
        log::warn!("'{}' is not a valid generator type", gen_type);
        None
      }
    }
  }
}

/// Trait that represents generation of a value based on a source value.
pub trait GenerateValue<T> {
  /// Generates a new value based on the source value. An error will be returned if the value can not
  /// be generated.
  fn generate_value(&self, value: &T, context: &HashMap<String, Value>) -> Result<T, String>;
}

impl GenerateValue<u16> for Generator {
  fn generate_value(&self, value: &u16, context: &HashMap<String, Value>) -> Result<u16, String> {
    match self {
      &Generator::RandomInt(min, max) => Ok(rand::thread_rng().gen_range(min as u16, (max as u16).saturating_add(1))),
      &Generator::ProviderStateGenerator(ref exp, ref dt) =>
        match generate_value_from_context(exp, context, dt) {
          Ok(val) => u16::try_from(val),
          Err(err) => Err(err)
        },
      _ => Err(format!("Could not generate a u16 value from {} using {:?}", value, self))
    }
  }
}

const DIGIT_CHARSET: &'static str = "0123456789";
fn generate_decimal(digits: usize) -> String {
  let mut rnd = rand::thread_rng();
  let chars: Vec<char> = DIGIT_CHARSET.chars().collect();
  match digits {
    0 => "".to_string(),
    1 => chars.choose(&mut rnd).unwrap().to_string(),
    2 => format!("{}.{}", chars.choose(&mut rnd).unwrap(), chars.choose(&mut rnd).unwrap()),
    _ => {
      let mut sample = String::new();
      for _ in 0..(digits + 1) {
        sample.push(*chars.choose(&mut rnd).unwrap());
      }
      if sample.starts_with("00") {
        let chars = DIGIT_CHARSET[1..].chars();
        sample.insert(0, chars.choose(&mut rnd).unwrap());
      }
      let pos = rnd.gen_range(1, digits - 1);
      let selected_digits = if pos != 1 && sample.starts_with('0') {
        &sample[1..(digits + 1)]
      } else {
        &sample[..digits]
      };
      let generated = format!("{}.{}", &selected_digits[..pos], &selected_digits[pos..]);
      trace!("RandomDecimalGenerator: sample_digits=[{}], pos={}, selected_digits=[{}], generated=[{}]",
             sample, pos, selected_digits, generated);
      generated
    }
  }
}

fn generate_hexadecimal(digits: usize) -> String {
  const HEX_CHARSET: &'static str = "0123456789ABCDEF";
  let mut rnd = rand::thread_rng();
  HEX_CHARSET.chars().choose_multiple(&mut rnd, digits).iter().join("")
}

fn generate_ascii_string(size: usize) -> String {
  rand::thread_rng().sample_iter(&Alphanumeric).take(size).collect()
}

fn strip_anchors(regex: &str) -> &str {
  regex
    .strip_prefix('^').unwrap_or(regex)
    .strip_suffix('$').unwrap_or(regex)
}

impl GenerateValue<String> for Generator {
  fn generate_value(&self, _: &String, context: &HashMap<String, Value>) -> Result<String, String> {
    let mut rnd = rand::thread_rng();
    match self {
      &Generator::RandomInt(min, max) => Ok(format!("{}", rnd.gen_range(min, max.saturating_add(1)))),
      &Generator::Uuid => Ok(Uuid::new_v4().hyphenated().to_string()),
      &Generator::RandomDecimal(digits) => Ok(generate_decimal(digits as usize)),
      &Generator::RandomHexadecimal(digits) => Ok(generate_hexadecimal(digits as usize)),
      &Generator::RandomString(size) => Ok(generate_ascii_string(size as usize)),
      &Generator::Regex(ref regex) => {
        let mut parser = regex_syntax::ParserBuilder::new().unicode(false).build();
        match parser.parse(strip_anchors(regex)) {
          Ok(hir) => {
            match rand_regex::Regex::with_hir(hir, 20) {
              Ok(gen) => Ok(rnd.sample(gen)),
              Err(err) => {
                log::warn!("Failed to generate a value from regular expression - {}", err);
                Err(format!("Failed to generate a value from regular expression - {}", err))
              }
            }
          },
          Err(err) => {
            log::warn!("'{}' is not a valid regular expression - {}", regex, err);
            Err(format!("'{}' is not a valid regular expression - {}", regex, err))
          }
        }
      },
      &Generator::Date(ref format) => match format {
        Some(pattern) => match parse_pattern(pattern) {
          Ok(tokens) => Ok(Local::now().date().format(&to_chrono_pattern(&tokens)).to_string()),
          Err(err) => {
            log::warn!("Date format {} is not valid - {}", pattern, err);
            Err(format!("Date format {} is not valid - {}", pattern, err))
          }
        },
        None => Ok(Local::now().naive_local().date().to_string())
      },
      &Generator::Time(ref format) => match format {
        Some(pattern) => match parse_pattern(pattern) {
          Ok(tokens) => Ok(Local::now().format(&to_chrono_pattern(&tokens)).to_string()),
          Err(err) => {
            log::warn!("Time format {} is not valid - {}", pattern, err);
            Err(format!("Time format {} is not valid - {}", pattern, err))
          }
        },
        None => Ok(Local::now().time().format("%H:%M:%S").to_string())
      },
      &Generator::DateTime(ref format) => match format {
        Some(pattern) => match parse_pattern(pattern) {
          Ok(tokens) => Ok(Local::now().format(&to_chrono_pattern(&tokens)).to_string()),
          Err(err) => {
            log::warn!("DateTime format {} is not valid - {}", pattern, err);
            Err(format!("DateTime format {} is not valid - {}", pattern, err))
          }
        },
        None => Ok(Local::now().format("%Y-%m-%dT%H:%M:%S.%3f%z").to_string())
      },
      &Generator::RandomBoolean => Ok(format!("{}", rnd.gen::<bool>())),
      &Generator::ProviderStateGenerator(ref exp, ref dt) =>
        match generate_value_from_context(exp, context, dt) {
          Ok(val) => String::try_from(val),
          Err(err) => Err(err)
        }
    }
  }
}

impl GenerateValue<Vec<String>> for Generator {
  fn generate_value(&self, vals: &Vec<String>, context: &HashMap<String, Value>) -> Result<Vec<String>, String> {
    self.generate_value(vals.first().unwrap_or(&s!("")), context).map(|v| vec![v])
  }
}

impl GenerateValue<Value> for Generator {
  fn generate_value(&self, value: &Value, context: &HashMap<String, Value>) -> Result<Value, String> {
    match self {
      &Generator::RandomInt(min, max) => {
        let rand_int = rand::thread_rng().gen_range(min, max.saturating_add(1));
        match value {
          &Value::String(_) => Ok(json!(format!("{}", rand_int))),
          &Value::Number(_) => Ok(json!(rand_int)),
          _ => Err(format!("Could not generate a random int from {}", value))
        }
      },
      &Generator::Uuid => match value {
        &Value::String(_) => Ok(json!(Uuid::new_v4().simple().to_string())),
        _ => Err(format!("Could not generate a UUID from {}", value))
      },
      &Generator::RandomDecimal(digits) => match value {
        &Value::String(_) => Ok(json!(generate_decimal(digits as usize))),
        &Value::Number(_) => match generate_decimal(digits as usize).parse::<f64>() {
          Ok(val) => Ok(json!(val)),
          Err(err) => Err(format!("Could not generate a random decimal from {} - {}", value, err))
        },
        _ => Err(format!("Could not generate a random decimal from {}", value))
      },
      &Generator::RandomHexadecimal(digits) => match value {
        &Value::String(_) => Ok(json!(generate_hexadecimal(digits as usize))),
        _ => Err(format!("Could not generate a random hexadecimal from {}", value))
      },
      &Generator::RandomString(size) => match value {
        &Value::String(_) => Ok(json!(generate_ascii_string(size as usize))),
        _ => Err(format!("Could not generate a random string from {}", value))
      },
      &Generator::Regex(ref regex) => {
        let mut parser = regex_syntax::ParserBuilder::new().unicode(false).build();
        match parser.parse(regex) {
          Ok(hir) => {
            let gen = rand_regex::Regex::with_hir(hir, 20).unwrap();
            Ok(json!(rand::thread_rng().sample::<String, _>(gen)))
          },
          Err(err) => {
            log::warn!("'{}' is not a valid regular expression - {}", regex, err);
            Err(format!("Could not generate a random string from {} - {}", regex, err))
          }
        }
      },
      &Generator::Date(ref format) => match format {
        Some(pattern) => match parse_pattern(pattern) {
          Ok(tokens) => Ok(json!(Local::now().date().format(&to_chrono_pattern(&tokens)).to_string())),
          Err(err) => {
            log::warn!("Date format {} is not valid - {}", pattern, err);
            Err(format!("Could not generate a random date from {} - {}", pattern, err))
          }
        },
        None => Ok(json!(Local::now().naive_local().date().to_string()))
      },
      &Generator::Time(ref format) => match format {
        Some(pattern) => match parse_pattern(pattern) {
          Ok(tokens) => Ok(json!(Local::now().format(&to_chrono_pattern(&tokens)).to_string())),
          Err(err) => {
            log::warn!("Time format {} is not valid - {}", pattern, err);
            Err(format!("Could not generate a random time from {} - {}", pattern, err))
          }
        },
        None => Ok(json!(Local::now().time().format("%H:%M:%S").to_string()))
      },
      &Generator::DateTime(ref format) => match format {
        Some(pattern) => match parse_pattern(pattern) {
          Ok(tokens) => Ok(json!(Local::now().format(&to_chrono_pattern(&tokens)).to_string())),
          Err(err) => {
            log::warn!("DateTime format {} is not valid - {}", pattern, err);
            Err(format!("Could not generate a random date-time from {} - {}", pattern, err))
          }
        },
        None => Ok(json!(Local::now().format("%Y-%m-%dT%H:%M:%S.%3f%z").to_string()))
      },
      &Generator::RandomBoolean => Ok(json!(rand::thread_rng().gen::<bool>())),
      &Generator::ProviderStateGenerator(ref exp, ref dt) =>
        match generate_value_from_context(exp, context, dt) {
          Ok(val) => val.as_json(),
          Err(err) => Err(err)
        }
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

/// Trait to define a handler for applying generators to data of a particular content type.
pub trait ContentTypeHandler<T> {
  /// Processes the body using the map of generators, returning a (possibly) updated body.
  fn process_body(&mut self, generators: &HashMap<String, Generator>, context: &HashMap<String, Value>) -> OptionalBody;
  /// Applies the generator to the key in the body.
  fn apply_key(&mut self, key: &String, generator: &Generator, context: &HashMap<String, Value>);
}

/// Implementation of a content type handler for JSON
pub struct JsonHandler {
  /// JSON document to apply the generators to.
  pub value: Value
}

impl JsonHandler {
  fn query_object_graph(&self, path_exp: &Vec<PathToken>, tree: &mut Arena<String>, root: NodeId, body: Value) {
    let mut body_cursor = body;
    let mut it = path_exp.iter();
    let mut node_cursor = root;
    loop {
      match it.next() {
        Some(token) => {
          match token {
            &PathToken::Field(ref name) => {
              match body_cursor.clone().as_object() {
                Some(map) => match map.get(name) {
                  Some(val) => {
                    let node = tree.new_node(name.clone());
                    node_cursor.append(node, tree);
                    body_cursor = val.clone();
                    node_cursor = node;
                  },
                  None => return
                },
                None => return
              }
            },
            &PathToken::Index(index) => {
              match body_cursor.clone().as_array() {
                Some(list) => if list.len() > index {
                  let node = tree.new_node(format!("{}", index));
                  node_cursor.append(node, tree);
                  body_cursor = list[index].clone();
                  node_cursor = node;
                },
                None => return
              }
            }
            &PathToken::Star => {
              match body_cursor.clone().as_object() {
                Some(map) => {
                  let remaining = it.by_ref().cloned().collect();
                  for (key, val) in map {
                    let node = tree.new_node(key.clone());
                    node_cursor.append(node, tree);
                    body_cursor = val.clone();
                    self.query_object_graph(&remaining, tree, node, val.clone());
                  }
                },
                None => return
              }
            },
            &PathToken::StarIndex => {
              match body_cursor.clone().as_array() {
                Some(list) => {
                  let remaining = it.by_ref().cloned().collect();
                  for (index, val) in list.iter().enumerate() {
                    let node = tree.new_node(format!("{}", index));
                    node_cursor.append(node, tree);
                    body_cursor = val.clone();
                    self.query_object_graph(&remaining, tree, node,val.clone());
                  }
                },
                None => return
              }
            },
            _ => ()
          }
        },
        None => break
      }
    }
  }
}

impl ContentTypeHandler<Value> for JsonHandler {
  fn process_body(&mut self, generators: &HashMap<String, Generator>, context: &HashMap<String, Value>) -> OptionalBody {
    for (key, generator) in generators {
      self.apply_key(key, generator, context);
    };
    OptionalBody::Present(self.value.to_string().into(), Some("application/json".into()))
  }

  fn apply_key(&mut self, key: &String, generator: &Generator, context: &HashMap<String, Value>) {
    match parse_path_exp(key) {
      Ok(path_exp) => {
        let mut tree = Arena::new();
        let root = tree.new_node("".into());
        self.query_object_graph(&path_exp, &mut tree, root, self.value.clone());
        let expanded_paths = root.descendants(&tree).fold(Vec::<String>::new(), |mut acc, node_id| {
          let node = tree.index(node_id);
          if !node.data.is_empty() && node.first_child().is_none() {
            let path: Vec<String> = node_id.ancestors(&tree).map(|n| format!("{}", tree.index(n).data)).collect();
            if path.len() == path_exp.len() {
              acc.push(path.iter().rev().join("/"));
            }
          }
          acc
        });

        if !expanded_paths.is_empty() {
          for pointer_str in expanded_paths {
            match self.value.pointer_mut(&pointer_str) {
              Some(json_value) => match generator.generate_value(&json_value.clone(), context) {
                Ok(new_value) => *json_value = new_value,
                Err(_) => ()
              },
              None => ()
            }
          }
        } else if path_exp.len() == 1 {
          match generator.generate_value(&self.value.clone(), context) {
            Ok(new_value) => self.value = new_value,
            Err(_) => ()
          }
        }
      },
      Err(err) => log::warn!("Generator path '{}' is invalid, ignoring: {}", key, err)
    }
  }
}

/// Implementation of a content type handler for XML (currently unimplemented).
pub struct XmlHandler<'a> {
  /// XML document to apply the generators to.
  pub value: Document<'a>
}

impl <'a> ContentTypeHandler<Document<'a>> for XmlHandler<'a> {
  fn process_body(&mut self, _generators: &HashMap<String, Generator>, _context: &HashMap<String, Value>) -> OptionalBody {
    unimplemented!()
  }

  fn apply_key(&mut self, _key: &String, _generator: &Generator, _context: &HashMap<String, Value>) {
    unimplemented!()
  }
}

/// Data structure for representing a collection of generators
#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
#[serde(transparent)]
pub struct Generators {
  /// Map of generator categories to maps of generators
  pub categories: HashMap<GeneratorCategory, HashMap<String, Generator>>
}

impl Generators {
  /// If the generators are empty (that is there are no rules assigned to any categories)
  pub fn is_empty(&self) -> bool {
    self.categories.values().all(|category| category.is_empty())
  }

  /// If the generators are not empty (that is there is at least one rule assigned to a category)
  pub fn is_not_empty(&self) -> bool {
    self.categories.values().any(|category| !category.is_empty())
  }

  /// Loads the generators for a JSON map
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
                _ => log::warn!("Ignoring invalid generator JSON '{}' -> {:?}", sub_k, sub_v)
              }
            }
          },
          Err(err) => log::warn!("Ignoring generator with invalid category '{}' - {}", k, err)
        },
        _ => log::warn!("Ignoring invalid generator JSON '{}' -> {:?}", k, v)
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
          None => log::warn!("Ignoring invalid generator JSON '{:?}' with invalid type attribute -> {:?}", category, map)
        },
        _ => log::warn!("Ignoring invalid generator JSON '{:?}' with invalid type attribute -> {:?}", category, map)
      },
      None => log::warn!("Ignoring invalid generator JSON '{:?}' with no type attribute -> {:?}", category, map)
    }
  }

  fn to_json(&self) -> Value {
    Value::Object(self.categories.iter().fold(serde_json::Map::new(), |mut map, (name, category)| {
      let cat: String = name.clone().into();
      match name {
        &GeneratorCategory::PATH | &GeneratorCategory::METHOD | &GeneratorCategory::STATUS => {
          match category.get("") {
            Some(generator) => {
              map.insert(cat.clone(), generator.to_json());
            },
            None => ()
          }
        },
        _ => {
          let mut generators = serde_json::Map::new();
          for (key, val) in category {
            generators.insert(key.clone(), val.to_json());
          }
          map.insert(cat.clone(), Value::Object(generators));
        }
      }
      map
    }))
  }

  /// Adds the generator to the category (body, headers, etc.)
  pub fn add_generator(&mut self, category: &GeneratorCategory, generator: Generator) {
    self.add_generator_with_subcategory(category, "", generator);
  }

  /// Adds a generator to the category with a sub-category key (i.e. headers or query parameters)
  pub fn add_generator_with_subcategory<S: Into<String>>(&mut self, category: &GeneratorCategory,
                                                         subcategory: S, generator: Generator) {
    let category_map = self.categories.entry(category.clone()).or_insert(HashMap::new());
    category_map.insert(subcategory.into(), generator.clone());
  }

  /// If there are generators for the provided category, invokes the closure for all keys and values
  /// in the category.
  pub fn apply_generator<F>(&self, category: &GeneratorCategory, mut closure: F)
    where F: FnMut(&String, &Generator) {
    if self.categories.contains_key(category) && !self.categories[category].is_empty() {
      for (key, value) in self.categories[category].clone() {
        closure(&key, &value)
      }
    }
  }

  /// Applies all the body generators to the body and returns a new body (if anything was applied).
  pub fn apply_body_generators(&self, body: &OptionalBody, content_type: Option<ContentType>, context: &HashMap<String, Value>) -> OptionalBody {
    if body.is_present() && self.categories.contains_key(&GeneratorCategory::BODY) &&
      !self.categories[&GeneratorCategory::BODY].is_empty() {
      let generators = &self.categories[&GeneratorCategory::BODY];
      match content_type {
        Some(content_type) => if content_type.is_json() {
          let result: Result<Value, serde_json::Error> = serde_json::from_slice(&body.value());
          match result {
            Ok(val) => {
              let mut handler = JsonHandler { value: val };
              handler.process_body(&generators, context)
            },
            Err(err) => {
              log::error!("Failed to parse the body, so not applying any generators: {}", err);
              body.clone()
            }
          }
        } else if content_type.is_xml() {
          match parse_bytes(&body.value()) {
            Ok(val) => {
              let mut handler = XmlHandler { value: val.as_document() };
              handler.process_body(&generators, context)
            },
            Err(err) => {
              log::error!("Failed to parse the body, so not applying any generators: {}", err);
              body.clone()
            }
          }
        } else {
          body.clone()
        },
        _ => body.clone()
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

impl PartialEq for Generators {
  fn eq(&self, other: &Self) -> bool {
    self.categories == other.categories
  }

  fn ne(&self, other: &Self) -> bool {
    self.categories != other.categories
  }
}

impl Default for Generators {
  fn default() -> Self {
    Generators {
      categories: hashmap!{}
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
    &PactSpecification::V3 | &PactSpecification::V4 => generators.to_json(),
    _ => Value::Null
  }
}

/// Macro to make constructing generators easy
/// Example usage:
/// ```ignore
/// generators! {
///   "HEADER" => {
///     "A" => Generator::Uuid
///   }
/// }
///```
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

fn generate_value_from_context(expression: &String, context: &HashMap<String, Value>, data_type: &Option<DataType>) -> Result<DataValue, String> {
  let result = if contains_expressions(expression) {
    parse_expression(expression, &MapValueResolver { context: context.clone() })
  } else {
    context.get(expression).map(|v| json_to_string(v))
      .ok_or(format!("Value '{}' was not found in the provided context", expression))
  };
  data_type.clone().unwrap_or(DataType::RAW).wrap(result)
}

#[cfg(test)]
mod tests {
  use super::*;
  use expectest::prelude::*;
  use expectest::expect;
  use super::Generator;
  use std::str::FromStr;
  use hamcrest2::*;
  use crate::models::generators::Generator::{RandomInt, RandomDecimal, Regex};

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
    expect!(Generator::from_map(&s!("RandomInt"), &json!({ "min": 0, "max": 1234567890 }).as_object().unwrap())).to(be_some().value(Generator::RandomInt(0, 1234567890)));
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
  fn datetime_generator_from_json_test() {
    expect!(Generator::from_map(&s!("DateTime"), &serde_json::Map::new())).to(be_some().value(Generator::DateTime(None)));
    expect!(Generator::from_map(&s!("DateTime"), &json!({ "min": 5 }).as_object().unwrap())).to(be_some().value(Generator::DateTime(None)));
    expect!(Generator::from_map(&s!("DateTime"), &json!({ "format": "yyyy-MM-dd" }).as_object().unwrap())).to(be_some().value(Generator::DateTime(Some(s!("yyyy-MM-dd")))));
    expect!(Generator::from_map(&s!("DateTime"), &json!({ "format": 5 }).as_object().unwrap())).to(be_some().value(Generator::DateTime(Some(s!("5")))));
  }

  #[test]
  fn provider_state_generator_from_json_test() {
    expect!(Generator::from_map(&s!("ProviderState"), &serde_json::Map::new())).to(be_none());
    expect!(Generator::from_map(&s!("ProviderState"), &json!({ "expression": "5" }).as_object().unwrap())).to(
      be_some().value(Generator::ProviderStateGenerator("5".into(), None)));
    expect!(Generator::from_map(&s!("ProviderState"), &json!({ "expression": "5", "dataType": "INTEGER" }).as_object().unwrap())).to(
      be_some().value(Generator::ProviderStateGenerator("5".into(), Some(DataType::INTEGER))));
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
    expect!(Generator::DateTime(Some(s!("yyyyMMdd"))).to_json()).to(be_equal_to(json!({
      "type": "DateTime",
      "format": "yyyyMMdd"
    })));
    expect!(Generator::DateTime(None).to_json()).to(be_equal_to(json!({
      "type": "DateTime"
    })));
    expect!(Generator::ProviderStateGenerator("$a".into(), Some(DataType::INTEGER)).to_json()).to(be_equal_to(json!({
      "type": "ProviderState",
      "expression": "$a",
      "dataType": "INTEGER"
    })));
    expect!(Generator::ProviderStateGenerator("$a".into(), None).to_json()).to(be_equal_to(json!({
      "type": "ProviderState",
      "expression": "$a"
    })));
  }

  #[test]
  fn generators_to_json_test() {
    let mut generators = Generators::default();
    generators.add_generator(&GeneratorCategory::STATUS, RandomInt(200, 299));
    generators.add_generator(&GeneratorCategory::PATH, Regex("\\d+".into()));
    generators.add_generator(&GeneratorCategory::METHOD, RandomInt(200, 299));
    generators.add_generator_with_subcategory(&GeneratorCategory::BODY, "$.1", RandomDecimal(4));
    generators.add_generator_with_subcategory(&GeneratorCategory::BODY, "$.2", RandomDecimal(4));
    generators.add_generator_with_subcategory(&GeneratorCategory::HEADER, "A", RandomDecimal(4));
    generators.add_generator_with_subcategory(&GeneratorCategory::HEADER, "B", RandomDecimal(4));
    generators.add_generator_with_subcategory(&GeneratorCategory::QUERY, "a", RandomDecimal(4));
    generators.add_generator_with_subcategory(&GeneratorCategory::QUERY, "b", RandomDecimal(4));
    let json = generators.to_json();
    expect(json).to(be_equal_to(json!({
      "body": {
        "$.1": {"digits": 4, "type": "RandomDecimal"},
        "$.2": {"digits": 4, "type": "RandomDecimal"}
      },
      "header": {
        "A": {"digits": 4, "type": "RandomDecimal"},
        "B": {"digits": 4, "type": "RandomDecimal"}
      },
      "method": {"max": 299, "min": 200, "type": "RandomInt"},
      "path": {"regex": "\\d+", "type": "Regex"},
      "query": {
        "a": {"digits": 4, "type": "RandomDecimal"},
        "b": {"digits": 4, "type": "RandomDecimal"}
      },
      "status": {"max": 299, "min": 200, "type": "RandomInt"}
    })));
  }

  #[test]
  fn generate_decimal_test() {
    assert_that!(generate_decimal(4), matches_regex(r"^\d{1,3}\.\d{1,3}$"));
    assert_that!(generate_hexadecimal(4), matches_regex(r"^[0-9A-F]{4}$"));
  }

  #[test]
  fn generate_int_with_max_int_test() {
    assert_that!(Generator::RandomInt(0, i32::max_value()).generate_value(&0,
      &hashmap!{}).unwrap().to_string(), matches_regex(r"^\d+$"));
  }

  #[test]
  fn provider_state_generator_test() {
    expect!(Generator::ProviderStateGenerator("${a}".into(), Some(DataType::INTEGER)).generate_value(&0,
      &hashmap!{ "a".into() => json!(1234) })).to(be_ok().value(1234));
  }

  #[test]
  fn date_generator_test() {
    let generated = Generator::Date(None).generate_value(&"".to_string(), &hashmap!{});
    assert_that!(generated.unwrap(), matches_regex(r"^\d{4}-\d{2}-\d{2}$"));

    let generated2 = Generator::Date(Some("yyyy-MM-ddZ".into())).generate_value(&"".to_string(), &hashmap!{});
    assert_that!(generated2.unwrap(), matches_regex(r"^\d{4}-\d{2}-\d{2}[-+]\d{4}$"));
  }

  #[test]
  fn time_generator_test() {
    let generated = Generator::Time(None).generate_value(&"".to_string(), &hashmap!{});
    assert_that!(generated.unwrap(), matches_regex(r"^\d{2}:\d{2}:\d{2}$"));

    let generated2 = Generator::Time(Some("HH:mm:ssZ".into())).generate_value(&"".to_string(), &hashmap!{});
    assert_that!(generated2.unwrap(), matches_regex(r"^\d{2}:\d{2}:\d{2}[-+]\d+$"));
  }

  #[test]
  fn datetime_generator_test() {
    let generated = Generator::DateTime(None).generate_value(&"".to_string(), &hashmap!{});
    assert_that!(generated.unwrap(), matches_regex(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}[-+]\d+$"));

    let generated2 = Generator::DateTime(Some("yyyy-MM-dd HH:mm:ssZ".into())).generate_value(&"".to_string(), &hashmap!{});
    assert_that!(generated2.unwrap(), matches_regex(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}[-+]\d+$"));
  }

  #[test]
  fn regex_generator_test() {
    let generated = Generator::Regex(r"\d{4}\w{1,4}".into()).generate_value(&"".to_string(), &hashmap!{});
    assert_that!(generated.unwrap(), matches_regex(r"^\d{4}\w{1,4}$"));

    let generated = Generator::Regex(r"\d{1,2}/\d{1,2}".into()).generate_value(&"".to_string(), &hashmap!{});
    assert_that!(generated.unwrap(), matches_regex(r"^\d{1,2}/\d{1,2}$"));

    let generated = Generator::Regex(r"^\d{1,2}/\d{1,2}$".into()).generate_value(&"".to_string(), &hashmap!{});
    assert_that!(generated.unwrap(), matches_regex(r"^\d{1,2}/\d{1,2}$"));
  }

  #[test]
  fn uuid_generator_test() {
    let generated = Generator::Uuid.generate_value(&"".to_string(), &hashmap!{});
    assert_that!(generated.unwrap(), matches_regex(r"^[a-fA-F0-9]{8}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{12}$"));
  }

  #[test]
  fn random_decimal_generator_test() {
    for _ in 1..100 {
      let generated = Generator::RandomDecimal(10).generate_value(&"".to_string(), &hashmap! {}).unwrap();
      expect!(generated.clone().len()).to(be_equal_to(11));
      assert_that!(generated.clone(), matches_regex(r"^\d+\.\d+$"));
      let mut chars = generated.chars();
      let first_char = chars.next().unwrap();
      let second_char = chars.next().unwrap();
      println!("{}: '{}' != '0' || ('{}' == '0' && '{}' == '.')", generated, first_char, first_char, second_char);
      expect!(first_char != '0' || (first_char == '0' && second_char == '.')).to(be_true());
    }
  }

  #[test]
  fn handle_edge_case_when_digits_is_1() {
    let generated = Generator::RandomDecimal(1).generate_value(&"".to_string(), &hashmap! {}).unwrap();
    assert_that!(generated, matches_regex(r"^\d$"));
  }

  #[test]
  fn handle_edge_case_when_digits_is_2() {
    let generated = Generator::RandomDecimal(2).generate_value(&"".to_string(), &hashmap! {}).unwrap();
    assert_that!(generated, matches_regex(r"^\d\.\d$"));
  }
}
