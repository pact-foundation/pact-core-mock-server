//! Executor abstraction for executing callbacks to user code (request filters, provider state change callbacks)

use pact_matching::models::Request;

/// Trait for executors that call request filters
pub trait RequestFilterExecutor {
  fn call(&self, request: &Request) -> Request;
}
