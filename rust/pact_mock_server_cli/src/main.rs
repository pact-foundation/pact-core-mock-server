//! The `pact_mock_server` crate provides the CLI for the pact mock server for mocking HTTP requests
//! and generating responses based on a pact file. It implements the V3 Pact specification
//! (https://github.com/pact-foundation/pact-specification/tree/version-3).

#![warn(missing_docs)]

use std::cell::RefCell;
use std::env;
use std::io;
use std::str::FromStr;
use std::sync::Mutex;

use anyhow::anyhow;
use clap::{Arg, ArgAction, command, Command, ErrorKind};
use lazy_static::*;
use pact_models::PactSpecification;
use rand::distributions::Alphanumeric;
use rand::Rng;
use tracing_core::LevelFilter;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

use pact_mock_server::server_manager::ServerManager;

pub(crate) fn display_error(error: String, usage: &str) -> ! {
    eprintln!("ERROR: {}", error);
    eprintln!();
    eprintln!("{}", usage);
    panic!("{}", error)
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

fn print_version() {
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
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = FmtSubscriber::builder()
      .with_max_level(log_level)
      .with_writer(non_blocking.and(io::stdout))
      .with_thread_names(true)
      .with_ansi(!no_term_log)
      .finish();
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

fn uuid_value(v: &str) -> Result<Uuid, String> {
  Uuid::parse_str(v).map_err(|e| format!("'{}' is not a valid UUID value: {}", v, e) )
}

#[tokio::main]
async fn main() {
  match handle_command_args().await {
    Ok(_) => (),
    Err(err) => std::process::exit(err)
  }
}

#[derive(Debug, Clone)]
pub(crate) struct ServerOpts {
  pub output_path: Option<String>,
  pub base_port: Option<u16>,
  pub server_key: String
}

lazy_static!{
  pub(crate) static ref SERVER_OPTIONS: Mutex<RefCell<ServerOpts>> = Mutex::new(RefCell::new(ServerOpts {
    output_path: None,
    base_port: None,
    server_key: String::default()
  }));
  pub(crate) static ref SERVER_MANAGER: Mutex<ServerManager> = Mutex::new(ServerManager::new());
}

async fn handle_command_args() -> Result<(), i32> {
  let mut app = setup_args();

  let matches = app.clone().try_get_matches();
  match matches {
    Ok(ref matches) => {
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
      let port_8080 = "8080".to_string();
      let port = matches.get_one::<String>("port").unwrap_or(&port_8080);
      let localhost = "localhost".to_string();
      let host = matches.get_one::<String>("host").unwrap_or(&localhost);
      match port.parse::<u16>() {
        Ok(p) => {
          match matches.subcommand() {
            Some(("start", sub_matches)) => {
              let output_path = sub_matches.get_one::<String>("output").map(|s| s.to_owned());
              let base_port = sub_matches.get_one::<u16>("base-port").cloned();
              let server_key = sub_matches.get_one::<String>("server-key").map(|s| s.to_owned())
                .unwrap_or_else(|| rand::thread_rng().sample_iter(Alphanumeric).take(16).map(char::from).collect::<String>());
              {
                let inner = (*SERVER_OPTIONS).lock().unwrap();
                let mut options = inner.deref().borrow_mut();
                options.output_path = output_path;
                options.base_port = base_port;
                options.server_key = server_key;
              }
              server::start_server(p).await
            },
            Some(("list", _)) => list::list_mock_servers(host, p, &mut app).await,
            Some(("create", sub_matches)) => create_mock::create_mock_server(host, p, sub_matches, &mut app).await,
            Some(("verify", sub_matches)) => verify::verify_mock_server(host, p, sub_matches, &mut app).await,
            Some(("shutdown", sub_matches)) => shutdown::shutdown_mock_server(host, p, sub_matches, &mut app).await,
            Some(("shutdown-master", sub_matches)) => shutdown::shutdown_master_server(host, p, sub_matches, &mut app).await,
            _ => Err(3)
          }
        },
        Err(_) => display_error(format!("{} is not a valid port number", port), app.render_usage().as_str())
      }
    },
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

fn setup_args() -> Command<'static> {
  command!()
    .about("Standalone Pact mock server")
    .arg_required_else_help(true)
    .subcommand_required(true)
    .propagate_version(true)
    .mut_arg("version", |arg| arg.short('v'))
    .arg(Arg::new("port")
      .short('p')
      .long("port")
      .global(true)
      .action(ArgAction::Set)
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
      .about("Lists all the running mock servers"))
    .subcommand(Command::new("create")
      .about("Creates a new mock server from a pact file")
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
      .arg(Arg::new("tls")
        .long("tls")
        .action(ArgAction::SetTrue)
        .help("Enable TLS with the mock server (will use a self-signed certificate)"))
      )
    .subcommand(Command::new("verify")
      .about("Verify the mock server by id or port number, and generate a pact file if all ok")
      .arg(Arg::new("mock-server-id")
        .short('i')
        .long("mock-server-id")
        .action(ArgAction::Set)
        .required_unless_present("mock-server-port")
        .conflicts_with("mock-server-port")
        .help("the ID of the mock server")
        .value_parser(uuid_value))
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
      .arg(Arg::new("mock-server-id")
        .short('i')
        .long("mock-server-id")
        .action(ArgAction::Set)
        .required_unless_present("mock-server-port")
        .conflicts_with("mock-server-port")
        .help("the ID of the mock server")
        .value_parser(uuid_value))
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
    setup_args() /*.debug_assert()*/;
  }
}
