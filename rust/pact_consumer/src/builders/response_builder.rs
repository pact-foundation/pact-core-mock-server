use std::collections::HashMap;

use bytes::Bytes;
use log::debug;
use maplit::*;
use pact_plugin_driver::catalogue_manager::find_content_matcher;
use pact_plugin_driver::content::PluginConfiguration;
use serde_json::Value;

use pact_models::bodies::OptionalBody;
use pact_models::generators::Generators;
use pact_models::http_parts::HttpPart;
use pact_models::matchingrules::MatchingRules;
use pact_models::prelude::ContentType;
use pact_models::response::Response;
use pact_models::v4::http_parts::{body_from_json, HttpResponse};

use crate::prelude::*;

/// Builder for `Response` objects. Normally created via `PactBuilder`.
#[derive(Clone, Debug)]
pub struct ResponseBuilder {
  response: HttpResponse,
  plugin_config: Option<PluginConfiguration>
}

impl ResponseBuilder {
    /// Set the status code for the response. Defaults to `200`.
    ///
    /// ```
    /// use pact_consumer::builders::ResponseBuilder;
    /// use pact_consumer::prelude::*;
    ///
    /// let response = ResponseBuilder::default().status(404).build();
    /// assert_eq!(response.status, 404);
    /// ```
    pub fn status(&mut self, status: u16) -> &mut Self {
        self.response.status = status;
        self
    }

    // This is a partial list of popular HTTP status codes. If you use any
    // others regularly, feel free to add them.

    /// Set the status code to `200 OK`. (This is default.)
    pub fn ok(&mut self) -> &mut Self {
        self.status(200)
    }

    /// Set the status code to `201 Created`.
    pub fn created(&mut self) -> &mut Self {
        self.status(201)
    }

    /// Set the status code to `204 No Content`.
    pub fn no_content(&mut self) -> &mut Self {
        self.status(204)
    }

    /// Set the status code to `401 Unauthorized`.
    pub fn unauthorized(&mut self) -> &mut Self {
        self.status(401)
    }

    /// Set the status code to `403 Forbidden`.
    pub fn forbidden(&mut self) -> &mut Self {
        self.status(403)
    }

    /// Set the status code to `404 Not Found`.
    pub fn not_found(&mut self) -> &mut Self {
        self.status(404)
    }

    /// Build the specified `Response` object.
    pub fn build(&self) -> Response {
        self.response.as_v3_response()
    }

    /// Build the specified `Response` object in V4 format.
    pub fn build_v4(&self) -> HttpResponse {
      self.response.clone()
    }

  // TODO: This needs to setup rules/generators based on the content type
  fn setup_core_matcher(&mut self, content_type: &ContentType, definition: Value) {
    match definition {
      Value::String(s) => self.response.body = OptionalBody::Present(Bytes::from(s), Some(content_type.clone()), None),
      Value::Object(ref o) => if o.contains_key("contents") {
        self.response.body = body_from_json(&definition, "contents", &None);
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
          let response = &mut self.response;
          debug!("Plugin matcher, will get the plugin to provide the response contents");
          match definition {
            Value::Object(attributes) => {
              let map = attributes.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
              match matcher.configure_interation(&content_type, map).await {
                Ok(contents) => {
                  response.body = contents.body.clone();
                  if !response.has_header("content-type") {
                    response.add_header("content-type", vec![content_type.to_string().as_str()]);
                  }
                  if let Some(rules) = contents.rules {
                    response.matching_rules.add_rules("body", rules);
                  }
                  if let Some(generators) = contents.generators {
                    response.generators.add_generators(generators);
                  }
                  if !contents.plugin_config.is_empty() {
                    self.plugin_config = Some(contents.plugin_config.clone());
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

impl Default for ResponseBuilder {
    fn default() -> Self {
        ResponseBuilder { response: HttpResponse::default(), plugin_config: None }
    }
}

impl HttpPartBuilder for ResponseBuilder {
  fn headers_and_matching_rules_mut(&mut self) -> (&mut HashMap<String, Vec<String>>, &mut MatchingRules) {
    (
      self.response.headers.get_or_insert(hashmap!{}),
      &mut self.response.matching_rules,
    )
  }

  fn generators(&mut self) -> &mut Generators {
    &mut self.response.generators
  }

  fn body_and_matching_rules_mut(&mut self) -> (&mut OptionalBody, &mut MatchingRules) {
    (
      &mut self.response.body,
      &mut self.response.matching_rules,
    )
  }
}
