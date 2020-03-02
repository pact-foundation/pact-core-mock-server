//! Executor abstraction for executing callbacks to user code (request filters, provider state change callbacks)

use pact_matching::models::Request;

/// Trait for executors that call request filters
pub trait RequestFilterExecutor {
  fn call(&self, request: &Request) -> Request;
}

/// A "null" request filter executor, which does nothing, but permits
/// bypassing of typechecking issues where no filter should be applied.
pub struct NullRequestFilterExecutor;

impl RequestFilterExecutor for NullRequestFilterExecutor {
  fn call(&self, request: &Request) -> Request {
    unimplemented!("NullRequestFilterExecutor should never be called")
  }
}
