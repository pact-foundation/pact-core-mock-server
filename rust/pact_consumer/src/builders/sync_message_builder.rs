//! Builder for constructing synchronous message interactions

use std::collections::HashMap;

use maplit::hashmap;
use pact_models::content_types::ContentType;
use pact_models::json_utils::json_to_string;
use pact_models::path_exp::DocPath;
use pact_models::plugins::PluginData;
use pact_models::prelude::{MatchingRuleCategory, MatchingRules, OptionalBody, ProviderState};
use pact_models::v4::interaction::InteractionMarkup;
use pact_models::v4::message_parts::MessageContents;
use pact_models::v4::sync_message::SynchronousMessage;
use pact_plugin_driver::catalogue_manager::find_content_matcher;
use pact_plugin_driver::content::{ContentMatcher, InteractionContents, PluginConfiguration};
use pact_plugin_driver::plugin_models::PactPluginManifest;
use serde_json::{json, Map, Value};
use tracing::debug;

use crate::prelude::{JsonPattern, Pattern, PluginInteractionBuilder};

#[derive(Clone, Debug)]
/// Synchronous message interaction builder. Normally created via PactBuilder::sync_message_interaction.
pub struct SyncMessageInteractionBuilder {
  description: String,
  provider_states: Vec<ProviderState>,
  comments: Vec<String>,
  test_name: Option<String>,
  request_contents: InteractionContents,
  response_contents: Vec<InteractionContents>,
  contents_plugin: Option<PactPluginManifest>,
  plugin_config: HashMap<String, PluginConfiguration>
}

impl SyncMessageInteractionBuilder {
  /// Create a new message interaction builder
  pub fn new<D: Into<String>>(description: D) -> SyncMessageInteractionBuilder {
    SyncMessageInteractionBuilder {
      description: description.into(),
      provider_states: vec![],
      comments: vec![],
      test_name: None,
      request_contents: Default::default(),
      response_contents: vec![],
      contents_plugin: None,
      plugin_config: Default::default()
    }
  }

  /// Specify a "provider state" for this interaction. This is normally use to
  /// set up database fixtures when using a pact to test a provider.
  pub fn given<G: Into<String>>(&mut self, given: G) -> &mut Self {
    self.provider_states.push(ProviderState::default(&given.into()));
    self
  }

  /// Adds a text comment to this interaction. This allows to specify just a bit more information
  /// about the interaction. It has no functional impact, but can be displayed in the broker HTML
  /// page, and potentially in the test output.
  pub fn comment<G: Into<String>>(&mut self, comment: G) -> &mut Self {
    self.comments.push(comment.into());
    self
  }

  /// Sets the test name for this interaction. This allows to specify just a bit more information
  /// about the interaction. It has no functional impact, but can be displayed in the broker HTML
  /// page, and potentially in the test output.
  pub fn test_name<G: Into<String>>(&mut self, name: G) -> &mut Self {
    self.test_name = Some(name.into());
    self
  }

  /// Adds a key/value pair to the message request metadata. The key can be anything that is
  /// convertible into a string, and the value must be conveyable into a JSON value.
  pub fn request_metadata<S: Into<String>, J: Into<Value>>(&mut self, key: S, value: J) -> &mut Self {
    let metadata = self.request_contents.metadata
      .get_or_insert_with(|| hashmap!{});
    metadata.insert(key.into(), value.into());
    self
  }

  /// The interaction we've built (in V4 format).
  pub fn build(&self) -> SynchronousMessage {
    debug!("Building V4 SynchronousMessages interaction: {:?}", self);

    let mut rules = MatchingRules::default();
    rules.add_category("body")
      .add_rules(self.request_contents.rules.as_ref().cloned().unwrap_or_default());
    SynchronousMessage {
      id: None,
      key: None,
      description: self.description.clone(),
      provider_states: self.provider_states.clone(),
      request: MessageContents {
        contents: self.request_contents.body.clone(),
        metadata: self.request_contents.metadata.as_ref().cloned().unwrap_or_default(),
        matching_rules: rules,
        generators: self.request_contents.generators.as_ref().cloned().unwrap_or_default()
      },
      response: self.response_contents.iter().map(|contents| {
        let mut rules = MatchingRules::default();
        rules.add_category("body")
          .add_rules(contents.rules.as_ref().cloned().unwrap_or_default());
        MessageContents {
          contents: contents.body.clone(),
          metadata: contents.metadata.as_ref().cloned().unwrap_or_default(),
          matching_rules: rules,
          generators: self.request_contents.generators.as_ref().cloned().unwrap_or_default()
        }
      }).collect(),
      comments: hashmap!{
        "text".to_string() => json!(self.comments),
        "testname".to_string() => json!(self.test_name)
      },
      pending: false,
      plugin_config: self.contents_plugin.as_ref().map(|plugin| {
        hashmap!{
          plugin.name.clone() => self.request_contents.plugin_config.interaction_configuration.clone()
        }
      }).unwrap_or_default(),
      interaction_markup: InteractionMarkup {
        markup: self.interaction_markup(),
        markup_type: self.request_contents.interaction_markup_type.clone()
      },
      transport: None
    }
  }

