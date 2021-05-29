use pact_matching::models::*;
use pact_matching::models::v4::SynchronousHttp;
use pact_models::provider_states::ProviderState;

use super::request_builder::RequestBuilder;
use super::response_builder::ResponseBuilder;

/// Builder for `Interaction` objects. Normally created via
/// `PactBuilder::interaction`.
pub struct InteractionBuilder {
    description: String,
    provider_states: Vec<ProviderState>,
    comments: Vec<String>,
    test_name: Option<String>,

    /// A builder for this interaction's `Request`.
    pub request: RequestBuilder,

    /// A builder for this interaction's `Response`.
    pub response: ResponseBuilder,
}

impl InteractionBuilder {
  /// Create a new interaction.
  pub fn new<D: Into<String>>(description: D) -> Self {
    InteractionBuilder {
      description: description.into(),
      provider_states: vec![],
      comments: vec![],
      test_name: None,
      request: RequestBuilder::default(),
      response: ResponseBuilder::default(),
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

  /// The interaction we've built.
  pub fn build(&self) -> RequestResponseInteraction {
    RequestResponseInteraction {
      id: None,
      description: self.description.clone(),
      provider_states: self.provider_states.clone(),
      request: self.request.build(),
      response: self.response.build(),
    }
  }

  /// The interaction we've built (in V4 format).
  pub fn build_v4(&self) -> SynchronousHttp {
    SynchronousHttp {
      id: None,
      key: None,
      description: self.description.clone(),
      provider_states: self.provider_states.clone(),
      request: self.request.build().as_v4_request(),
      response: self.response.build().as_v4_response(),
      comments: Default::default(),
      pending: false
    }
  }
}
