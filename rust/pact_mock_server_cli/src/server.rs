use hyper::server::{Handler, Server, Request, Response};
use pact_matching::models::Pact;
use pact_matching::s;
use pact_mock_server::server_manager::ServerManager;
use uuid::Uuid;
use serde_json::{self, Value, json};
use std::{
  sync::{Arc, Mutex},
  thread,
  time::Duration,
  iter::FromIterator,
  ops::Deref,
  net::TcpListener,
  process
};
use crate::verify;
use webmachine_rust::*;
use webmachine_rust::context::*;
use webmachine_rust::headers::*;
use clap::ArgMatches;
use rand::{self, Rng};
use maplit::*;

fn json_error(error: String) -> String {
    let json_response = json!({ s!("error") : json!(error) });
    json_response.to_string()
}

fn get_next_port(base_port: Option<u16>) -> u16 {
  match base_port {
    None => 0,
    Some(p) => if p > 0 {
      let mut port = p;
      let mut listener = TcpListener::bind(("127.0.0.1", port));
      while listener.is_err() && port < p + 1000 {
        port += 1;
        listener = TcpListener::bind(("127.0.0.1", port));
      }
      match listener {
        Ok(listener) => listener.local_addr().unwrap().port(),
        Err(_) => 0
      }
    } else {
      0
    }
  }
}

fn start_provider(
    context: &mut WebmachineContext,
    base_port: Option<u16>,
    server_manager: Arc<Mutex<ServerManager>>
) -> Result<bool, u16> {
    match context.request.body {
        Some(ref body) if !body.is_empty() => {
            match serde_json::from_str(body) {
                Ok(ref json) => {
                    let pact = Pact::from_json(&context.request.request_path, json);
                    let mock_server_id = Uuid::new_v4().to_string();

                    let mut lock = server_manager.lock().unwrap();
                    match lock.start_mock_server(mock_server_id.clone(), pact, get_next_port(base_port)) {
                        Ok(mock_server) => {
                            let mock_server_json = json!({
                                s!("id") : json!(mock_server_id.clone()),
                                s!("port") : json!(mock_server as i64),
                            });
                            let json_response = json!({ s!("mockServer") : mock_server_json });
                            context.response.body = Some(json_response.to_string());
                            context.response.add_header(s!("Location"),
                                vec![HeaderValue::basic(&format!("/mockserver/{}", mock_server_id))]);
                            Ok(true)
                        },
                        Err(msg) => {
                            context.response.body = Some(json_error(format!("Failed to start mock server - {}", msg)));
                            Err(422)
                        }
                    }
                },
                Err(err) => {
                    log::error!("Failed to parse json body - {}", err);
                    context.response.body = Some(json_error(format!("Failed to parse json body - {}", err)));
                    Err(422)
                }
            }
        },
        _ => {
            log::error!("No pact json was supplied");
            context.response.body = Some(json_error(s!("No pact json was supplied")));
            Err(422)
        }
    }
}

pub fn verify_mock_server_request(
    context: &mut WebmachineContext,
    output_path: &Option<String>,
    server_manager: Arc<Mutex<ServerManager>>
) -> Result<bool, u16> {
    let id = context.metadata.get(&s!("id")).unwrap_or(&s!("")).clone();
    match verify::validate_id(&id, server_manager) {
        Ok(ms) => {
            let mut map = btreemap!{ s!("mockServer") => ms.to_json() };
            let mismatches = ms.mismatches();
            if !mismatches.is_empty() {
                map.insert(s!("mismatches"), json!(
                    Vec::from_iter(mismatches.iter()
                        .map(|m| m.to_json()))));
                context.response.body = Some(json!(map).to_string());
                Err(422)
            } else {
                match ms.write_pact(&output_path) {
                    Ok(_) => Ok(true),
                    Err(err) => {
                        map.insert(s!("error"), json!(format!("Failed to write pact to file - {}", err)));
                        context.response.body = Some(json!(map).to_string());
                        Err(422)
                    }
                }
            }
        },
        Err(_) => Err(422)
    }
}

fn main_resource(base_port: Arc<Option<u16>>, server_manager: Arc<Mutex<ServerManager>>) -> WebmachineResource {
    let server_manager1 = server_manager.clone();

    WebmachineResource {
        allowed_methods: vec![s!("OPTIONS"), s!("GET"), s!("HEAD"), s!("POST")],
        resource_exists: Box::new(|context| context.request.request_path == "/"),
        render_response: Box::new(move |_| {
            let mock_servers = server_manager.lock().unwrap().map_mock_servers(&|ms| {
                ms.to_json()
            });
            let json_response = json!({ s!("mockServers") : json!(mock_servers) });
            Some(json_response.to_string())
        }),
        process_post: Box::new(move |context| start_provider(context, base_port.deref().clone(), server_manager1.clone())),
        .. WebmachineResource::default()
    }
}

