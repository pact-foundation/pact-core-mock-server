use std::collections::HashMap;

use anyhow::anyhow;
use log::debug;
use serde_json::Value;

use pact_matching::{generate_request, match_request};
use pact_models::generators::GeneratorTestMode;
use pact_models::pact::Pact;

/// Check that all requests in `actual` match the patterns provide by
/// `expected`, and raise an error if anything fails.
pub(crate) fn check_requests_match(
    actual_label: &str,
    actual: &Box<dyn Pact + Send>,
    expected_label: &str,
    expected: &Box<dyn Pact + Send>,
    context: &HashMap<&str, Value>
) -> anyhow::Result<()> {
    // First make sure we have the same number of interactions.
    if expected.interactions().len() != actual.interactions().len() {
        return Err(anyhow!(
                "the pact `{}` has {} interactions, but `{}` has {}",
                expected_label,
                expected.interactions().len(),
                actual_label,
                actual.interactions().len(),
            ));
    }

    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;

    // Next, check each interaction to see if it matches.
    for (e, a) in expected.interactions().iter().zip(actual.interactions()) {
        let actual_request = a.as_request_response().unwrap().request.clone();
        debug!("actual_request = {:?}", actual_request);
        let generated_request = generate_request(&actual_request, &GeneratorTestMode::Provider, context);
        debug!("generated_request = {:?}", generated_request);
        let mismatches = runtime.block_on(async {
            match_request(e.as_request_response().unwrap().request.clone(),
                generated_request).await
        });
        if !mismatches.all_matched() {
          let mut reasons = String::new();
          for mismatch in mismatches.mismatches() {
            reasons.push_str(&format!("- {}\n", mismatch.description()));
          }
          return Err(anyhow!(
            "the pact `{}` does not match `{}` because:\n{}",
            expected_label,
            actual_label,
            reasons,
          ));
        }
    }

    Ok(())
}

macro_rules! assert_requests_match {
    ($actual:expr, $expected:expr) => (
        {
            let result = $crate::test_support::check_requests_match(
                stringify!($actual),
                &($actual),
                stringify!($expected),
                &($expected),
                &HashMap::new(),
            );
            if let ::std::result::Result::Err(message) = result {
                panic!("{}", message)
            }
        }
    )
}

macro_rules! assert_requests_do_not_match {
    ($actual:expr, $expected:expr) => (
        {
            let result = $crate::test_support::check_requests_match(
                stringify!($actual),
                &($actual),
                stringify!($expected),
                &($expected),
                &HashMap::new(),
            );
            if let ::std::result::Result::Ok(()) = result {
                panic!(
                    "pact `{}` unexpectedly matched pattern `{}`",
                    stringify!($actual),
                    stringify!($expected),
                );
            }
        }
    )
}

macro_rules! assert_requests_with_context_match {
    ($actual:expr, $expected:expr, $context:expr) => (
        {
            let result = $crate::test_support::check_requests_match(
                stringify!($actual),
                &($actual),
                stringify!($expected),
                &($expected),
                $context,
            );
            if let ::std::result::Result::Err(message) = result {
                panic!("{}", message)
            }
        }
    )
}

macro_rules! assert_requests_with_context_do_not_match {
    ($actual:expr, $expected:expr, $context:expr) => (
        {
            let result = $crate::test_support::check_requests_match(
                stringify!($actual),
                &($actual),
                stringify!($expected),
                &($expected),
                $context,
            );
            if let ::std::result::Result::Ok(()) = result {
                panic!(
                    "pact `{}` unexpectedly matched pattern `{}`",
                    stringify!($actual),
                    stringify!($expected),
                );
            }
        }
    )
}
