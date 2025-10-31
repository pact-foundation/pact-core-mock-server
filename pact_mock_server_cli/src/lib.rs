//! The `pact_mock_server` crate provides the CLI for the pact mock server for mocking HTTP requests
//! and generating responses based on a pact file. It implements the
//! [V3 Pact specification](https://github.com/pact-foundation/pact-specification/tree/version-3)
//! and [V4 Pact specification](https://github.com/pact-foundation/pact-specification/tree/version-4).

#![warn(missing_docs)]

use std::env;
use std::io;
use std::process::ExitCode;
use std::str::FromStr;
use std::sync::Mutex;

use anyhow::anyhow;
use clap::ArgMatches;
use clap::{Arg, ArgAction, command, Command};
use clap::error::ErrorKind;
use lazy_static::*;
use pact_models::PactSpecification;
use rand::distr::Alphanumeric;
use rand::Rng;
use regex::Regex;
use tracing_core::LevelFilter;
use tracing_subscriber::FmtSubscriber;

use pact_mock_server::server_manager::ServerManager;
use tracing_subscriber::layer::SubscriberExt;

pub(crate) fn display_error(error: String, _usage: &str, code: i32) -> ! {
  eprintln!("ERROR: {}\nExiting with status {}", error, code);
  std::process::exit(code)
}

pub(crate) fn handle_error(error: &str) -> i32 {
  eprintln!("ERROR: {}", error);
  eprintln!();
  -100
}

mod server;
mod create_mock;
mod list;
mod verify;
mod shutdown;

pub fn print_version() {
    println!("pact mock server version  : v{}", clap::crate_version!());
    println!("pact specification version: v{}", PactSpecification::V4.version_str());
}

