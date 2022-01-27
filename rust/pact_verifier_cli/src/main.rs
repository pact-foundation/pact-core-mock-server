//! # Standalone Pact Verifier
//!
//! This project provides a command line interface to verify pact files against a running provider. It is a single executable binary. It implements the [V2 Pact specification](https://github.com/pact-foundation/pact-specification/tree/version-2).
//!
//! [Online rust docs](https://docs.rs/pact_verifier_cli/)
//!
//! The Pact Verifier works by taking all the interactions (requests and responses) from a number of pact files. For each interaction, it will make the request defined in the pact to a running service provider and check the response received back against the one defined in the pact file. All mismatches will then be reported.
//!
//! ## Command line interface
//!
//! The pact verifier is bundled as a single binary executable `pact_verifier_cli`. Running this with out any options displays the standard help.
//!
//! ```console,ignore
//! pact_verifier_cli v0.6.2
//! Standalone Pact verifier
//!
//! USAGE:
//!     pact_verifier_cli [FLAGS] [OPTIONS] --broker-url <broker-url>... --dir <dir>... --file <file>... --provider-name <provider-name> --url <url>...
//!
//! FLAGS:
//!         --enable-pending           Enables Pending Pacts
//!         --filter-no-state          Only validate interactions that have no defined provider state
//!         --help                     Prints help information
//!         --publish                  Enables publishing of verification results back to the Pact Broker. Requires the
//!                                    broker-url and provider-version parameters.
//!         --state-change-as-query    State change request data will be sent as query parameters instead of in the request
//!                                    body
//!         --state-change-teardown    State change teardown requests are to be made after each interaction
//!     -v, --version                  Prints version information
//!
//! OPTIONS:
//!         --base-path <base-path>                                Base path to add to all requests
//!     -b, --broker-url <broker-url>...
//!             URL of the pact broker to fetch pacts from to verify (requires the provider name parameter) [env:
//!             PACT_BROKER_BASE_URL=https://testdemo.pactflow.io]
//!         --build-url <build-url>
//!             URL of the build to associate with the published verification results.
//!
//!         --consumer-version-tags <consumer-version-tags>
//!             Consumer tags to use when fetching pacts from the Broker. Accepts comma-separated values.
//!
//!     -d, --dir <dir>...                                         Directory of pact files to verify (can be repeated)
//!     -f, --file <file>...                                       Pact file to verify (can be repeated)
//!     -c, --filter-consumer <filter-consumer>...
//!             Consumer name to filter the pacts to be verified (can be repeated)
//!
//!         --filter-description <filter-description>
//!             Only validate interactions whose descriptions match this filter
//!
//!         --filter-state <filter-state>
//!             Only validate interactions whose provider states match this filter
//!
//!     -h, --hostname <hostname>                                  Provider hostname (defaults to localhost)
//!         --include-wip-pacts-since <include-wip-pacts-since>
//!             Allow pacts that don't match given consumer selectors (or tags) to  be verified, without causing the overall
//!             task to fail. For more information, see https://pact.io/wip
//!     -l, --loglevel <loglevel>
//!             Log level (defaults to warn) [possible values: error, warn, info, debug,
//!             trace, none]
//!         --password <password>
//!             Password to use when fetching pacts from URLS [env: PACT_BROKER_PASSWORD=]
//!
//!     -p, --port <port>                                          Provider port (defaults to protocol default 80/443)
//!     -n, --provider-name <provider-name>                        Provider name (defaults to provider)
//!         --provider-tags <provider-tags>
//!             Provider tags to use when publishing results. Accepts comma-separated values.
//!
//!         --provider-version <provider-version>
//!             Provider version that is being verified. This is required when publishing results.
//!
//!     -s, --state-change-url <state-change-url>                  URL to post state change requests to
//!     -t, --token <token>
//!             Bearer token to use when fetching pacts from URLS [env: PACT_BROKER_TOKEN=Dk8qO3_ZOqau8EeMaagK5w]
//!
//!     -u, --url <url>...                                         URL of pact file to verify (can be repeated)
//!         --user <user>
//!             Username to use when fetching pacts from URLS [env: PACT_BROKER_USERNAME=]
//! ```
//!
//! ## Options
//!
//! ### Log Level
//!
//! You can control the log level with the `-l, --loglevel <loglevel>` option. It defaults to warn, and the options that you can specify are: error, warn, info, debug, trace, none.
//!
//! ### Pact File Sources
//!
//! You can specify the pacts to verify with the following options. They can be repeated to set multiple sources.
//!
//! | Option | Type | Description |
//! |--------|------|-------------|
//! | `-f, --file <file>` | File | Loads a pact from the given file |
//! | `-u, --url <url>` | URL | Loads a pact from a URL resource |
//! | `-d, --dir <dir>` | Directory | Loads all the pacts from the given directory |
//! | `-b, --broker-url <broker-url>` | Pact Broker | Loads all the pacts for the provider from the pact broker. Requires the `-n, --provider-name <provider-name>` option |
//!
//! ### Provider Options
//!
//! The running provider can be specified with the following options:
//!
//! | Option | Description |
//! |--------|-------------|
//! | `-h, --hostname <hostname>` | The provider hostname, defaults to `localhost` |
//! | `-p, --port <port>` | The provider port (defaults to 8080) |
//! | `-n, --provider-name <provider-name>` | The name of the provider. Required if you are loading pacts from a pact broker |
//!
//! ### Filtering the interactions
//!
//! The interactions that are verified can be filtered by the following options:
//!
//! #### `-c, --filter-consumer <filter-consumer>`
//!
//! This will only verify the interactions of matching consumers. You can specify multiple consumers by either seperating the names with a comma, or repeating the option.
//!
//! #### `--filter-description <filter-description>`
//!
//! This option will filter the interactions that are verified that match by desciption. You can use a regular expression to match.
//!
//! #### `--filter-state <filter-state>`
//!
//! This option will filter the interactions that are verified that match by provider state. You can use a regular expression to match. Can't be used with the `--filter-no-state` option.
//!
//! #### `--filter-no-state`
//!
//! This option will filter the interactions that are verified that don't have a defined provider state. Can't be used with the `--filter-state` option.
//!
//! ### State change requests
//!
//! Provider states are a mechanism to define the state that the provider needs to be in to be able to verify a particular request. This is achieved by setting a state change URL that will receive a POST request with the provider state before the actual request is made.
//!
//! #### `-s, --state-change-url <state-change-url>`
//!
//! This sets the URL that the POST requests will be made to before each actual request.
//!
//! #### `--state-change-as-query`
//!
//! By default, the state for the state change request will be sent as a JSON document in the body of the request. This option forces it to be sent as a query parameter instead.
//!
//! #### `--state-change-teardown`
//!
//! This option will cause the verifier to also make a tear down request after the main request is made. It will receive a second field in the body or a query parameter named `action` with the value `teardown`.
//!
//! ## Example run
//!
//! This will verify all the pacts for the `happy_provider` found in the pact broker (running on localhost) against the provider running on localhost port 5050. Only the pacts for the consumers `Consumer` and `Consumer2` will be verified.
//!
//! ```console,ignore
//! $ pact_verifier_cli -b http://localhost -n 'happy_provider' -p 5050 --filter-consumer Consumer --filter-consumer Consumer2
//! 21:59:28 [WARN] pact_matching::models: No metadata found in pact file "http://localhost/pacts/provider/happy_provider/consumer/Consumer/version/1.0.0", assuming V1.1 specification
//! 21:59:28 [WARN] pact_matching::models: No metadata found in pact file "http://localhost/pacts/provider/happy_provider/consumer/Consumer2/version/1.0.0", assuming V1.1 specification
//!
//! Verifying a pact between Consumer and happy_provider
//!   Given I am friends with Fred
//!     WARNING: State Change ignored as there is no state change URL
//!   Given I have no friends
//!     WARNING: State Change ignored as there is no state change URL
//!   a request to unfriend but no friends
//!     returns a response which
//!       has status code 200 (OK)
//!       includes headers
//!       has a matching body (OK)
//!   a request friends
//!     returns a response which
//!       has status code 200 (FAILED)
//!       includes headers
//!         "Content-Type" with value "application/json" (FAILED)
//!       has a matching body (FAILED)
//!   a request to unfriend
//!     returns a response which
//!       has status code 200 (OK)
//!       includes headers
//!         "Content-Type" with value "application/json" (OK)
//!       has a matching body (FAILED)
//!
//!
//! Verifying a pact between Consumer2 and happy_provider
//!   Given I am friends with Fred
//!     WARNING: State Change ignored as there is no state change URL
//!   Given I have no friends
//!     WARNING: State Change ignored as there is no state change URL
//!   a request to unfriend but no friends
//!     returns a response which
//!       has status code 200 (OK)
//!       includes headers
//!       has a matching body (OK)
//!   a request friends
//!     returns a response which
//!       has status code 200 (FAILED)
//!       includes headers
//!         "Content-Type" with value "application/json" (FAILED)
//!       has a matching body (FAILED)
//!   a request to unfriend
//!     returns a response which
//!       has status code 200 (OK)
//!       includes headers
//!         "Content-Type" with value "application/json" (OK)
//!       has a matching body (FAILED)
//!
//!
//! Failures:
//!
//! 0) Verifying a pact between Consumer and happy_provider - a request friends returns a response which has a matching body
//!     expected "application/json" body but was "text/plain"
//!
//! 1) Verifying a pact between Consumer and happy_provider - a request friends returns a response which has status code 200
//!     expected 200 but was 404
//!
//! 2) Verifying a pact between Consumer and happy_provider - a request friends returns a response which includes header "Content-Type" with value "application/json"
//!     Expected header "Content-Type" to have value "application/json" but was "text/plain"
//!
//! 3) Verifying a pact between Consumer and happy_provider Given I am friends with Fred - a request to unfriend returns a response which has a matching body
//!     $.body -> Type mismatch: Expected Map {"reply":"Bye"} but received  "Ok"
//!
//!
//! 4) Verifying a pact between Consumer2 and happy_provider - a request friends returns a response which has a matching body
//!     expected "application/json" body but was "text/plain"
//!
//! 5) Verifying a pact between Consumer2 and happy_provider - a request friends returns a response which has status code 200
//!     expected 200 but was 404
//!
//! 6) Verifying a pact between Consumer2 and happy_provider - a request friends returns a response which includes header "Content-Type" with value "application/json"
//!     Expected header "Content-Type" to have value "application/json" but was "text/plain"
//!
//! 7) Verifying a pact between Consumer2 and happy_provider Given I am friends with Fred - a request to unfriend returns a response which has a matching body
//!     $.body -> Type mismatch: Expected Map {"reply":"Bye"} but received  "Ok"
//!
//!
//!
//! There were 8 pact failures
//!
//! ```

