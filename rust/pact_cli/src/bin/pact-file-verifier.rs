//! Pact file format validator
//!
//! Validator for Pact files.

#![warn(missing_docs)]

use std::env;
use std::fs::File;
use std::str::FromStr;

use anyhow::{anyhow, Context};
use clap::{App, AppSettings, Arg, ArgMatches, ErrorKind};
use log::*;
use serde_json::Value;
use simplelog::{ColorChoice, Config, TerminalMode, TermLogger};

use pact_cli::{setup_loggers, verification};
use pact_cli::verification::{display_results, VerificationResult};
use pact_matching::models::{determine_spec_version, http_utils, MessagePact, parse_meta_data, RequestResponsePact};
use pact_matching::models::http_utils::HttpAuth;
use pact_matching::models::v4::V4Pact;
use pact_models::PactSpecification;
use pact_models::verify_json::{json_type_of, PactFileVerificationResult, PactJsonVerifier, ResultLevel};

fn setup_app<'a, 'b>(program: &str, version: &'b str) -> App<'a, 'b> {
  App::new(program)
    .version(version)
    .about("Pact file format verifier")
    .version_short("v")
    .arg(Arg::with_name("loglevel")
      .short("l")
      .long("loglevel")
      .takes_value(true)
      .use_delimiter(false)
      .possible_values(&["error", "warn", "info", "debug", "trace", "none"])
      .help("Log level (defaults to warn)"))
    .arg(Arg::with_name("file")
      .short("f")
      .long("file")
      .required_unless_one(&["url"])
      .takes_value(true)
      .use_delimiter(false)
      .multiple(true)
      .number_of_values(1)
      .empty_values(false)
      .help("Pact file to verify (can be repeated)"))
    .arg(Arg::with_name("url")
      .short("u")
      .long("url")
      .required_unless_one(&["file"])
      .takes_value(true)
      .use_delimiter(false)
      .multiple(true)
      .number_of_values(1)
      .empty_values(false)
      .help("URL of pact file to verify (can be repeated)"))
    .arg(Arg::with_name("spec")
      .long("specification")
      .short("s")
      .takes_value(true)
      .possible_values(&["v1", "v2", "v3", "v4", "auto"])
      .default_value("auto")
      .help("Pact specification to verify as. Defaults to detecting the version from the Pact file."))
    .arg(Arg::with_name("user")
      .long("user")
      .takes_value(true)
      .use_delimiter(false)
      .number_of_values(1)
      .empty_values(false)
      .conflicts_with("token")
      .help("Username to use when fetching pacts from URLS"))
    .arg(Arg::with_name("password")
      .long("password")
      .takes_value(true)
      .use_delimiter(false)
      .number_of_values(1)
      .empty_values(false)
      .conflicts_with("token")
      .help("Password to use when fetching pacts from URLS"))
    .arg(Arg::with_name("token")
      .short("t")
      .long("token")
      .takes_value(true)
      .use_delimiter(false)
      .number_of_values(1)
      .empty_values(false)
      .conflicts_with("user")
      .help("Bearer token to use when fetching pacts from URLS"))
    .arg(Arg::with_name("output")
      .short("o")
      .long("output")
      .takes_value(true)
      .possible_values(&["console", "json"])
      .default_value("console")
      .help("Format to use to output results as"))
    .arg(Arg::with_name("strict")
      .long("strict")
      .help("Enable strict validation. This will reject things like additional attributes"))
}

fn handle_cli() -> Result<(), i32> {
  let args: Vec<String> = env::args().collect();
  let program = args[0].clone();
  let app = setup_app(&program, clap::crate_version!());
  let matches = app
    .setting(AppSettings::ArgRequiredElseHelp)
    .setting(AppSettings::ColoredHelp)
    .get_matches_safe();

  match matches {
    Ok(results) => handle_matches(&results),
    Err(ref err) => {
      match err.kind {
        ErrorKind::HelpDisplayed => {
          println!("{}", err.message);
          Ok(())
        },
        ErrorKind::VersionDisplayed => Ok(()),
        _ => err.exit()
      }
    }
  }
}

