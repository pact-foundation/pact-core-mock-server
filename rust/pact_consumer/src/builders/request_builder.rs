use std::collections::HashMap;

use bytes::Bytes;
#[cfg(test)]
#[allow(unused_imports)]
use env_logger;
use log::debug;
use maplit::*;
use pact_plugin_driver::catalogue_manager::find_content_matcher;
#[cfg(test)]
use regex::Regex;
#[cfg(test)]
use serde_json::json;
use serde_json::Value;

use pact_models::bodies::OptionalBody;
use pact_models::content_types::ContentType;
use pact_models::expression_parser::DataType;
use pact_models::generators::{Generator, GeneratorCategory, Generators};
use pact_models::http_parts::HttpPart;
use pact_models::json_utils::body_from_json;
use pact_models::matchingrules::{Category, MatchingRules};
use pact_models::path_exp::DocPath;
use pact_models::request::Request;
use pact_models::v4::http_parts::HttpRequest;

use crate::prelude::*;
use crate::util::GetDefaulting;

/// Builder for `Request` objects. Normally created via `PactBuilder`.
#[derive(Clone, Debug)]
pub struct RequestBuilder {
    request: HttpRequest,
}

impl RequestBuilder {
    /// Specify the request method. Defaults to `"GET"`.
    ///
    /// ```
    /// use pact_consumer::builders::RequestBuilder;
    /// use pact_consumer::prelude::*;
    ///
    /// let request = RequestBuilder::default().method("POST").build();
    /// assert_eq!(request.method, "POST");
    /// ```
    pub fn method<M: Into<String>>(&mut self, method: M) -> &mut Self {
        self.request.method = method.into();
        self
    }

    /// Set the HTTP method to `GET`. This is the default, so we don't actually
    /// care.
    pub fn get(&mut self) -> &mut Self {
        self.method("GET")
    }

    /// Set the HTTP method to `POST`.
    pub fn post(&mut self) -> &mut Self {
        self.method("POST")
    }

    /// Set the HTTP method to `PUT`.
    pub fn put(&mut self) -> &mut Self {
        self.method("PUT")
    }

    /// Set the HTTP method to `DELETE`.
    pub fn delete(&mut self) -> &mut Self {
        self.method("DELETE")
    }

    /// Specify the request path. Defaults to `"/"`.
    pub fn path<P: Into<StringPattern>>(&mut self, path: P) -> &mut Self {
        let path = path.into();
        self.request.path = path.to_example();
        path.extract_matching_rules(
            DocPath::empty(),
            self.request.matching_rules.add_category(Category::PATH),
        );
        self
    }

    /// Specify the request path with generators. Defaults to `"/"`.
    pub fn path_from_provider_state<E, P: Into<StringPattern>>(&mut self, expression: E, path: P) -> &mut Self
        where
          E: Into<String>
    {
        let path = path.into();
        let expression = expression.into();
        self.path(path);
        {
            let generators = self.generators();
            generators.add_generator(&GeneratorCategory::PATH, Generator::ProviderStateGenerator(expression, Some(DataType::STRING)))
        }
        self
    }

    /// Specify a query parameter. You may pass either a single value or
    /// a list of values to represent a repeated parameter.
    ///
    /// ```
    /// use pact_consumer::*;
    /// use pact_consumer::builders::RequestBuilder;
    /// use regex::Regex;
    ///
    /// RequestBuilder::default()
    ///     .query_param("simple", "value")
    ///     .query_param("pattern", term!("^[0-9]+$", "123"));
    /// ```
    ///
    /// To pass multiple parameters with the same name, call `query_param` more
    /// than once with the same `key`.
    pub fn query_param<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: Into<String>,
        V: Into<StringPattern>,
    {
        let key = key.into();
        let value = value.into();

        // Extract our example JSON and add it the `Vec` for the appropriate
        // parameter.
        self.request
            .query
            .get_defaulting()
            .entry(key.clone())
            .or_insert_with(Default::default)
            .push(value.to_example());

        let mut path = DocPath::root();
        path.push_field(key);

        // Extract our matching rules.
        value.extract_matching_rules(
            path,
            self.request.matching_rules.add_category("query"),
        );

        self
    }

    /// Build the specified `Request` object.
    pub fn build(&self) -> Request {
         self.request.as_v3_request()
    }

    /// Build the specified `Request` object in V4 format.
    pub fn build_v4(&self) -> HttpRequest {
        self.request.clone()
    }

  // TODO: This needs to setup rules/generators based on the content type
  fn setup_core_matcher(&mut self, content_type: &ContentType, definition: Value) {
    match definition {
      Value::String(s) => self.request.body = OptionalBody::Present(Bytes::from(s), Some(content_type.clone()), None),
      Value::Object(ref o) => if o.contains_key("contents") {
        self.request.body = body_from_json(&definition, "contents", &None);
      }
      _ => {}
    }
  }

  /// Set the body using the definition. If the body is being supplied by a plugin,
  /// this is what is sent to the plugin to setup the body.
  pub async fn contents(&mut self, content_type: ContentType, definition: Value) -> &mut Self {
    match find_content_matcher(&content_type) {
      Some(matcher) => {
        debug!("Found a matcher for '{}': {:?}", content_type, matcher);
        if matcher.is_core() {
          debug!("Matcher is from the core framework");
          self.setup_core_matcher(&content_type, definition);
        } else {
          let request = &mut self.request;
          debug!("Plugin matcher, will get the plugin to provide the response contents");
          match definition {
            Value::Object(attributes) => {
              let map = attributes.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
              let result = matcher.configure_content(&content_type, map).await;
              match result {
                Ok((body, rules, generators)) => {
                  request.body = body.clone();
                  if !request.has_header("content-type") {
                    request.add_header("content-type", vec![content_type.to_string().as_str()]);
                  }
                  if let Some(rules) = rules {
                    request.matching_rules.add_rules("body", rules);
                  }
                  if let Some(generators) = generators {
                    request.generators.add_generators(generators);
                  }
                }
                Err(err) => panic!("Failed to call out to plugin - {}", err)
              }
            }
            _ => panic!("{} is not a valid value for contents", definition)
          }
        }
      }
      None => {
        debug!("No matcher was found, will default to the core framework");
        self.setup_core_matcher(&content_type, definition);
      }
    }
    self
  }
}