fn setup_loggers(
  level: &str,
  command: &str,
  output: Option<&str>,
  no_file_log: bool,
  no_term_log: bool
) -> anyhow::Result<()> {
  let log_level = match level {
    "none" => LevelFilter::OFF,
    _ => LevelFilter::from_str(level).unwrap()
  };

  if command == "start" && !no_file_log {
    let file_appender = tracing_appender::rolling::daily(output.unwrap_or("."), "pact_mock_server.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    static mut LOG_GUARD: Option<tracing_appender::non_blocking::WorkerGuard> = None;
    unsafe {
      LOG_GUARD = Some(guard);
    }
    let file_layer = tracing_subscriber::fmt::layer()
      .with_writer(non_blocking)
      .with_ansi(false)
      .with_thread_names(true);

    let subscriber = 
    FmtSubscriber::builder().with_max_level(log_level).with_ansi(!no_term_log).finish()
      .with(file_layer);

    tracing::subscriber::set_global_default(subscriber)
  } else {
    let subscriber = FmtSubscriber::builder()
      .with_max_level(log_level)
      .with_thread_names(true)
      .with_ansi(!no_term_log)
      .finish();
    tracing::subscriber::set_global_default(subscriber)
  }.map_err(|err| anyhow!(err))
}

fn integer_value(v: &str) -> Result<u16, String> {
  v.parse::<u16>().map_err(|e| format!("'{}' is not a valid port value: {}", v, e) )
}

fn mock_server_id(v: &str) -> Result<String, String> {
  if v.is_empty() {
    Err("Server ID can not be empty".to_string())
  } else {
    let re = Regex::new(r"[A-Z0-9]{8}").unwrap();
    if re.is_match(v) {
      Ok(v.to_string())
    } else {
      Err(format!("'{}' is not a valid server ID", v))
    }
  }
}

#[derive(Debug, Clone)]
pub(crate) struct ServerOpts {
  pub output_path: Option<String>,
  pub base_port: Option<u16>,
  pub server_key: String
}

lazy_static!{
  pub(crate) static ref SERVER_MANAGER: Mutex<ServerManager> = Mutex::new(ServerManager::new());
}

pub async fn handle_command_args() -> Result<(), i32> {
  let mut app = setup_args();

  let matches = app.try_get_matches();
  match matches {
    Ok(results) => handle_matches(&results).await,

    Err(ref err) => {
      match err.kind() {
        ErrorKind::DisplayHelp => {
          println!("{}", err);
          Ok(())
        },
        ErrorKind::DisplayVersion => {
          print_version();
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

pub fn process_mock_command(args: &ArgMatches) -> Result<(), ExitCode>  {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let res = handle_matches(args).await;
        match res {
            Ok(()) => Ok(()),
            Err(code) => Err(ExitCode::from(code as u8)),
        }
    })
}



async fn handle_matches(matches: &ArgMatches) -> Result<(), i32> {
      let log_level = matches.get_one::<String>("loglevel").map(|lvl| lvl.as_str());
      let no_file_log = matches.get_flag("no-file-log");
      let no_term_log = matches.get_flag("no-term-log");
      if let Err(err) = setup_loggers(log_level.unwrap_or("info"),
        matches.subcommand_name().unwrap(),
        matches.subcommand().map(|(name, args)| {
          if name == "start" {
            args.get_one::<String>("output").map(|o| o.as_str())
          } else {
            None
          }
        }).flatten(),
        no_file_log,
        no_term_log
      ) {
        eprintln!("WARN: Could not setup loggers: {}", err);
        eprintln!();
      }

      let port = *matches.get_one::<u16>("port").unwrap_or(&8080);
      let localhost = "localhost".to_string();
      let host = matches.get_one::<String>("host").unwrap_or(&localhost);

      let usage = setup_args().render_usage().to_string();

      match matches.subcommand() {
        Some(("start", sub_matches)) => {
          let output_path = sub_matches.get_one::<String>("output").map(|s| s.to_owned());
          let base_port = sub_matches.get_one::<u16>("base-port").cloned();
          let server_key = sub_matches.get_one::<String>("server-key").map(|s| s.to_owned())
            .unwrap_or_else(|| rand::thread_rng().sample_iter(Alphanumeric).take(16).map(char::from).collect::<String>());
          let options = ServerOpts {
            output_path,
            base_port,
            server_key,
          };
          server::start_server(port, options).await
        },
        Some(("list", _)) => list::list_mock_servers(host, port, usage.as_str()).await,
        Some(("create", sub_matches)) => create_mock::create_mock_server(host, port, sub_matches, usage.as_str()).await,
        Some(("verify", sub_matches)) => verify::verify_mock_server(host, port, sub_matches, usage.as_str()).await,
        Some(("shutdown", sub_matches)) => shutdown::shutdown_mock_server(host, port, sub_matches, usage.as_str()).await,
        Some(("shutdown-master", sub_matches)) => shutdown::shutdown_master_server(host, port, sub_matches, usage.as_str()).await,
        _ => Err(3)
      }
}
pub fn setup_args() -> Command {
  #[allow(unused_mut)]
  let mut create_command = Command::new("create")
    .about("Creates a new mock server from a pact file")
    .version(clap::crate_version!())
    .arg(Arg::new("file")
      .short('f')
      .long("file")
      .action(ArgAction::Set)
      .required(true)
      .help("the pact file to define the mock server"))
    .arg(Arg::new("cors")
      .short('c')
      .long("cors-preflight")
      .action(ArgAction::SetTrue)
      .help("Handle CORS pre-flight requests"))
    .arg(Arg::new("specification")
      .long("specification")
      .action(ArgAction::Set)
      .num_args(1)
      .help("The Pact specification version to use (defaults to V4)"));

  #[cfg(feature = "tls")]
  {
    create_command = create_command.arg(Arg::new("tls")
     .long("tls")
     .action(ArgAction::SetTrue)
     .help("Enable TLS with the mock server (will use a self-signed certificate)"));
  }

  command!()
    .about("Standalone Pact mock server")
    .disable_help_flag(true)
    .arg_required_else_help(true)
    .disable_version_flag(true)
    .arg(Arg::new("help")
      .long("help")
      .action(ArgAction::Help)
      .global(true)
      .help("Print help and exit"))
    .arg(Arg::new("version")
      .short('v')
      .long("version")
      .action(ArgAction::Version)
      .global(true)
      .help("Print version information and exit"))
    .arg(Arg::new("port")
      .short('p')
      .long("port")
      .global(true)
      .action(ArgAction::Set)
      .value_parser(integer_value)
      .help("port the master mock server runs on (defaults to 8080)"))
    .arg(Arg::new("host")
      .short('h')
      .long("host")
      .global(true)
      .action(ArgAction::Set)
      .help("hostname the master mock server runs on (defaults to localhost)"))
    .arg(Arg::new("loglevel")
      .short('l')
      .long("loglevel")
      .global(true)
      .action(ArgAction::Set)
      .value_parser(["error", "warn", "info", "debug", "trace", "none"])
      .help("Log level for mock servers to write to the log file (defaults to info)"))
    .arg(Arg::new("no-term-log")
      .long("no-term-log")
      .global(true)
      .action(ArgAction::SetTrue)
      .help("Turns off using terminal ANSI escape codes"))
    .arg(Arg::new("no-file-log")
      .long("no-file-log")
      .global(true)
      .action(ArgAction::SetTrue)
      .help("Do not log to an output file"))
    .subcommand(Command::new("start")
      .about("Starts the master mock server")
      .version(clap::crate_version!())
      .arg(Arg::new("output")
        .short('o')
        .long("output")
        .action(ArgAction::Set)
        .help("the directory where to write files to (defaults to current directory)"))
      .arg(Arg::new("base-port")
        .long("base-port")
        .action(ArgAction::Set)
        .help("the base port number that mock server ports will be allocated from. If not specified, ports will be randomly assigned by the OS.")
        .value_parser(integer_value))
      .arg(Arg::new("server-key")
        .long("server-key")
        .action(ArgAction::Set)
        .help("the server key to use to authenticate shutdown requests (defaults to a random generated one)"))
      )
    .subcommand(Command::new("list")
      .about("Lists all the running mock servers")
      .version(clap::crate_version!()))
    .subcommand(create_command)
    .subcommand(Command::new("verify")
      .about("Verify the mock server by id or port number, and generate a pact file if all ok")
      .version(clap::crate_version!())
      .arg(Arg::new("mock-server-id")
        .short('i')
        .long("mock-server-id")
        .action(ArgAction::Set)
        .required_unless_present("mock-server-port")
        .conflicts_with("mock-server-port")
        .help("the ID of the mock server")
        .value_parser(mock_server_id))
      .arg(Arg::new("mock-server-port")
        .short('m')
        .long("mock-server-port")
        .action(ArgAction::Set)
        .required_unless_present("mock-server-id")
        .help("the port number of the mock server")
        .value_parser(integer_value))
      )
    .subcommand(Command::new("shutdown")
      .about("Shutdown the mock server by id or port number, releasing all its resources")
      .version(clap::crate_version!())
      .arg(Arg::new("mock-server-id")
        .short('i')
        .long("mock-server-id")
        .action(ArgAction::Set)
        .required_unless_present("mock-server-port")
        .conflicts_with("mock-server-port")
        .help("the ID of the mock server")
        .value_parser(mock_server_id))
      .arg(Arg::new("mock-server-port")
        .short('m')
        .long("mock-server-port")
        .action(ArgAction::Set)
        .required_unless_present("mock-server-id")
        .help("the port number of the mock server")
        .value_parser(integer_value))
      )
    .subcommand(Command::new("shutdown-master")
      .about("Performs a graceful shutdown of the master server (displayed when it started)")
      .version(clap::crate_version!())
      .arg(Arg::new("server-key")
        .short('k')
        .long("server-key")
        .action(ArgAction::Set)
        .required(true)
        .help("the server key of the master server"))
      .arg(Arg::new("period")
        .long("period")
        .action(ArgAction::Set)
        .help("the period of time in milliseconds to allow the server to shutdown (defaults to 100ms)")
        .value_parser(integer_value))
      )
}

#[cfg(test)]
mod test {
  use expectest::expect;
  use expectest::prelude::*;

  use crate::{integer_value, setup_args};

  #[test]
  fn validates_integer_value() {
      expect!(integer_value("1234")).to(be_ok());
      expect!(integer_value("1234x")).to(be_err());
  }

  #[test_log::test]
  fn verify_cli() {
    setup_args().debug_assert();
  }
}