fn handle_matches(args: &ArgMatches) -> Result<(), i32> {
  let log_level = args.value_of("loglevel");
  if let Err(err) = setup_loggers(log_level.unwrap_or("warn")) {
    eprintln!("WARN: Could not setup loggers: {}", err);
    eprintln!();
  }

  let spec_version = args.value_of("spec").map(|version| {
    if version == "auto" {
      PactSpecification::Unknown
    } else {
      PactSpecification::from(version)
    }
  }).unwrap_or(PactSpecification::Unknown);

  let files = load_files(args).map_err(|_| 1)?;

  let results = files.iter().map(|(source, pact_json)| {
    let spec_version = match spec_version {
      PactSpecification::Unknown => {
        let metadata = parse_meta_data(pact_json);
        determine_spec_version(source, &metadata)
      }
      _ => spec_version.clone()
    };
    let results = match spec_version {
      PactSpecification::V4 => V4Pact::verify_json("/", pact_json, args.is_present("strict")),
      _ => match pact_json {
        Value::Object(map) => if map.contains_key("messages") {
          MessagePact::verify_json("/", pact_json, args.is_present("strict"))
        } else {
          RequestResponsePact::verify_json("/", pact_json, args.is_present("strict"))
        },
        _ => vec![PactFileVerificationResult::new("/", ResultLevel::ERROR,
          &format!("Must be an Object, got {}", json_type_of(pact_json)))]
      }
    };
    VerificationResult::new(source, results)
  }).collect();

  let display_result = display_results(&results, args.value_of("output").unwrap_or("console"));

  if display_result.is_err() {
    Err(3)
  } else if results.iter().any(|res| res.has_errors()) {
    Err(2)
  } else {
    Ok(())
  }
}

fn load_files(args: &ArgMatches) -> anyhow::Result<Vec<(String, Value)>> {
  let mut sources: Vec<(String, anyhow::Result<Value>)> = vec![];
  if let Some(values) = args.values_of("file") {
    sources.extend(values.map(|v| {
      (v.to_string(), load_file(v))
    }).collect::<Vec<(String, anyhow::Result<Value>)>>());
  };
  if let Some(values) = args.values_of("url") {
    sources.extend(values.map(|v| {
      (v.to_string(), fetch_pact(v, args).map(|(_, value)| value))
    }).collect::<Vec<(String, anyhow::Result<Value>)>>());
  };

  if sources.iter().any(|(_, res)| res.is_err()) {
    error!("Failed to load the following pact files:");
    for (source, result) in sources.iter().filter(|(_, res)| res.is_err()) {
      error!("    '{}' - {}", source, result.as_ref().unwrap_err());
    }
    Err(anyhow!("Failed to load one or more pact files"))
  } else {
    Ok(sources.iter().map(|(source, result)| (source.clone(), result.as_ref().unwrap().clone())).collect())
  }
}

fn fetch_pact(url: &str, args: &ArgMatches) -> anyhow::Result<(String, Value)> {
  let auth = if args.is_present("user") {
    args.value_of("password").map(|user | {
      HttpAuth::User(user.to_string(), args.value_of("password").map(|p| p.to_string()))
    })
  } else if args.is_present("token") {
    args.value_of("token").map(|token| HttpAuth::Token(token.to_string()))
  } else {
    None
  };
  http_utils::fetch_json_from_url(&url.to_string(), &auth)
}

fn load_file(file_name: &str) -> anyhow::Result<Value> {
  let file = File::open(file_name)?;
  serde_json::from_reader(file).context("file is not JSON")
}

fn main() {
  match handle_cli() {
    Ok(_) => (),
    Err(err) => std::process::exit(err)
  }
}
