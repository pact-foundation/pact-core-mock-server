use pact_matching::{CONTENT_MATCHER_CATALOGUE_ENTRIES, MATCHER_CATALOGUE_ENTRIES};
use pact_mock_server::MOCK_SERVER_CATALOGUE_ENTRIES;
use pact_models::{Consumer, Provider};
use pact_models::interaction::Interaction;
use pact_models::pact::Pact;
use pact_models::sync_pact::RequestResponsePact;
use pact_models::v4::pact::V4Pact;

use crate::prelude::*;

use super::interaction_builder::InteractionBuilder;
use pact_plugin_driver::catalogue_manager::register_core_entries;

/// Builder for `Pact` objects.
///
/// ```
/// use pact_consumer::prelude::*;
/// use pact_consumer::*;
///
/// let pact = PactBuilder::new("Greeting Client", "Greeting Server")
///     .interaction("asks for a greeting", |i| {
///         i.request.path("/greeting/hello");
///         i.response
///             .header("Content-Type", "application/json")
///             .json_body(json_pattern!({ "message": "hello" }));
///     })
///     .build();
///
/// // The request method and response status default as follows.
/// assert_eq!(pact.interactions()[0].as_request_response().unwrap().request.method, "GET");
/// assert_eq!(pact.interactions()[0].as_request_response().unwrap().response.status, 200);
/// ```
pub struct PactBuilder {
  pact: Box<dyn Pact>,
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
        PactBuilder { pact: pact.boxed() }
    }

    /// Create a new `PactBuilder` for a V4 specification Pact, specifying the names of the service
    /// consuming the API and the service providing it.
    pub fn new_v4<C, P>(consumer: C, provider: P) -> Self
      where
        C: Into<String>,
        P: Into<String>
    {
      let pact = V4Pact {
        consumer: Consumer { name: consumer.into() },
        provider: Provider { name: provider.into() },
        .. V4Pact::default()
      };
      PactBuilder { pact: pact.boxed() }
    }

    /// Add a new HTTP `Interaction` to the `Pact`.
    pub fn interaction<D, F>(&mut self, description: D, build_fn: F) -> &mut Self
    where
        D: Into<String>,
        F: FnOnce(&mut InteractionBuilder),
    {
        let mut interaction = InteractionBuilder::new(description.into());
        build_fn(&mut interaction);
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
}

impl StartMockServer for PactBuilder {
    fn start_mock_server(&self) -> ValidatingMockServer {
        ValidatingMockServer::start(self.build())
    }
}