#![warn(missing_docs)]

// Due to large generated future for async fns
#![type_length_limit="100000000"]

use std::env;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use clap::{AppSettings, ArgMatches, ErrorKind};
use log::{debug, error, LevelFilter};
use pact_models::{PACT_RUST_VERSION, PactSpecification};
use pact_models::prelude::HttpAuth;
use simplelog::{ColorChoice, Config, TerminalMode, TermLogger};
use tokio::time::sleep;

use pact_verifier::{FilterInfo, NullRequestFilterExecutor, PactSource, ProviderInfo, VerificationOptions, verify_provider_async, PublishOptions};
use pact_verifier::callback_executors::HttpRequestProviderStateExecutor;
use pact_verifier::metrics::VerificationMetrics;
use pact_verifier::selectors::{consumer_tags_to_selectors, json_to_selectors};

mod args;

/// Handles the command line arguments from the running process
pub async fn handle_cli(version: &str) -> Result<(), i32> {
  let args: Vec<String> = env::args().collect();
  let program = args[0].clone();
  let app = args::setup_app(program, version);
  let matches = app
    .setting(AppSettings::ArgRequiredElseHelp)
    .setting(AppSettings::ColoredHelp)
    .get_matches_safe();

  match matches {
    Ok(results) => handle_matches(&results).await,
    Err(ref err) => {
      match err.kind {
        ErrorKind::HelpDisplayed => {
          println!("{}", err.message);
          Ok(())
        },
        ErrorKind::VersionDisplayed => {
          print_version(version);
          println!();
          Ok(())
        },
        _ => {
          err.exit()
        }
      }
    }
  }
}