  fn interaction_markup(&self) -> String {
    let mut markup = self.request_contents.interaction_markup.clone();

    for interaction in &self.response_contents {
      if !interaction.interaction_markup.is_empty() {
        markup.push_str(interaction.interaction_markup.as_str());
      }
    }

    markup
  }

  /// Configure the interaction contents from a map of values
  pub async fn contents_from(&mut self, contents: Value) -> &mut Self {
    debug!("Configuring interaction from {:?}", contents);

    let contents_map = contents.as_object().cloned().unwrap_or(Map::default());
    let contents_hashmap = contents_map.iter()
      .map(|(k, v)| (k.clone(), v.clone())).collect();
    if let Some(content_type) = contents_map.get("pact:content-type") {
      let ct = ContentType::parse(json_to_string(content_type).as_str()).unwrap();
      if let Some(content_matcher) = find_content_matcher(&ct) {
        debug!("Found a matcher for '{}': {:?}", ct, content_matcher);
        if content_matcher.is_core() {
          debug!("Content matcher is a core matcher, will use the internal implementation");
          self.setup_core_matcher(Some(ct.clone()), &contents_hashmap, Some(content_matcher));
        } else {
          match content_matcher.configure_interation(&ct, contents_hashmap).await {
            Ok((contents, plugin_config)) => {
              if let Some(interaction) = contents.iter().find(|i| i.part_name == "request") {
                self.request_contents = interaction.clone();
              }

              for interaction in contents.iter().filter(|i| i.part_name == "response") {
                self.response_contents.push(interaction.clone());
              }

              self.contents_plugin = content_matcher.plugin();

              if let Some(plugin_config) = plugin_config {
                let plugin_name = content_matcher.plugin_name();
                self.add_plugin_config(plugin_config, plugin_name)
              }
            }
            Err(err) => panic!("Failed to call out to plugin - {}", err)
          }
        }
      } else {
        debug!("No content matcher found, will use the internal implementation");
        self.setup_core_matcher(Some(ct.clone()), &contents_hashmap, None);
      }
    } else {
      debug!("No content type provided, will use the internal implementation");
      self.setup_core_matcher(None, &contents_hashmap, None);
    }

    self
  }

  /// Configure the interaction contents from a plugin builder
  pub async fn contents_for_plugin<B: PluginInteractionBuilder>(&mut self, builder: B) -> &mut Self {
    self.contents_from(builder.build()).await
  }

  fn add_plugin_config(&mut self, plugin_config: PluginConfiguration, plugin_name: String) {
    if self.plugin_config.contains_key(&*plugin_name) {
      let entry = self.plugin_config.get_mut(&*plugin_name).unwrap();
      for (k, v) in plugin_config.pact_configuration {
        entry.pact_configuration.insert(k.clone(), v.clone());
      }
    } else {
      self.plugin_config.insert(plugin_name.to_string(), plugin_config.clone());
    }
  }

