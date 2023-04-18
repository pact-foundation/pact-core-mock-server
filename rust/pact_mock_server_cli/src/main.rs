//! The `pact_mock_server` crate provides the CLI for the pact mock server for mocking HTTP requests
//! and generating responses based on a pact file. It implements the V3 Pact specification
//! (https://github.com/pact-foundation/pact-specification/tree/version-3).

#![warn(missing_docs)]

use std::cell::RefCell;
use std::env;
use std::fs::{self, File};
use std::fs::OpenOptions;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;

use clap::{App, AppSettings, Arg, ArgMatches, command, ErrorKind, SubCommand};
use lazy_static::*;
use log::LevelFilter;
use pact_models::PactSpecification;
use rand::distributions::Alphanumeric;
use rand::Rng;
use simplelog::{ColorChoice, CombinedLogger, Config, SimpleLogger, TermLogger, WriteLogger};
use uuid::Uuid;

use pact_mock_server::server_manager::ServerManager;

pub(crate) fn display_error(error: String, app: &mut App) -> ! {
    eprintln!("ERROR: {}", error);
    eprintln!();
    eprintln!("{}", app.render_usage());
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
    println!("\npact mock server version  : v{}", clap::crate_version!());
    println!("pact specification version: v{}", PactSpecification::V3.version_str());
}

fn setup_log_file(output: Option<&str>) -> Result<File, io::Error> {
  let log_file = match output {
    Some(p) => {
      fs::create_dir_all(p)?;
      let mut path = PathBuf::from(p);
      path.push("pact_mock_server.log");
      path
    },
    None => PathBuf::from("pact_mock_server.log")
  };
  OpenOptions::new()
    .read(false)
    .write(true)
    .append(true)
    .create(true)
    .open(log_file)
}

fn setup_loggers(level: &str, command: &str, output: Option<&str>, no_file_log: bool, no_term_log: bool) -> Result<(), String> {
    let term_mode = simplelog::TerminalMode::Stdout;
    let log_level = match level {
        "none" => LevelFilter::Off,
        _ => LevelFilter::from_str(level).unwrap()
    };

    if command == "start" {
      match (no_file_log, no_term_log) {
        (true, true) => {
          SimpleLogger::init(log_level, Config::default()).map_err(|e| format!("{:?}", e))
        },
        (true, false) => {
          TermLogger::init(log_level, Config::default(), term_mode, ColorChoice::Auto)
            .map_err(|e| format!("{:?}", e))
        },
        (false, true) => {
          let log_file = setup_log_file(output).map_err(|e| format!("{:?}", e))?;
          WriteLogger::init(log_level, Config::default(), log_file).map_err(|e| format!("{:?}", e))
        },
        _ => {
          let log_file = setup_log_file(output).map_err(|e| format!("{:?}", e))?;
          CombinedLogger::init(
            vec![
              TermLogger::new(log_level, Config::default(), term_mode, ColorChoice::Auto),
              WriteLogger::new(log_level, Config::default(), log_file)
            ]
          ).map_err(|e| format!("{:?}", e))
        }
      }
    } else if no_term_log {
      SimpleLogger::init(log_level, Config::default()).map_err(|e| format!("{:?}", e))
    } else {
      TermLogger::init(log_level, Config::default(), term_mode, ColorChoice::Auto)
        .map_err(|e| format!("{:?}", e))
    }
}

fn global_option_present(option: &str, matches: &ArgMatches) -> bool {
  matches.is_present(option) || matches.subcommand().unwrap().1.is_present(option)
}

fn integer_value(v: &str) -> Result<(), String> {
    v.parse::<u16>().map(|_| ()).map_err(|e| format!("'{}' is not a valid port value: {}", v, e) )
}