async fn handle_matches(matches: &clap::ArgMatches<'_>) -> Result<(), i32> {
  let level = matches.value_of("loglevel").unwrap_or("warn");
  let log_level = match level {
    "none" => LevelFilter::Off,
    _ => LevelFilter::from_str(level).unwrap()
  };
  TermLogger::init(log_level, Config::default(), TerminalMode::Mixed, ColorChoice::Auto).unwrap_or_default();
  let provider = ProviderInfo {
    host: matches.value_of("hostname").unwrap_or("localhost").to_string(),
    port: matches.value_of("port").map(|port| port.parse::<u16>().unwrap()),
    path: matches.value_of("base-path").unwrap_or("/").into(),
    protocol: matches.value_of("scheme").unwrap_or("http").to_string(),
    .. ProviderInfo::default()
  };
  let source = pact_source(matches);
  let filter = interaction_filter(matches);
  let provider_state_executor = Arc::new(HttpRequestProviderStateExecutor {
    state_change_url: matches.value_of("state-change-url").map(|s| s.to_string()),
    state_change_body: !matches.is_present("state-change-as-query"),
    state_change_teardown: matches.is_present("state-change-teardown")
  });

  let verification_options = VerificationOptions {
    request_filter: None::<Arc<NullRequestFilterExecutor>>,
    disable_ssl_verification: matches.is_present("disable-ssl-verification"),
    request_timeout: matches.value_of("request-timeout")
      .map(|t| t.parse::<u64>().unwrap_or(5000)).unwrap_or(5000),
  };

  let publish_options = if matches.is_present("publish") {
    Some(PublishOptions {
      provider_version: matches.value_of("provider-version").map(|v| v.to_string()),
      build_url: matches.value_of("build-url").map(|v| v.to_string()),
      provider_tags: matches.values_of("provider-tags")
        .map_or_else(Vec::new, |tags| tags.map(|tag| tag.to_string()).collect()),
      provider_branch: matches.value_of("provider-branch").map(|v| v.to_string())
    })
  } else {
    None
  };

  for s in &source {
    debug!("Pact source to verify = {}", s);
  };

  verify_provider_async(
    provider,
    source,
    filter,
    matches.values_of_lossy("filter-consumer").unwrap_or_default(),
    &verification_options,
    publish_options.as_ref(),
    &provider_state_executor,
    Some(VerificationMetrics {
      test_framework: "pact_verifier_cli".to_string(),
      app_name: "pact_verifier_cli".to_string(),
      app_version: env!("CARGO_PKG_VERSION").to_string()
    })
  ).await
    .map_err(|err| {
      error!("Verification failed with error: {}", err);
      2
    })
    .and_then(|(result, _)| if result { Ok(()) } else { Err(1) })
}

