use std::future::Future;

use pact_plugin_driver::catalogue_manager::register_core_entries;
use pact_plugin_driver::plugin_manager::load_plugin;
use pact_plugin_driver::plugin_models::PluginDependency;

use pact_matching::{CONTENT_MATCHER_CATALOGUE_ENTRIES, MATCHER_CATALOGUE_ENTRIES};
use pact_mock_server::MOCK_SERVER_CATALOGUE_ENTRIES;
use pact_models::{Consumer, Provider};
use pact_models::interaction::Interaction;
use pact_models::pact::Pact;
use pact_models::sync_pact::RequestResponsePact;
use pact_models::v4::pact::V4Pact;

use crate::prelude::*;

use super::interaction_builder::InteractionBuilder;
use std::path::PathBuf;

/// Builder for `Pact` objects.
///
/// ```
/// use pact_consumer::prelude::*;
/// use pact_consumer::*;
///
/// # tokio_test::block_on(async {
/// let pact = PactBuilder::new("Greeting Client", "Greeting Server")
///     .interaction("asks for a greeting", "", |mut i| async move {
///         i.request.path("/greeting/hello");
///         i.response
///             .header("Content-Type", "application/json")
///             .json_body(json_pattern!({ "message": "hello" }));
///         i
///     })
///     .await
///     .build();
///
/// // The request method and response status default as follows.
/// assert_eq!(pact.interactions()[0].as_request_response().unwrap().request.method, "GET");
/// assert_eq!(pact.interactions()[0].as_request_response().unwrap().response.status, 200);
/// # });
/// ```
pub struct PactBuilder {
  pact: Box<dyn Pact + Send>,
  output_dir: Option<PathBuf>
}

impl PactBuilder {
    /// Create a new `PactBuilder`, specifying the names of the service
    /// consuming the API and the service providing it.
    pub fn new<C, P>(consumer: C, provider: P) -> Self
    where
        C: Into<String>,
        P: Into<String>,
    {
        register_core_entries(CONTENT_MATCHER_CATALOGUE_ENTRIES.as_ref());
        register_core_entries(MATCHER_CATALOGUE_ENTRIES.as_ref());
        register_core_entries(MOCK_SERVER_CATALOGUE_ENTRIES.as_ref());

        let mut pact = RequestResponsePact::default();
        pact.consumer = Consumer {
            name: consumer.into(),
        };
        pact.provider = Provider {
            name: provider.into(),
        };
        PactBuilder { pact: pact.boxed(), output_dir: None }
    }

    /// Create a new `PactBuilder` for a V4 specification Pact, specifying the names of the service
    /// consuming the API and the service providing it.
    pub fn new_v4<C, P>(consumer: C, provider: P) -> Self
      where
        C: Into<String>,
        P: Into<String>
    {
      register_core_entries(CONTENT_MATCHER_CATALOGUE_ENTRIES.as_ref());
      register_core_entries(MATCHER_CATALOGUE_ENTRIES.as_ref());
      register_core_entries(MOCK_SERVER_CATALOGUE_ENTRIES.as_ref());

      let pact = V4Pact {
        consumer: Consumer { name: consumer.into() },
        provider: Provider { name: provider.into() },
        .. V4Pact::default()
      };
      PactBuilder { pact: pact.boxed(), output_dir: None }
    }

    /// Add a plugin to be used by the test
    ///
    /// Panics:
    /// Plugins only work with V4 specification pacts. This method will panic if the pact
    /// being built is V3 format. Use `PactBuilder::new_v4` to create a builder with a V4 format
    /// pact.
    pub async fn using_plugin(&mut self, name: &str, version: Option<String>) -> &mut Self {
      if !self.pact.is_v4() {
        panic!("Plugins require V4 specification pacts. Use PactBuilder::new_v4");
      }

      let result = load_plugin(&PluginDependency {
        name: name.to_string(),
        version,
        dependency_type: Default::default()
      }).await;
      match result {
        Ok(plugin) => self.pact.add_plugin(plugin.manifest.name.as_str(), Some(plugin.manifest.version))
          .expect("Could not add plugin to pact"),
        Err(err) => panic!("Could not load plugin - {}", err)
      }

      self
    }

    /// Add a new HTTP `Interaction` to the `Pact`. Needs to return a clone of the method
    /// that is passed in.
    pub async fn interaction<D, F, O>(&mut self, description: D, interaction_type: D, build_fn: F) -> &mut Self
    where
        D: Into<String>,
        F: FnOnce(InteractionBuilder) -> O,
        O: Future<Output=InteractionBuilder> + Send
    {
        let interaction = InteractionBuilder::new(description.into(), interaction_type.into());
        let interaction = build_fn(interaction).await;
        self.push_interaction(&interaction.build())
    }

    /// Directly add a pre-built `Interaction` to our `Pact`. Normally it's
    /// easier to use `interaction` instead of this function.
    pub fn push_interaction(&mut self, interaction: &dyn Interaction) -> &mut Self {
      self.pact.add_interaction(interaction).unwrap();
      self
    }

    /// Return the `Pact` we've built.
    pub fn build(&self) -> Box<dyn Pact + Send> {
      self.pact.boxed()
    }

  /// Sets the output directory to write pact files to
  pub fn output_dir<D: Into<PathBuf>>(&mut self, dir: D) -> &mut Self {
    self.output_dir = Some(dir.into());
    self
  }
}

impl StartMockServer for PactBuilder {
    fn start_mock_server(&self) -> ValidatingMockServer {
        ValidatingMockServer::start(self.build(), self.output_dir.clone())
    }
}