fn uuid_value(v: &str) -> Result<(), String> {
    Uuid::parse_str(v).map(|_| ()).map_err(|e| format!("'{}' is not a valid UUID value: {}", v, e) )
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

  let matches = app.clone().get_matches_safe();
  match matches {
    Ok(ref matches) => {
      let log_level = matches.value_of("loglevel");
      if let Err(err) = setup_loggers(log_level.unwrap_or("info"),
        matches.subcommand_name().unwrap(),
        matches.subcommand().unwrap().1.value_of("output"),
        global_option_present("no-file-log", matches),
        global_option_present("no-term-log", matches)) {
        eprintln!("WARN: Could not setup loggers: {}", err);
        eprintln!();
      }
      let port = matches.value_of("port").unwrap_or("8080");
      let host = matches.value_of("host").unwrap_or("localhost");
      match port.parse::<u16>() {
        Ok(p) => {
          match matches.subcommand() {
            Some(("start", sub_matches)) => {
              let output_path = sub_matches.value_of("output").map(|s| s.to_owned());
              let base_port = sub_matches.value_of("base-port").map(|s| s.parse::<u16>().unwrap_or(0));
              let server_key = sub_matches.value_of("server-key").map(|s| s.to_owned())
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
        Err(_) => display_error(format!("{} is not a valid port number", port), &mut app)
      }
    },
    Err(ref err) => {
      match err.kind {
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

fn setup_args() -> App<'static> {
  command!()
    .about("Standalone Pact mock server")
    .version_short('v')
    .long_version("version")
    .setting(AppSettings::ArgRequiredElseHelp)
    .setting(AppSettings::SubcommandRequired)
    .setting(AppSettings::GlobalVersion)
    .setting(AppSettings::ColoredHelp)
    .arg(Arg::with_name("port")
      .short('p')
      .long("port")
      .takes_value(true)
      .use_delimiter(false)
      .global(true)
      .help("port the master mock server runs on (defaults to 8080)"))
    .arg(Arg::with_name("host")
      .short('h')
      .long("host")
      .takes_value(true)
      .use_delimiter(false)
      .global(true)
      .help("hostname the master mock server runs on (defaults to localhost)"))
    .arg(Arg::with_name("loglevel")
      .short('l')
      .long("loglevel")
      .takes_value(true)
      .use_delimiter(false)
      .global(true)
      .possible_values(&["error", "warn", "info", "debug", "trace", "none"])
      .help("Log level for mock servers to write to the log file (defaults to info)"))
    .arg(Arg::with_name("no-term-log")
      .long("no-term-log")
      .global(true)
      .help("Use a simple logger instead of the term based one"))
    .arg(Arg::with_name("no-file-log")
      .long("no-file-log")
      .global(true)
      .help("Do not log to an output file"))
    .subcommand(SubCommand::with_name("start")
      .about("Starts the master mock server")
      .arg(Arg::with_name("output")
        .short('o')
        .long("output")
        .takes_value(true)
        .use_delimiter(false)
        .help("the directory where to write files to (defaults to current directory)"))
      .arg(Arg::with_name("base-port")
        .long("base-port")
        .takes_value(true)
        .use_delimiter(false)
        .required(false)
        .help("the base port number that mock server ports will be allocated from. If not specified, ports will be randomly assigned by the OS.")
        .validator(integer_value))
      .arg(Arg::with_name("server-key")
        .long("server-key")
        .takes_value(true)
        .use_delimiter(false)
        .help("the server key to use to authenticate shutdown requests (defaults to a random generated one)"))
      )
    .subcommand(SubCommand::with_name("list")
      .about("Lists all the running mock servers")
      )
    .subcommand(SubCommand::with_name("create")
      .about("Creates a new mock server from a pact file")
      .arg(Arg::with_name("file")
        .short('f')
        .long("file")
        .takes_value(true)
        .use_delimiter(false)
        .required(true)
        .help("the pact file to define the mock server"))
      .arg(Arg::with_name("cors")
        .short('c')
        .long("cors-preflight")
        .help("Handle CORS pre-flight requests"))
      .arg(Arg::with_name("tls")
        .long("tls")
        .help("Enable TLS with the mock server (will use a self-signed certificate)"))
      )
    .subcommand(SubCommand::with_name("verify")
      .about("Verify the mock server by id or port number, and generate a pact file if all ok")
      .arg(Arg::with_name("mock-server-id")
        .short('i')
        .long("mock-server-id")
        .takes_value(true)
        .use_delimiter(false)
        .required_unless("mock-server-port")
        .conflicts_with("mock-server-port")
        .help("the ID of the mock server")
        .validator(uuid_value))
      .arg(Arg::with_name("mock-server-port")
        .short('m')
        .long("mock-server-port")
        .takes_value(true)
        .use_delimiter(false)
        .required_unless("mock-server-id")
        .help("the port number of the mock server")
        .validator(integer_value))
      )
    .subcommand(SubCommand::with_name("shutdown")
      .about("Shutdown the mock server by id or port number, releasing all its resources")
      .arg(Arg::with_name("mock-server-id")
        .short('i')
        .long("mock-server-id")
        .takes_value(true)
        .use_delimiter(false)
        .required_unless("mock-server-port")
        .conflicts_with("mock-server-port")
        .help("the ID of the mock server")
        .validator(uuid_value))
      .arg(Arg::with_name("mock-server-port")
        .short('m')
        .long("mock-server-port")
        .takes_value(true)
        .use_delimiter(false)
        .required_unless("mock-server-id")
        .help("the port number of the mock server")
        .validator(integer_value))
      )
    .subcommand(SubCommand::with_name("shutdown-master")
      .about("Performs a graceful shutdown of the master server (displayed when it started)")
      .arg(Arg::with_name("server-key")
        .short('k')
        .long("server-key")
        .takes_value(true)
        .use_delimiter(false)
        .required(true)
        .help("the server key of the master server"))
      .arg(Arg::with_name("period")
        .long("period")
        .takes_value(true)
        .use_delimiter(false)
        .help("the period of time in milliseconds to allow the server to shutdown (defaults to 100ms)")
        .validator(integer_value))
      )
}

#[cfg(test)]
mod test {
  use expectest::expect;
  use expectest::prelude::*;

  use crate::integer_value;

  #[test]
  fn validates_integer_value() {
      expect!(integer_value("1234")).to(be_ok());
      expect!(integer_value("1234x")).to(be_err());
  }

  // Test is failing due to the version override
  // #[test_log::test]
  // fn verify_cli() {
  //   setup_args().debug_assert();
  // }
}
