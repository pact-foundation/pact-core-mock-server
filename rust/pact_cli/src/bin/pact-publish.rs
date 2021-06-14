//! CLI to publish Pact files to a Pact broker.

#![warn(missing_docs)]

use std::env;
use std::fs::File;

use anyhow::{anyhow, Context};
use clap::{App, AppSettings, Arg, ArgMatches, ErrorKind};
use log::*;
use serde_json::Value;

use pact_cli::{glob_value, setup_loggers};
use pact_matching::models::http_utils;
use pact_matching::models::http_utils::HttpAuth;

fn setup_app<'a, 'b>(program: &str, version: &'b str) -> App<'a, 'b> {
  App::new(program)
    .version(version)
    .about("Pact file publisher")
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
      .required_unless_one(&["dir", "glob"])
      .takes_value(true)
      .use_delimiter(false)
      .multiple(true)
      .number_of_values(1)
      .empty_values(false)
      .help("Pact file to publish (can be repeated)"))
    .arg(Arg::with_name("dir")
      .short("d")
      .long("dir")
      .required_unless_one(&["file", "glob"])
      .takes_value(true)
      .use_delimiter(false)
      .multiple(true)
      .number_of_values(1)
      .empty_values(false)
      .help("Directory of pact files to publish (can be repeated)"))
    .arg(Arg::with_name("glob")
      .short("g")
      .long("glob")
      .required_unless_one(&["file", "dir"])
      .takes_value(true)
      .use_delimiter(false)
      .multiple(true)
      .number_of_values(1)
      .empty_values(false)
      .validator(glob_value)
      .help("Glob pattern to match pact files to publish (can be repeated)")
      .long_help("
      Glob pattern to match pact files to publish

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
    .arg(Arg::with_name("validate")
      .long("validate")
      .short("v")
      .help("Validate the Pact files before publishing."))
    .arg(Arg::with_name("user")
      .long("user")
      .takes_value(true)
      .use_delimiter(false)
      .number_of_values(1)
      .empty_values(false)
      .conflicts_with("token")
      .help("Username to use to publish with"))
    .arg(Arg::with_name("password")
      .long("password")
      .takes_value(true)
      .use_delimiter(false)
      .number_of_values(1)
      .empty_values(false)
      .conflicts_with("token")
      .help("Password to use to publish with"))
    .arg(Arg::with_name("token")
      .short("t")
      .long("token")
      .takes_value(true)
      .use_delimiter(false)
      .number_of_values(1)
      .empty_values(false)
      .conflicts_with("user")
      .help("Bearer token to use to publish with"))
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

  let files = load_files(args).map_err(|_| 1)?;

  // let results = files.iter().map(|(source, pact_json)| {
  //   let results = match spec_version {
  //     PactSpecification::V4 => V4Pact::verify_json("/", pact_json, args.is_present("strict")),
  //     _ => match pact_json {
  //       Value::Object(map) => if map.contains_key("messages") {
  //         MessagePact::verify_json("/", pact_json, args.is_present("strict"))
  //       } else {
  //         RequestResponsePact::verify_json("/", pact_json, args.is_present("strict"))
  //       },
  //       _ => vec![PactFileVerificationResult::new("/", ResultLevel::ERROR,
  //         &format!("Must be an Object, got {}", json_type_of(pact_json)))]
  //     }
  //   };
  //   VerificationResult::new(source, results)
  // }).collect();
  //
  // let display_result = display_results(&results, "console");
  //
  // if display_result.is_err() {
  //   Err(3)
  // } else if results.iter().any(|res| res.has_errors()) {
  //   Err(2)
  // } else {
  //   Ok(())
  // }
  Err(1)
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