fn print_version(version: &str) {
  println!("\npact verifier version   : v{}", version);
  println!("pact specification      : v{}", PactSpecification::V4.version_str());
  println!("models version          : v{}", PACT_RUST_VERSION.unwrap_or_default());
}

fn pact_source(matches: &ArgMatches) -> Vec<PactSource> {
  let mut sources = vec![];

  if let Some(values) = matches.values_of("file") {
    sources.extend(values.map(|v| PactSource::File(v.to_string())).collect::<Vec<PactSource>>());
  };

  if let Some(values) = matches.values_of("dir") {
    sources.extend(values.map(|v| PactSource::Dir(v.to_string())).collect::<Vec<PactSource>>());
  };

  if let Some(values) = matches.values_of("url") {
    sources.extend(values.map(|v| {
      if matches.is_present("user") {
        PactSource::URL(v.to_string(), matches.value_of("user").map(|user| {
          HttpAuth::User(user.to_string(), matches.value_of("password").map(|p| p.to_string()))
        }))
      } else if matches.is_present("token") {
        PactSource::URL(v.to_string(), matches.value_of("token").map(|token| HttpAuth::Token(token.to_string())))
      } else {
        PactSource::URL(v.to_string(), None)
      }
    }).collect::<Vec<PactSource>>());
  };

  if let Some(broker_url) = matches.value_of("broker-url") {
    let name = matches.value_of("provider-name").map(|n| n.to_string()).unwrap_or_default();
    let auth = matches.value_of("user").map(|user| {
      HttpAuth::User(user.to_string(), matches.value_of("password").map(|p| p.to_string()))
    }).or_else(|| matches.value_of("token").map(|t| HttpAuth::Token(t.to_string())));

    let source = if matches.is_present("consumer-version-selectors") || matches.is_present("consumer-version-tags") {
      let pending = matches.is_present("enable-pending");
      let wip = matches.value_of("include-wip-pacts-since").map(|wip| wip.to_string());
      let provider_tags = matches.values_of("provider-tags")
        .map_or_else(Vec::new, |tags| tags.map(|tag| tag.to_string()).collect());
      let provider_branch = matches.value_of("provider-branch").map(|v| v.to_string());

      let selectors = if matches.is_present("consumer-version-selectors") {
        matches.values_of("consumer-version-selectors")
          .map_or_else(Vec::new, |s| json_to_selectors(s.collect::<Vec<_>>()))
      } else if matches.is_present("consumer-version-tags") {
        matches.values_of("consumer-version-tags")
          .map_or_else(Vec::new, |tags| consumer_tags_to_selectors(tags.collect::<Vec<_>>()))
      } else {
        vec![]
      };

      PactSource::BrokerWithDynamicConfiguration {
        provider_name: name,
        broker_url: broker_url.into(),
        enable_pending: pending,
        include_wip_pacts_since: wip,
        provider_tags,
        provider_branch,
        selectors,
        auth,
        links: vec![]
      }
    } else {
      PactSource::BrokerUrl(name, broker_url.to_string(), auth, vec![])
    };
    sources.push(source);
  };
  sources
}