impl Default for RequestBuilder {
    fn default() -> Self {
        RequestBuilder { request: HttpRequest::default() }
    }
}

impl HttpPartBuilder for RequestBuilder {
  fn headers_and_matching_rules_mut(&mut self) -> (&mut HashMap<String, Vec<String>>, &mut MatchingRules) {
    (
      self.request.headers.get_or_insert(hashmap!{}),
      &mut self.request.matching_rules,
    )
  }

  fn generators(&mut self) -> &mut Generators {
    &mut self.request.generators
  }

  fn body_and_matching_rules_mut(&mut self) -> (&mut OptionalBody, &mut MatchingRules) {
      (
          &mut self.request.body,
          &mut self.request.matching_rules,
      )
  }
}

#[tokio::test]
async fn path_pattern() {
    let greeting_regex = Regex::new("/greeting/.*").unwrap();
    let pattern = PactBuilder::new("C", "P")
        .interaction("I", "", |mut i| {
            i.request.path(Term::new(greeting_regex, "/greeting/hello"));
            futures::future::ready(i)
        })
        .await
        .build();
    let good = PactBuilder::new("C", "P")
        .interaction("I", "", |mut i| { i.request.path("/greeting/hi"); futures::future::ready(i) })
        .await
        .build();
    let bad = PactBuilder::new("C", "P")
        .interaction("I", "", |mut i| { i.request.path("/farewell/bye"); futures::future::ready(i) })
        .await
        .build();
    assert_requests_match!(good, pattern);
    assert_requests_do_not_match!(bad, pattern);
}

#[tokio::test]
async fn path_generator() {
    let actual = PactBuilder::new("C", "P")
      .interaction("I", "", |mut i| {
          i.request.path_from_provider_state("/greeting/${greeting}", "/greeting/hi");
          futures::future::ready(i)
      })
      .await
      .build();

    let expected = PactBuilder::new("C", "P")
      .interaction("I", "", |mut i| {
          i.request.path("/greeting/hello");
          futures::future::ready(i)
      })
      .await
      .build();

    let good_context = &mut HashMap::new();
    good_context.insert("greeting", json!("hello"));
    assert_requests_with_context_match!(actual, expected, good_context);

    let bad_context = &mut HashMap::new();
    bad_context.insert("greeting", json!("goodbye"));
    assert_requests_with_context_do_not_match!(actual, expected, bad_context);
}

#[tokio::test]
async fn query_param_pattern() {
    let pattern = PactBuilder::new("C", "P")
        .interaction("I", "", |mut i| {
            i.request.query_param("greeting", term!("^h.*$", "hello"));
            futures::future::ready(i)
        })
        .await
        .build();
    let good = PactBuilder::new("C", "P")
        .interaction("I", "", |mut i| { i.request.query_param("greeting", "hi"); futures::future::ready(i) })
        .await
        .build();
    let bad = PactBuilder::new("C", "P")
        .interaction("I", "", |mut i| { i.request.query_param("greeting", "bye"); futures::future::ready(i) })
        .await
        .build();
    assert_requests_match!(good, pattern);
    assert_requests_do_not_match!(bad, pattern);
}

#[tokio::test]
async fn query_param_with_underscore() {
    let pattern = PactBuilder::new("C", "P")
        .interaction("get a user", "", |mut i| {
            i.request
                .path("/users")
                // This `term!` was being ignored in `pact_matching`, but only
                // if there was an underscore.
                .query_param("user_id", term!("^[0-9]+$", "1"));
            futures::future::ready(i)
        })
        .await
        .build();
    let good = PactBuilder::new("C", "P")
        .interaction("I", "", |mut i| {
            i.request
                .path("/users")
                // Call with a different ID than we expected.
                .query_param("user_id", "2");
            futures::future::ready(i)
        })
        .await
        .build();
    assert_requests_match!(good, pattern);
}

#[tokio::test]
async fn term_does_not_require_anchors() {
    use crate::prelude::*;

    let pattern = PactBuilder::new("C", "P")
        .interaction("I", "", |mut i| {
            // Unfortunately, we appear to need a leading "^" and trailing "$"
            // on this regex, or else it will match the other examples below.
            i.request.path(term!("^/users/[0-9]+$", "/users/12"));
            futures::future::ready(i)
        })
        .await
        .build();
    let good = PactBuilder::new("C", "P")
        .interaction("I", "", |mut i| { i.request.path("/users/2"); futures::future::ready(i) })
        .await
        .build();
    let bad1 = PactBuilder::new("C", "P")
        .interaction("I", "", |mut i| { i.request.path("/users/2/posts"); futures::future::ready(i) })
        .await
        .build();
    let bad2 = PactBuilder::new("C", "P")
        .interaction("I", "", |mut i| { i.request.path("/account/1/users/2"); futures::future::ready(i) })
        .await
        .build();
    assert_requests_match!(good, pattern);
    assert_requests_do_not_match!(bad1, pattern);
    assert_requests_do_not_match!(bad2, pattern);
}
