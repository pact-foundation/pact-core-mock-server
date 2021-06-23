//! `generators` module includes all the classes to deal with V3 format generators

use std::{collections::HashMap, ops::Index};

use indextree::{Arena, NodeId};
use itertools::Itertools;
use log::*;
use serde_json::{self, Value};
use sxd_document::dom::Document;

use pact_models::bodies::OptionalBody;
use pact_models::content_types::ContentType;
use pact_models::generators::{apply_generators, ContentTypeHandler, GenerateValue, Generator, GeneratorCategory, Generators, GeneratorTestMode};
use pact_models::matchingrules::MatchingRuleCategory;
use pact_models::path_exp::{parse_path_exp, PathToken};
use pact_models::xml_utils::parse_bytes;

use crate::{DiffConfig, MatchingContext};

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
  fn process_body(
    &mut self,
    generators: &HashMap<String, Generator>,
    mode: &GeneratorTestMode,
    context: &HashMap<&str, Value>
  ) -> Result<OptionalBody, String> {
    for (key, generator) in generators {
      if generator.corresponds_to_mode(mode) {
        debug!("Applying generator {:?} to key {}", generator, key);
        self.apply_key(key, generator, context);
      }
    };
    Ok(OptionalBody::Present(self.value.to_string().into(), Some("application/json".into())))
  }

  fn apply_key(&mut self, key: &String, generator: &dyn GenerateValue<Value>, context: &HashMap<&str, Value>) {
    match parse_path_exp(key) {
      Ok(path_exp) => {
        let mut tree = Arena::new();
        let root = tree.new_node("".into());
        self.query_object_graph(&path_exp, &mut tree, root, self.value.clone());
        let expanded_paths = root.descendants(&tree).fold(Vec::<String>::new(), |mut acc, node_id| {
          let node = tree.index(node_id);
          if !node.get().is_empty() && node.first_child().is_none() {
            let path: Vec<String> = node_id.ancestors(&tree).map(|n| format!("{}", tree.index(n).get())).collect();
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
  fn process_body(&mut self, _generators: &HashMap<String, Generator>, _mode: &GeneratorTestMode, _context: &HashMap<&str, Value>) -> Result<OptionalBody, String> {
    error!("UNIMPLEMENTED: Generators are not currently supported with XML");
    Err("Generators are not supported with XML".to_string())
  }

  fn apply_key(&mut self, _key: &String, _generator: &dyn GenerateValue<Document<'a>>, _context: &HashMap<&str, Value>) {
    error!("UNIMPLEMENTED: Generators are not currently supported with XML");
  }
}

/// If there are generators for the provided category, invokes the closure for all keys and values
/// in the category.
pub fn apply_generator<F>(generators: &Generators, mode: &GeneratorTestMode, category: &GeneratorCategory, mut closure: F)
  where F: FnMut(&String, &Generator) {
  if generators.categories.contains_key(category) && !generators.categories[category].is_empty() {
    apply_generators(mode, &generators.categories[category], &mut closure)
  }
}

/// Applies all the body generators to the body and returns a new body (if anything was applied).
pub fn apply_body_generators(generators: &Generators, mode: &GeneratorTestMode, body: &OptionalBody, content_type: Option<ContentType>, context: &HashMap<&str, Value>) -> OptionalBody {
  if body.is_present() && generators.categories.contains_key(&GeneratorCategory::BODY) &&
    !generators.categories[&GeneratorCategory::BODY].is_empty() {
    let generators = &generators.categories[&GeneratorCategory::BODY];
    generators_process_body(mode, &body, content_type, context, &generators)
  } else {
    body.clone()
  }
}

/// Apply the generators to the body, returning a new body
pub fn generators_process_body(
  mode: &GeneratorTestMode,
  body: &OptionalBody,
  content_type: Option<ContentType>,
  context: &HashMap<&str, Value>,
  generators: &HashMap<String, Generator>
) -> OptionalBody {
  match content_type {
    Some(content_type) => if content_type.is_json() {
      debug!("apply_body_generators: JSON content type");
      let result: Result<Value, serde_json::Error> = serde_json::from_slice(&body.value().unwrap_or_default());
      match result {
        Ok(val) => {
          let mut handler = JsonHandler { value: val };
          handler.process_body(&generators, mode, context).unwrap_or_else(|err| {
            error!("Failed to generate the body: {}", err);
            body.clone()
          })
        },
        Err(err) => {
          error!("Failed to parse the body, so not applying any generators: {}", err);
          body.clone()
        }
      }
    } else if content_type.is_xml() {
      debug!("apply_body_generators: XML content type");
      match parse_bytes(&body.value().unwrap_or_default()) {
        Ok(val) => {
          let mut handler = XmlHandler { value: val.as_document() };
          handler.process_body(&generators, mode, context).unwrap_or_else(|err| {
            error!("Failed to generate the body: {}", err);
            body.clone()
          })
        },
        Err(err) => {
          error!("Failed to parse the body, so not applying any generators: {}", err);
          body.clone()
        }
      }
    } else {
      warn!("Unsupported content type {} - Generators only support JSON and XML", content_type);
      body.clone()
    },
    _ => body.clone()
  }
}

pub(crate) fn find_matching_variant<T>(
  value: &T,
  variants: &Vec<(usize, MatchingRuleCategory, HashMap<String, Generator>)>,
  callback: &dyn Fn(&Vec<&str>, &T, &MatchingContext) -> bool
) -> Option<(usize, HashMap<String, Generator>)>
  where T: Clone + std::fmt::Debug {
  let result = variants.iter()
    .find(|(index, rules, _)| {
      debug!("find_matching_variant: Comparing variant {} with value '{:?}'", index, value);
      let context = MatchingContext::new(DiffConfig::NoUnexpectedKeys, rules);
      let matches = callback(&vec!["$"], value, &context);
      debug!("find_matching_variant: Comparing variant {} => {}", index, matches);
      matches
    });
  debug!("find_matching_variant: result = {:?}", result);
  result.map(|(index, _, generators)| (*index, generators.clone()))
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;
  use serde_json::json;

  use pact_models::matchingrules_list;
  use pact_models::generators::{GenerateValue, Generator};
  use pact_models::matchingrules::MatchingRule;

  #[test]
  #[ignore]
  // TODO: get this test passing after refactor
  fn array_contains_generator_test() {
    let generator = Generator::ArrayContains(vec![
      (0, matchingrules_list! {
        "body"; "$.href" => [ MatchingRule::Regex(".*(\\/orders\\/\\d+)$".into()) ]
      }, hashmap! {
        "$.href".to_string() => Generator::MockServerURL("http://localhost:8080/orders/1234".into(), ".*(\\/orders\\/\\d+)$".into())
      }),
      (1, matchingrules_list! {
        "body"; "$.href" => [ MatchingRule::Regex(".*(\\/orders\\/\\d+)$".into()) ]
      }, hashmap! {
        "$.href".to_string() => Generator::MockServerURL("http://localhost:8080/orders/1234".into(), ".*(\\/orders\\/\\d+)$".into())
      })
    ]);
    let value = json!([
      {
        "href": "http://localhost:9000/orders/1234",
        "method": "PUT",
        "name": "update"
      },
      {
        "href": "http://localhost:9000/orders/1234",
        "method": "DELETE",
        "name": "delete"
      }
    ]);
    let context = hashmap!{
      "mockServer" => json!({
        "href": "https://somewhere.else:1234/subpath"
      })
    };
    let generated = generator.generate_value(&value, &context);
    expect!(generated.clone()).to(be_ok());
    let generated_value = generated.unwrap();
    assert_eq!(generated_value, json!([
      {
        "href": "https://somewhere.else:1234/subpath/orders/1234",
        "method": "PUT",
        "name": "update"
      },
      {
        "href": "https://somewhere.else:1234/subpath/orders/1234",
        "method": "DELETE",
        "name": "delete"
      }
    ]));
  }
}
