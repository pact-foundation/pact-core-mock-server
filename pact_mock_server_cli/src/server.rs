use std::{
  net::TcpListener,
  process,
  sync::mpsc,
  thread,
  time::Duration
};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use anyhow::anyhow;
use bytes::Bytes;
use http::Request;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use itertools::Either;
use maplit::btreemap;
use pact_models::generators::generate_hexadecimal;
use pact_models::pact::load_pact_from_json;
#[cfg(feature = "tls")] use pact_models::pact::Pact;
#[cfg(feature = "tls")] use rustls::crypto::ring::default_provider;
#[cfg(feature = "tls")] use rustls::crypto::CryptoProvider;
use serde_json::{self, json, Value};
use tokio::select;
use tokio::sync::oneshot::{channel, Sender};
use tokio::task::JoinSet;
use tracing::{debug, error, info, trace};
use webmachine_rust::*;
use webmachine_rust::context::*;
use webmachine_rust::headers::*;

use pact_mock_server::builder::MockServerBuilder;
use pact_mock_server::mock_server::{MockServer, MockServerConfig};

use crate::{SERVER_MANAGER, ServerOpts};
use crate::verify;

fn json_error(error: String) -> Bytes {
  let json_response = json!({ "error" : json!(error) });
  Bytes::from(json_response.to_string())
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

fn start_provider(context: &mut WebmachineContext) -> Result<bool, u16> {
  debug!("start_provider => {}", context.request.request_path);

  let base_port = context.metadata.get("base-port")
    .map(|port| {
      let port = port.to_string();
      if port.is_empty() {
        None
      } else {
        match port.parse::<u16>() {
          Ok(port) => Some(port),
          Err(err) => {
            error!("Failed to parse the server base port: {}", err);
            None
          }
        }
      }
    })
    .flatten();

  match context.request.body {
    Some(ref body) if !body.is_empty() => {
      match serde_json::from_slice(body) {
        Ok(ref json) => {
          let pact = load_pact_from_json(&context.request.request_path, json)
            .map_err(|err| {
              error!("Failed to parse Pact JSON - {}", err);
              422_u16
            })?;
          debug!("Loaded pact = {:?}", pact);
          let mock_server_id = generate_hexadecimal(8);
          let pact_specification = match context.request.query.get("specification") {
            Some(specs) => {
              match specs.first() {
                Some(spec) if !spec.is_empty() => Some(spec.clone()),
                _ => None,
              }
            },
            None => None
          };

          let mut config = MockServerConfig {
            cors_preflight: query_param_set(context, "cors"),
            .. MockServerConfig::default()
            };
          if let Some(spec) = pact_specification {
            config.pact_specification = spec.into();
          }
          debug!("Mock server config = {:?}", config);

          #[allow(unused_assignments)]
          let mut result = Err(anyhow!("No mock server started yet"));
          #[cfg(feature = "tls")]
          {
            if query_param_set(context, "tls") {
              if CryptoProvider::get_default().is_none() {
                if let Err(_) = CryptoProvider::install_default(default_provider()) {
                  error!("Failed to install the default FIPS cryptographic provider");
                  result = Err(anyhow!("Failed to install the default FIPS cryptographic provider"))
                } else {
                  result = start_https_server(pact, config, get_next_port(base_port), &mock_server_id)
                }
              } else {
                result = start_https_server(pact, config, get_next_port(base_port), &mock_server_id)
              }
            } else {
              debug!("Starting mock server with id {}", &mock_server_id);
              let mut server_manager = SERVER_MANAGER.lock().unwrap();
              trace!("Unlocked server manager");
              result = MockServerBuilder::new()
                .with_pact(pact)
                .with_config(config)
                .bind_to_ip4_port(get_next_port(base_port))
                .with_id(mock_server_id.as_str())
                .attach_to_manager(&mut server_manager);
            };
          }

          #[cfg(not(feature = "tls"))]
          {
            debug!("Starting mock server with id {}", &mock_server_id);
            let mut server_manager = crate::SERVER_MANAGER.lock().unwrap();
            trace!("Unlocked server manager");
            result = MockServerBuilder::new()
              .with_pact(pact)
              .with_config(config)
              .bind_to_ip4_port(get_next_port(base_port))
              .with_id(mock_server_id.as_str())
              .attach_to_manager(&mut server_manager);
          }

          match result {
            Ok(mock_server) => {
              debug!("Mock server started on port {}", mock_server.port());
              let mock_server_json = json!({
                "id" : json!(mock_server_id),
                "port" : json!(mock_server.port() as i64),
              });
              let json_response = json!({ "mockServer" : mock_server_json });
              context.response.body = Some(Bytes::from(json_response.to_string()));
              context.response.add_header("Location",
                vec![HeaderValue::basic(format!("/mockserver/{}", mock_server_id).as_str())]);
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
      context.response.body = Some(json_error("No pact json was supplied".to_string()));
      Err(422)
    }
  }
}

#[cfg(feature = "tls")]
fn start_https_server(
  pact: Box<dyn Pact + Send + Sync>,
  config: MockServerConfig,
  port: u16,
  id: &String
) -> anyhow::Result<MockServer> {
  debug!("Starting TLS mock server with id {}", id);
  let mut server_manager = SERVER_MANAGER.lock().unwrap();
  trace!("Unlocked server manager");
  MockServerBuilder::new()
    .with_pact(pact)
    .with_config(config)
    .bind_to_port(port)
    .with_id(id.as_str())
    .with_self_signed_tls()?
    .attach_to_manager(&mut server_manager)
}

fn query_param_set(context: &mut WebmachineContext, name: &str) -> bool {
  context.request.query.get(name)
    .unwrap_or(&vec![]).first().unwrap_or(&String::default())
    .eq("true")
}

pub fn verify_mock_server_request(context: &mut WebmachineContext, output_path: &Option<String>) -> Result<bool, u16> {
  let id = match context.metadata.get("id") {
    Some(id) => id.to_string(),
    None => {
      error!("Mock server ID is missing from context");
      return Err(500)
    }
  };
  match verify::validate_id(&id, &SERVER_MANAGER) {
    Ok(ms) => {
      let mut map = btreemap!{ "mockServer" => ms.to_json() };
      let mismatches = ms.mismatches();
      if !mismatches.is_empty() {
        map.insert("mismatches", json!(mismatches.iter()
          .map(|m| m.to_json()).collect::<Vec<Value>>()));
        context.response.body = Some(Bytes::from(json!(map).to_string()));
        Err(422)
      } else {
        match ms.write_pact(output_path, false) {
          Ok(_) => Ok(true),
          Err(err) => {
            map.insert("error", json!(format!("Failed to write pact to file - {}", err)));
            context.response.body = Some(Bytes::from(json!(map).to_string()));
            Err(422)
          }
        }
      }
    },
    Err(_) => Err(422)
  }
}

fn shutdown_resource(auth: String, shutdown_tx: Arc<Sender<()>>) -> WebmachineResource {
  WebmachineResource {
    allowed_methods: owned_vec(&["POST"]),
    forbidden: callback(move |context, _| {
      let auth = auth.clone();
      !context.request.has_header_value("Authorization", auth.as_str())
    }),
    process_post: callback(move |context, _| {
      let shutdown_period = match context.request.body {
        Some(ref body) if !body.is_empty() => {
          match serde_json::from_slice::<Value>(body) {
            Ok(ref json) => match json.get("period") {
              Some(val) => match val.clone() {
                Value::Number(n) => Ok(n.as_u64().unwrap_or(100)),
                _ => Ok(100)
              },
              None => Ok(100)
            },
            Err(err) => {
              error!("Failed to parse json body - {}", err);
              context.response.body = Some(Bytes::from(json_error(format!("Failed to parse json body - {}", err))));
              Err(422)
            }
          }
        }
        _ => Ok(100)
      };

      match shutdown_period {
        Ok(period) => {
          if let Some(shutdown_tx) = Arc::into_inner(shutdown_tx.clone()) {
            let _ = shutdown_tx.send(());
          }
          thread::spawn(move || {
            info!("Scheduling master server to shutdown in {}ms", period);
            thread::sleep(Duration::from_millis(period));
            info!("Shutting down");
            process::exit(0);
          });
          Ok(true)
        }
        Err(err) => Err(err)
      }
    }),
    .. WebmachineResource::default()
  }
}

fn mock_server_resource(options: ServerOpts) -> WebmachineResource {
  let output_path = options.output_path.clone();
  WebmachineResource {
    allowed_methods: owned_vec(&["OPTIONS", "GET", "HEAD", "POST", "DELETE"]),
    resource_exists: callback(|context, _| {
      debug!("mock_server_resource -> resource_exists");
      let paths: Vec<String> = context.request.request_path
        .split('/')
        .filter(|p| !p.is_empty())
        .map(|p| p.to_string())
        .collect();
      if !paths.is_empty() && paths.len() <= 2 {
        match verify::validate_id(&paths[0].clone(), &SERVER_MANAGER) {
          Ok(ms) => {
            context.metadata.insert("id".to_string(), ms.id.clone().into());
            context.metadata.insert("port".to_string(), ms.port().into());
            if paths.len() > 1 {
              context.metadata.insert("subpath".to_string(), paths[1].as_str().into());
              paths[1] == "verify"
            } else {
              true
            }
          }
          Err(_) => false
        }
      } else {
        false
      }
    }),
    render_response: callback(|context, _| {
      debug!("mock_server_resource -> render_response");
      match context.metadata.get("subpath") {
        None => {
          let id = context.metadata.get("id").unwrap_or_default().to_string();
          debug!("Mock server id = {}", id);
          let response = {
            let guard = SERVER_MANAGER.lock().unwrap();
            guard.find_mock_server_by_id(&id, &|_, ms| match ms {
              Either::Left(ms) => (Some(ms.to_json().to_string()), None),
              Either::Right(_plugin) => {
                error!("Plugin mock servers are not currently supported");
                (None, Some(422))
              }
            })
          };
          match response {
            Some((res, Some(status))) => {
              context.response.status = status;
              res
            }
            Some((res, None)) => res,
            None => None
          }
            .map(|res| Bytes::from(res))
        }
        Some(_) => {
          context.response.status = 405;
          None
        }
      }
    }),
    process_post: callback(move |context, _| {
      debug!("mock_server_resource -> process_post");
      let subpath = context.metadata.get("subpath").unwrap_or_default().to_string();
      if subpath == "verify" {
        verify_mock_server_request(context, &output_path)
      } else {
        Err(422)
      }
    }),
    delete_resource: callback(|context, _| {
      debug!("mock_server_resource -> delete_resource");
      match context.metadata.get("subpath") {
        None => {
          let id = context.metadata.get("id").unwrap_or_default().to_string();
          thread::spawn(move || {
            if SERVER_MANAGER.lock().unwrap().shutdown_mock_server_by_id(id) {
              Ok(true)
            } else {
              Err(404)
            }
          }).join().expect("Could not spawn thread to shut down mock server")
        }
        Some(_) => Err(405)
      }
    }),
    ..WebmachineResource::default()
  }
}

pub async fn start_server(port: u16, options: ServerOpts) -> Result<(), i32> {
  let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), port);
  let listener = tokio::net::TcpListener::bind(addr).await
    .map_err(|err|{
      error!("Failed to bind to localhost port {}: {}", port, err);
      1
    })?;

  let (shutdown_tx, mut shutdown_rx) = channel::<()>();
  let mut join_set = JoinSet::new();

  let local_addr = listener.local_addr()
    .map_err(|err| {
      error!("Failed to get bound address: {}", err);
      2
    })?;
  info!("Master server started on port {}", local_addr.port());
  info!("Server key: '{}'", options.server_key);

  let auth = format!("Bearer {}", options.server_key);
  let base_port = options.base_port.clone();
  let options = options.clone();
  let dispatcher = Arc::new(WebmachineDispatcher {
    routes: btreemap! {
      "/" => WebmachineResource {
        allowed_methods: owned_vec(&["OPTIONS", "GET", "HEAD", "POST"]),
        resource_exists: callback(|context, _| {
          trace!("main_resource -> resource_exists");
          context.request.request_path == "/"
        }),
        render_response: callback(|_, _| {
          trace!("main_resource -> render_response");
          let server_manager = SERVER_MANAGER.lock().unwrap();
          trace!("Unlocked server manager");
          let mock_servers = server_manager.map_mock_servers(MockServer::to_json);
          trace!(?mock_servers, "Got mock server JSON");
          let json_response = json!({ "mockServers" : mock_servers });
          trace!("Returning response");
          Some(json_response.to_string().into())
        }),
        process_post: callback(move |context, _| {
          trace!("main_resource -> process_post");

          let (tx, rx) = mpsc::channel();
          let (tx2, rx2) = mpsc::channel();

          if let Some(base_port) = base_port {
            context.metadata.insert("base_port".to_string(), base_port.into());
          }
          if let Err(err) = tx.send(context.clone()) {
            error!("Failed to send context to start new mock server - {:?}", err);
            return Err(500)
          }

          let start_fn = move || {
            let handle = thread::current();
            debug!("starting mock server on thread {}", handle.name().unwrap_or("<unknown>"));
            let mut ctx = rx.recv().map_err(|err| {
              error!("Failed to receive context from channel: {}", err);
              500_u16
            })?;
            let result = start_provider(&mut ctx);
            debug!("Result of starting mock server: {:?}", result.clone());
            match tx2.send(ctx) {
              Ok(_) => result,
              Err(err) => {
                error!("Failed to send result back to main resource - {:?}", err);
                Err(500)
              }
            }
          };

          match thread::spawn(start_fn).join() {
            Ok(res) => {
              debug!("Result of thread: {:?}", res);
              let ctx = rx2.recv().map_err(|err| {
                error!("Failed to receive final context from channel: {}", err);
                500_u16
              })?;
              context.response = ctx.response;
              trace!("Final context = {:?}", context);
              res
            },
            Err(err) => {
              error!("Failed to spawn new thread to start new mock server - {:?}", err);
              Err(500)
            }
          }
        }),
        .. WebmachineResource::default()
      },
      "/mockserver" => mock_server_resource(options.clone()),
      "/shutdown" => shutdown_resource(auth, Arc::new(shutdown_tx))
    }
  });

  trace!("Starting main server loop");
  loop {
    let dispatcher = dispatcher.clone();
    select! {
      connection = listener.accept() => {
        match connection {
          Ok((stream, remote_address)) => {
            debug!("Received connection from remote {}", remote_address);
            let io = TokioIo::new(stream);
            join_set.spawn(async move {
              if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(|req: Request<Incoming>| dispatcher.dispatch(req))).await {
                error!("Failed to serve incoming connection: {err}");
              }
            });
          },
          Err(e) => {
            error!("Failed to accept connection: {e}");
          }
        }
      }

      _ = &mut shutdown_rx => {
        debug!("Received shutdown signal, waiting for existing connections to complete");
        while let Some(_) = join_set.join_next().await {};
        debug!("Existing connections complete, exiting main loop");
        break;
      }
    }
  }
  trace!("Main server loop done");
  Ok(())
}
