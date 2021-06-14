//! Pact file verification and schemas
use std::str::FromStr;

use log::{LevelFilter, SetLoggerError};
use simplelog::{ColorChoice, Config, TerminalMode, TermLogger};

pub mod verification;

pub fn setup_loggers(level: &str) -> Result<(), SetLoggerError> {
  let log_level = match level {
    "none" => LevelFilter::Off,
    _ => LevelFilter::from_str(level).unwrap()
  };
  TermLogger::init(log_level, Config::default(), TerminalMode::Stderr, ColorChoice::Auto)
}

pub fn glob_value(v: String) -> Result<(), String> {
  match glob::Pattern::new(&v) {
    Ok(_) => Ok(()),
    Err(err) => Err(format!("'{}' is not a valid glob pattern - {}", v, err))
  }
}