  fn setup_core_matcher(
    &mut self,
    content_type: Option<ContentType>,
    config: &HashMap<String, Value>,
    content_matcher: Option<ContentMatcher>
  ) {
    if let Some(request) = config.get("request") {
      let mut body = OptionalBody::from(request);
      if let Some(ct) = content_type.as_ref() {
        body.set_content_type(ct);
      }

      self.request_contents = InteractionContents {
        body, .. InteractionContents::default()
      };
    }
    if let Some(responses) = config.get("response") {
      match responses {
        Value::Array(responses) => {
          for response in responses {
            let mut body = OptionalBody::from(response);
            if let Some(ct) = content_type.as_ref() {
              body.set_content_type(ct);
            }

            self.response_contents.push(InteractionContents {
              body, .. InteractionContents::default()
            });
          }
        }
        _ => {
          let mut body = OptionalBody::from(responses);
          if let Some(ct) = content_type.as_ref() {
            body.set_content_type(ct);
          }

          self.response_contents.push(InteractionContents {
            body, .. InteractionContents::default()
          });
        }
      }
    }

    if let Some(_content_matcher) = content_matcher {
      // TODO: get the content matcher to apply the matching rules and generators
      //     val (body, rules, generators, _, _) = contentMatcher.setupBodyFromConfig(bodyConfig)
      //     val matchingRules = MatchingRulesImpl()
      //     if (rules != null) {
      //       matchingRules.addCategory(rules)
      //     }
      //     MessageContents(body, mapOf(), matchingRules, generators ?: Generators())
    }
  }

  /// Any global plugin config required to add to the Pact
  pub fn plugin_config(&self) -> Option<PluginData> {
    self.contents_plugin.as_ref().map(|plugin| {
      let config = if let Some(config) = self.plugin_config.get(plugin.name.as_str()) {
        config.pact_configuration.clone()
      } else {
        hashmap!{}
      };
      PluginData {
        name: plugin.name.clone(),
        version: plugin.version.clone(),
        configuration: config
      }
    })
  }

  /// Specify the body as `JsonPattern`, possibly including special matching
  /// rules.
  ///
  /// ```
  /// use pact_consumer::prelude::*;
  /// use pact_consumer::*;
  /// use pact_consumer::builders::SyncMessageInteractionBuilder;
  ///
  /// SyncMessageInteractionBuilder::new("hello message").request_json_body(json_pattern!({
  ///     "message": like!("Hello"),
  /// }));
  /// ```
  pub fn request_json_body<B: Into<JsonPattern>>(&mut self, body: B) -> &mut Self {
    let body = body.into();
    {
      let message_body = OptionalBody::Present(body.to_example().to_string().into(), Some("application/json".into()), None);
      let mut rules = MatchingRuleCategory::empty("content");
      body.extract_matching_rules(DocPath::root(), &mut rules);
      self.request_contents.body = message_body;
      if rules.is_not_empty() {
        match &mut self.request_contents.rules {
          None => self.request_contents.rules = Some(rules.clone()),
          Some(mr) => mr.add_rules(rules.clone())
        }
      }
    }
    self
  }

  /// Specify the body as `JsonPattern`, possibly including special matching
  /// rules. You can call this method multiple times, each will add a new response message to the
  /// interaction.
  ///
  /// ```
  /// use pact_consumer::prelude::*;
  /// use pact_consumer::*;
  /// use pact_consumer::builders::SyncMessageInteractionBuilder;
  ///
  /// SyncMessageInteractionBuilder::new("hello message").response_json_body(json_pattern!({
  ///     "message": like!("Hello"),
  /// }));
  /// ```
  pub fn response_json_body<B: Into<JsonPattern>>(&mut self, body: B) -> &mut Self {
    let body = body.into();
    {
      let message_body = OptionalBody::Present(body.to_example().to_string().into(), Some("application/json".into()), None);
      let mut rules = MatchingRuleCategory::empty("content");
      body.extract_matching_rules(DocPath::root(), &mut rules);
      self.response_contents.push(InteractionContents {
        part_name: "response".to_string(),
        body: message_body.clone(),
        rules: if rules.is_not_empty() { Some(rules) } else { None },
        .. InteractionContents::default()
      });
    }
    self
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;
  use serde_json::json;

  use crate::builders::SyncMessageInteractionBuilder;

  #[test]
  fn supports_setting_metadata_values() {
    let message = SyncMessageInteractionBuilder::new("test")
      .request_metadata("a", "a")
      .request_metadata("b", json!("b"))
      .request_metadata("c", vec![1, 2, 3])
      .build();
    expect!(message.request.metadata).to(be_equal_to(hashmap! {
      "a".to_string() => json!("a"),
      "b".to_string() => json!("b"),
      "c".to_string() => json!([1, 2, 3])
    }));
  }
}
