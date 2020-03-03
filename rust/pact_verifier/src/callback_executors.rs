//! Executor abstraction for executing callbacks to user code (request filters, provider state change callbacks)

use pact_matching::models::Request;

/// Trait for executors that call request filters
pub trait RequestFilterExecutor {
  /// Filters requests based on some criteria.
  fn call(&self, request: &Request) -> Request;
}

/// A "null" request filter executor, which does nothing, but permits
/// bypassing of typechecking issues where no filter should be applied.
pub struct NullRequestFilterExecutor {
  // This field is added (and is private) to guarantee that this struct
  // is never instantiated accidentally, and is instead only able to be
  // used for type-level programming.
  _private_field: (),
}

impl RequestFilterExecutor for NullRequestFilterExecutor {
  fn call(&self, _request: &Request) -> Request {
    unimplemented!("NullRequestFilterExecutor should never be called")
  }
}
