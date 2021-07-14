//! Pact file format validator
//!
//! Validator for Pact files.

#![warn(missing_docs)]

use std::{env, fs};
use std::fs::File;

use anyhow::anyhow;
use clap::{App, AppSettings, Arg, ArgMatches, ErrorKind};
use glob::glob;
use log::*;
use serde_json::Value;

use pact_cli::{glob_value, setup_loggers};
use pact_cli::verification::{display_results, VerificationResult, verify_json};
use pact_models::http_utils::{self, HttpAuth};
use pact_models::PactSpecification;

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
      .required_unless_one(&["url", "dir", "glob"])
      .takes_value(true)
      .use_delimiter(false)
      .multiple(true)
      .number_of_values(1)
      .empty_values(false)
      .help("Pact file to verify (can be repeated)"))
    .arg(Arg::with_name("url")
      .short("u")
      .long("url")
      .required_unless_one(&["file", "dir", "glob"])
      .takes_value(true)
      .use_delimiter(false)
      .multiple(true)
      .number_of_values(1)
      .empty_values(false)
      .help("URL of pact file to verify (can be repeated)"))
    .arg(Arg::with_name("dir")
      .short("d")
      .long("dir")
      .required_unless_one(&["file", "url", "glob"])
      .takes_value(true)
      .use_delimiter(false)
      .multiple(true)
      .number_of_values(1)
      .empty_values(false)
      .help("Directory of pact files to verify (can be repeated)"))
    .arg(Arg::with_name("glob")
      .short("g")
      .long("glob")
      .required_unless_one(&["file", "url", "dir"])
      .takes_value(true)
      .use_delimiter(false)
      .multiple(true)
      .number_of_values(1)
      .empty_values(false)
      .validator(glob_value)
      .help("Glob pattern to match pact files to verify (can be repeated)")
      .long_help("
      Glob pattern to match pact files to verify

      ?      matches any single character.
      *      matches any (possibly empty) sequence of characters.
      **     matches the current directory and arbitrary subdirectories. This sequence must form
             a single path component, so both **a and b** are invalid and will result in an
             error. A sequence of more than two consecutive * characters is also invalid.
      [...]  matches any character inside the brackets. Character sequences can also specify
             ranges of characters, as ordered by Unicode, so e.g. [0-9] specifies any character
             between 0 and 9 inclusive. An unclosed bracket is invalid.
      [!...] is the negation of [...], i.e. it matches any characters not in the brackets.

      The metacharacters ?, *, [, ] can be matched by using brackets (e.g. [?]). When a ]
      occurs immediately following [ or [! then it is interpreted as being part of, rather
      then ending, the character set, so ] and NOT ] can be matched by []] and [!]] respectively.
      The - character can be specified inside a character sequence pattern by placing it at
      the start or the end, e.g. [abc-].

      See https://docs.rs/glob/0.3.0/glob/struct.Pattern.html"))
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
    let results = verify_json(pact_json, spec_version, source, args.is_present("strict"));
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
  if let Some(values) = args.values_of("dir") {
    for value in values {
      for entry in fs::read_dir(value)? {
        let path = entry?.path();
        if path.is_file() && path.extension().unwrap_or_default() == "json" {
          let file_name = path.to_str().ok_or(anyhow!("Directory contains non-UTF-8 entry"))?;
          sources.push((file_name.to_string(), load_file(file_name)));
        }
      }
    }
  };
  if let Some(values) = args.values_of("glob") {
    for value in values {
      for entry in glob(value)? {
        let entry = entry?;
        let file_name = entry.to_str().ok_or(anyhow!("Glob matched non-UTF-8 entry"))?;
        sources.push((file_name.to_string(), load_file(file_name)));
      }
    }
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
  serde_json::from_reader(file)
    .map_err(|err| anyhow!("Failed to parse file as JSON - {}", err))
}

fn main() {
  match handle_cli() {
    Ok(_) => (),
    Err(err) => std::process::exit(err)
  }
}