fn shutdown_resource(server_key: Arc<String>) -> WebmachineResource {
  WebmachineResource {
    allowed_methods: vec![s!("POST")],
    forbidden: Box::new(move |context: &mut WebmachineContext| {
      !context.request.has_header_value(&"Authorization".to_owned(), &format!("Bearer {}", server_key.deref()))
    }),
    process_post: Box::new(|context| {
      let shutdown_period = match context.request.body {
        Some(ref body) if !body.is_empty() => {
          match serde_json::from_str::<Value>(body) {
            Ok(ref json) => match json.get("period") {
              Some(val) => match val.clone() {
                Value::Number(n) => Ok(n.as_u64().unwrap_or(100)),
                _ => Ok(100)
              },
              None => Ok(100)
            },
            Err(err) => {
              log::error!("Failed to parse json body - {}", err);
              context.response.body = Some(json_error(format!("Failed to parse json body - {}", err)));
              Err(422)
            }
          }
        },
        _ => Ok(100)
      };

      match shutdown_period {
        Ok(period) => {
          thread::spawn(move || {
            log::info!("Scheduling master server to shutdown in {}ms", period);
            thread::sleep(Duration::from_millis(period));
            log::info!("Shutting down");
            process::exit(0);
          });
          Ok(true)
        },
        Err(err) => Err(err)
      }
    }),
    .. WebmachineResource::default()
  }
}

fn mock_server_resource(
    output_path: Arc<Option<String>>,
    server_manager: Arc<Mutex<ServerManager>>
) -> WebmachineResource {
    let server_manager1 = server_manager.clone();
    let server_manager2 = server_manager.clone();
    let server_manager3 = server_manager.clone();

    WebmachineResource {
        allowed_methods: vec![s!("OPTIONS"), s!("GET"), s!("HEAD"), s!("POST"), s!("DELETE")],
        resource_exists: Box::new(move |context| {
            let paths: Vec<String> = context.request.request_path
                .split("/")
                .filter(|p| !p.is_empty())
                .map(|p| p.to_string())
                .collect();
            if paths.len() >= 1 && paths.len() <= 2 {
                match verify::validate_id(&paths[0].clone(), server_manager.clone()) {
                    Ok(ms) => {
                        context.metadata.insert(s!("id"), ms.id.clone());
                        context.metadata.insert(s!("port"), ms.addr.port().to_string());
                        if paths.len() > 1 {
                            context.metadata.insert(s!("subpath"), paths[1].clone());
                            paths[1] == s!("verify")
                        } else {
                            true
                        }
                    },
                    Err(_) => false
                }
            } else {
                false
            }
        }),
        render_response: Box::new(move |context| {
            match context.metadata.get(&s!("subpath")) {
                None => {
                    let id = context.metadata.get(&s!("id")).unwrap().clone();
                    server_manager1.lock().unwrap().find_mock_server_by_id(&id, &|ms| ms.to_json())
                        .map(|json| json.to_string())
                },
                Some(_) => {
                    context.response.status = 405;
                    None
                }
            }
        }),
        process_post: Box::new(move |context| {
            let subpath = context.metadata.get(&s!("subpath")).unwrap().clone();
            if subpath == "verify" {
                verify_mock_server_request(context, output_path.deref(), server_manager2.clone())
            } else {
                Err(422)
            }
        }),
        delete_resource: Box::new(move |context| {
            match context.metadata.get(&s!("subpath")) {
                None => {
                    let id = context.metadata.get(&s!("id")).unwrap().clone();
                    if server_manager3.lock().unwrap().shutdown_mock_server_by_id(id) {
                        Ok(true)
                    } else {
                        Err(404)
                    }
                },
                Some(_) => Err(405)
            }
        }),
        .. WebmachineResource::default()
    }
}

struct ServerHandler {
    output_path: Arc<Option<String>>,
    base_port: Arc<Option<u16>>,
    server_key: Arc<String>,
    server_manager: Arc<Mutex<ServerManager>>
}

impl ServerHandler {
    fn new(output_path: Option<String>, base_port: Option<u16>, server_key: String) -> ServerHandler {
        ServerHandler {
            output_path: Arc::new(output_path),
            base_port: Arc::new(base_port),
            server_key: Arc::new(server_key),
            server_manager: Arc::new(Mutex::new(ServerManager::new()))
        }
    }
}

impl Handler for ServerHandler {
  fn handle(&self, req: Request, res: Response) {
    let dispatcher = WebmachineDispatcher::new(
      btreemap! {
            s!("/") => Arc::new(main_resource(self.base_port.clone(), self.server_manager.clone())),
            s!("/mockserver") => Arc::new(mock_server_resource(self.output_path.clone(), self.server_manager.clone())),
            s!("/shutdown") => Arc::new(shutdown_resource(self.server_key.clone()))
        }
    );
    match dispatcher.dispatch(req, res) {
      Ok(_) => (),
      Err(err) => log::warn!("Error generating response - {}", err)
    };
  }
}

pub fn start_server(port: u16, matches: &ArgMatches) -> Result<(), i32> {
    let output_path = matches.value_of("output").map(|s| s.to_owned());
    let base_port = matches.value_of("base-port").map(|s| s.parse::<u16>().unwrap_or(0));
    let server_key = matches.value_of("server-key").map(|s| s.to_owned())
      .unwrap_or(rand::thread_rng().gen_ascii_chars().take(16).collect::<String>());
    match Server::http(format!("0.0.0.0:{}", port).as_str()) {
        Ok(mut server) => {
            server.keep_alive(None);
            match server.handle(ServerHandler::new(output_path, base_port, server_key.clone())) {
                Ok(listener) => {
                    log::info!("Master server started on port {}", listener.socket.port());
                    log::info!("Server key: '{}'", server_key);
                    Ok(())
                },
                Err(err) => {
                    log::error!("could not bind listener to port: {}", err);
                    Err(2)
                }
            }
        },
        Err(err) => {
            log::error!("could not start master server: {}", err);
            Err(1)
        }
    }
}
