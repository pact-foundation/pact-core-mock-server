//! Builder for constructing Asynchronous message interactions

use std::collections::HashMap;

use bytes::Bytes;
use log::*;
use maplit::hashmap;
use pact_models::content_types::ContentType;
use pact_models::json_utils::json_to_string;
use pact_models::plugins::PluginData;
use pact_models::prelude::{MatchingRules, OptionalBody, ProviderState};
use pact_models::v4::async_message::AsynchronousMessage;
use pact_models::v4::interaction::InteractionMarkup;
use pact_models::v4::message_parts::MessageContents;
use pact_plugin_driver::catalogue_manager::find_content_matcher;
use pact_plugin_driver::content::{ContentMatcher, InteractionContents, PluginConfiguration};
use pact_plugin_driver::plugin_models::PactPluginManifest;
use serde_json::{json, Map, Value};

#[derive(Clone, Debug)]
/// Asynchronous message interaction builder. Normally created via PactBuilder::message_interaction.
pub struct MessageInteractionBuilder {
  description: String,
  provider_states: Vec<ProviderState>,
  comments: Vec<String>,
  test_name: Option<String>,
  interaction_type: String,
  message_contents: InteractionContents,
  contents_plugin: Option<PactPluginManifest>,
  plugin_config: HashMap<String, PluginConfiguration>
}

impl MessageInteractionBuilder {
  /// Create a new message interaction builder
  pub fn new<D: Into<String>>(description: D, interaction_type: D) -> MessageInteractionBuilder {
    MessageInteractionBuilder {
      description: description.into(),
      provider_states: vec![],
      comments: vec![],
      test_name: None,
      interaction_type: interaction_type.into(),
      message_contents: Default::default(),
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

  /// The interaction we've built (in V4 format).
  pub fn build(&self) -> AsynchronousMessage {
    debug!("Building V4 AsynchronousMessage interaction: {:?}", self);

    let mut rules = MatchingRules::default();
    rules.add_category("body")
      .add_rules(self.message_contents.rules.as_ref().cloned().unwrap_or_default());
    AsynchronousMessage {
      id: None,
      key: None,
      description: self.description.clone(),
      provider_states: self.provider_states.clone(),
      contents: MessageContents {
        contents: self.message_contents.body.clone(),
        metadata: self.message_contents.metadata.as_ref().cloned().unwrap_or_default(),
        matching_rules: rules,
        generators: self.message_contents.generators.as_ref().cloned().unwrap_or_default()
      },
      comments: hashmap!{
        "text".to_string() => json!(self.comments),
        "testname".to_string() => json!(self.test_name)
      },
      pending: false,
      plugin_config: self.contents_plugin.as_ref().map(|plugin| {
        hashmap!{
          plugin.name.clone() => self.message_contents.plugin_config.interaction_configuration.clone()
        }
      }).unwrap_or_default(),
      interaction_markup: InteractionMarkup {
        markup: self.message_contents.interaction_markup.clone(),
        markup_type: self.message_contents.interaction_markup_type.clone()
      }
    }
  }

  /// Configure the interaction contents from a map
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
          self.setup_core_matcher(&ct, &contents_hashmap, Some(content_matcher));
        } else {
          debug!("Plugin matcher, will get the plugin to provide the interaction contents");
          match content_matcher.configure_interation(&ct, contents_hashmap).await {
            Ok((contents, plugin_config)) => {
              if let Some(contents) = contents.first() {
                self.message_contents = contents.clone();
                if !contents.plugin_config.is_empty() {
                  self.plugin_config.insert(content_matcher.plugin_name(), contents.plugin_config.clone());
                }
              }
              self.contents_plugin = content_matcher.plugin();

              if let Some(plugin_config) = plugin_config {
                let plugin_name = content_matcher.plugin_name();
                if self.plugin_config.contains_key(&*plugin_name) {
                  let entry = self.plugin_config.get_mut(&*plugin_name).unwrap();
                  for (k, v) in plugin_config.pact_configuration {
                    entry.pact_configuration.insert(k.clone(), v.clone());
                  }
                } else {
                  self.plugin_config.insert(plugin_name.to_string(), plugin_config.clone());
                }
              }
            }
            Err(err) => panic!("Failed to call out to plugin - {}", err)
          }
        }
      } else {
        debug!("No content matcher found, will use the internal implementation");
        self.setup_core_matcher(&ct, &contents_hashmap, None);
      }
    } else {
      self.message_contents = InteractionContents {
        body : OptionalBody::from(Value::Object(contents_map.clone())),
        .. InteractionContents::default()
      };
    }

    self
  }

  fn setup_core_matcher(
    &mut self,
    content_type: &ContentType,
    config: &HashMap<String, Value>,
    content_matcher: Option<ContentMatcher>
  ) {
    self.message_contents = InteractionContents {
      body: if let Some(contents) = config.get("contents") {
        OptionalBody::Present(
          Bytes::from(contents.to_string()),
          Some(content_type.clone()),
          None
        )
      } else {
        OptionalBody::Missing
      },
      .. InteractionContents::default()
    };

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
}