fn interaction_filter(matches: &ArgMatches) -> FilterInfo {
  if matches.is_present("filter-description") &&
    (matches.is_present("filter-state") || matches.is_present("filter-no-state")) {
    if matches.is_present("filter-state") {
      FilterInfo::DescriptionAndState(matches.value_of("filter-description").unwrap().to_string(),
                                      matches.value_of("filter-state").unwrap().to_string())
    } else {
      FilterInfo::DescriptionAndState(matches.value_of("filter-description").unwrap().to_string(),
                                      String::new())
    }
  } else if matches.is_present("filter-description") {
    FilterInfo::Description(matches.value_of("filter-description").unwrap().to_string())
  } else if matches.is_present("filter-state") {
    FilterInfo::State(matches.value_of("filter-state").unwrap().to_string())
  } else if matches.is_present("filter-no-state") {
    FilterInfo::State(String::new())
  } else {
    FilterInfo::None
  }
}

fn main() {
  let runtime = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .expect("Could not start a Tokio runtime for running async tasks");

  let result = runtime.block_on(async {
    let result = handle_cli(clap::crate_version!()).await;

    // Add a small delay to let asynchronous tasks to complete
    sleep(Duration::from_millis(500)).await;

    result
  });

  runtime.shutdown_timeout(Duration::from_millis(500));

  if let Err(err) = result {
    std::process::exit(err);
  }
}
