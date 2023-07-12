use std::fs::File;
use std::io::Write;

#[cfg(feature = "junit")] use junit_report::{ReportBuilder, TestCaseBuilder, TestSuiteBuilder};
use serde_json::Value;
use tracing::debug;

#[cfg(feature = "junit")] use pact_verifier::{interaction_mismatch_output, MismatchResult};
use pact_verifier::verification_result::VerificationExecutionResult;

pub(crate) fn write_json_report(result: &VerificationExecutionResult, file_name: &str) -> anyhow::Result<()> {
  debug!("Writing JSON result of the verification to '{file_name}'");
  let mut f = File::create(file_name)?;
  let json: Value = result.into();
  f.write_all(json.to_string().as_bytes())?;
  Ok(())
}

#[cfg(feature = "junit")]
pub(crate) fn write_junit_report(result: &VerificationExecutionResult, file_name: &str, provider: &String) -> anyhow::Result<()> {
  debug!("Writing JUnit result of the verification to '{file_name}'");
  let mut f = File::create(file_name)?;

  let mut test_suite = TestSuiteBuilder::new(provider);
  test_suite.set_system_out(result.output.join("\n").as_str());
  for interaction_result in &result.interaction_results {
    let duration = time::Duration::try_from(interaction_result.duration).unwrap_or_default();
    let test_case = match &interaction_result.result {
      Ok(_) => TestCaseBuilder::success(interaction_result.description.as_str(), duration),
      Err(result) => {
        if interaction_result.pending {
          TestCaseBuilder::skipped(interaction_result.description.as_str())
        } else {
          match result {
            MismatchResult::Mismatches { mismatches, expected, actual, .. } => {
              let mut output_buffer = vec![];
              interaction_mismatch_output(&mut output_buffer, false, 1, &interaction_result.description,
                &mismatches, expected.as_ref(), actual.as_ref());
              let mut builder = TestCaseBuilder::failure(
                interaction_result.description.as_str(),
                duration,
                "",
                "Verification for interaction failed"
              );
              builder.set_system_out(output_buffer.join("\n").as_str());
              builder
            },
            MismatchResult::Error(error, _) => TestCaseBuilder::error(
              interaction_result.description.as_str(),
              duration,
              "",
              error.as_str()
            )
          }
        }
      }
    };
    test_suite.add_testcase(test_case.build());
  }

  let mut report_builder = ReportBuilder::new();
  report_builder.add_testsuite(test_suite.build());
  let report = report_builder.build();
  report.write_xml(&mut f)?;
  Ok(())
}
