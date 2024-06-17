use std::{
  net::TcpListener,
  process,
  sync::mpsc,
  thread,
  time::Duration
};
use std::net::{Ipv6Addr, SocketAddr};
use std::sync::Arc;
use anyhow::anyhow;

use http::Request;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use itertools::Either;
use maplit::btreemap;
use pact_models::pact::load_pact_from_json;
use pact_models::PactSpecification;
use serde_json::{self, json, Value};
use tokio::select;
use tokio::sync::oneshot::channel;
use tokio::task::JoinSet;
use tracing::{debug, error, info, trace};
use uuid::Uuid;
use webmachine_rust::*;
use webmachine_rust::context::*;
use webmachine_rust::headers::*;

use pact_mock_server::builder::MockServerBuilder;
use pact_mock_server::mock_server::{MockServer, MockServerConfig};

use crate::{SERVER_MANAGER, SERVER_OPTIONS, ServerOpts};
use crate::verify;

// #[cfg(feature = "tls")] use pact_mock_server::tls::TlsConfigBuilder; TODO

fn json_error(error: String) -> String {
    let json_response = json!({ "error" : json!(error) });
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

fn start_provider(context: &mut WebmachineContext, options: ServerOpts) -> Result<bool, u16> {
  debug!("start_provider => {}", context.request.request_path);
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
          let mock_server_id = Uuid::new_v4().to_string();
          let config = MockServerConfig {
            cors_preflight: query_param_set(context, "cors"),
            pact_specification: PactSpecification::default(),
            transport_config: Default::default(),
            .. MockServerConfig::default()
          };
          debug!("Mock server config = {:?}", config);

          #[allow(unused_assignments)]
          let mut result = Err(anyhow!("No mock server started yet"));
          #[cfg(feature = "tls")]
          {
            // result = if query_param_set(context, "tls") {
            //   debug!("Starting TLS mock server with id {}", &mock_server_id);
            //   let key = include_str!("self-signed.key");
            //   let cert = include_str!("self-signed.cert");
            //   TlsConfigBuilder::new()
            //     .key(key.as_bytes())
            //     .cert(cert.as_bytes())
            //     .build()
            //     .map_err(|err| {
            //       format!("Failed to setup TLS using self-signed certificate - {}", err)
            //     })
            //     .and_then(|tls_config| {
            //       let mut guard = SERVER_MANAGER.lock().unwrap();
            //       guard.start_tls_mock_server(mock_server_id.clone(), pact, get_next_port(options.base_port), &tls_config, config)
            //     })
            // } else {
              debug!("Starting mock server with id {}", &mock_server_id);
              result = MockServerBuilder::new()
                .with_pact(pact)
                .with_config(config)
                .bind_to_port(get_next_port(options.base_port))
                .with_id(mock_server_id.as_str())
                .attach_to_global_manager()
                .map(|ms| ms.port());
            // };
          }

          #[cfg(not(feature = "tls"))]
          {
            debug!("Starting mock server with id {}", &mock_server_id);
            result = MockServerBuilder::new()
              .with_pact(pact)
              .with_config(config)
              .bind_to_port(get_next_port(options.base_port))
              .with_id(mock_server_id.as_str())
              .attach_to_manager()
              .map(|ms| ms.port());
          }

          match result {
            Ok(mock_server) => {
              debug!("mock server started on port {}", mock_server);
              let mock_server_json = json!({
                "id" : json!(mock_server_id),
                "port" : json!(mock_server as i64),
              });
              let json_response = json!({ "mockServer" : mock_server_json });
              context.response.body = Some(json_response.to_string().into_bytes());
              context.response.add_header("Location",
                vec![HeaderValue::basic(format!("/mockserver/{}", mock_server_id).as_str())]);
              Ok(true)
            },
            Err(msg) => {
              context.response.body = Some(json_error(format!("Failed to start mock server - {}", msg)).into_bytes());
              Err(422)
            }
          }
        },
        Err(err) => {
            log::error!("Failed to parse json body - {}", err);
            context.response.body = Some(json_error(format!("Failed to parse json body - {}", err)).into_bytes());
            Err(422)
        }
      }
    },
    _ => {
      log::error!("No pact json was supplied");
      context.response.body = Some(json_error("No pact json was supplied".to_string()).into_bytes());
      Err(422)
    }
  }
}

fn query_param_set(context: &mut WebmachineContext, name: &str) -> bool {
  context.request.query.get(name)
    .unwrap_or(&vec![]).first().unwrap_or(&String::default())
    .eq("true")
}

pub fn verify_mock_server_request(context: &mut WebmachineContext) -> Result<bool, u16> {
  let id = context.metadata.get("id").cloned().unwrap_or_default();
  match verify::validate_id(&id, &SERVER_MANAGER) {
    Ok(ms) => {
      let mut map = btreemap!{ "mockServer" => ms.to_json() };
      let mismatches = ms.mismatches();
      if !mismatches.is_empty() {
        map.insert("mismatches", json!(mismatches.iter()
          .map(|m| m.to_json()).collect::<Vec<Value>>()));
        context.response.body = Some(json!(map).to_string().into_bytes());
        Err(422)
      } else {
        let inner = SERVER_OPTIONS.lock().unwrap();
        let options = inner.borrow();
        match ms.write_pact(&options.output_path, false) {
          Ok(_) => Ok(true),
          Err(err) => {
            map.insert("error", json!(format!("Failed to write pact to file - {}", err)));
            context.response.body = Some(json!(map).to_string().into_bytes());
            Err(422)
          }
        }
      }
    },
    Err(_) => Err(422)
  }
}

fn shutdown_resource<'a>() -> WebmachineResource<'a> {
  WebmachineResource {
    allowed_methods: vec!["POST"],
    forbidden: callback(&|context, _| {
      let options = SERVER_OPTIONS.lock().unwrap();
      !context.request.has_header_value(&"Authorization".to_owned(), &format!("Bearer {}", options.borrow().server_key))
    }),
    process_post: callback(&|context, _| {
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
              context.response.body = Some(json_error(format!("Failed to parse json body - {}", err)).into_bytes());
              Err(422)
            }
          }
        }
        _ => Ok(100)
      };

      match shutdown_period {
        Ok(period) => {
          // Need to work out how to do this as the webmachine has to have a static lifetime
          // shutdown.send(()).unwrap_or_default();
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
    ..WebmachineResource::default()
  }
}

fn mock_server_resource<'a>() -> WebmachineResource<'a> {
  WebmachineResource {
    allowed_methods: vec!["OPTIONS", "GET", "HEAD", "POST", "DELETE"],
    resource_exists: callback(&|context, _| {
      debug!("mock_server_resource -> resource_exists");
      let paths: Vec<String> = context.request.request_path
        .split('/')
        .filter(|p| !p.is_empty())
        .map(|p| p.to_string())
        .collect();
      if !paths.is_empty() && paths.len() <= 2 {
        match verify::validate_id(&paths[0].clone(), &SERVER_MANAGER) {
          Ok(ms) => {
            context.metadata.insert("id".to_string(), ms.id.clone());
            context.metadata.insert("port".to_string(), ms.port().to_string());
            if paths.len() > 1 {
              context.metadata.insert("subpath".to_string(), paths[1].clone());
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
    render_response: callback(&|context, _| {
      debug!("mock_server_resource -> render_response");
      match context.metadata.get("subpath") {
        None => {
          let id = context.metadata.get("id").unwrap().clone();
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
        }
        Some(_) => {
          context.response.status = 405;
          None
        }
      }
    }),
    process_post: callback(&|context, _| {
      debug!("mock_server_resource -> process_post");
      let subpath = context.metadata.get("subpath").unwrap().clone();
      if subpath == "verify" {
        verify_mock_server_request(context)
      } else {
        Err(422)
      }
    }),
    delete_resource: callback(&|context, _| {
      debug!("mock_server_resource -> delete_resource");
      match context.metadata.get("subpath") {
        None => {
          let id = context.metadata.get("id").unwrap().clone();
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

pub async fn start_server(port: u16) -> Result<(), i32> {
  let addr = SocketAddr::new(Ipv6Addr::LOCALHOST.into(), port);
  let listener = tokio::net::TcpListener::bind(addr).await
    .map_err(|err|{
      error!("Failed to port {}: {}", port, err);
      1
    })?;
  let (_shutdown_tx, mut shutdown_rx) = channel::<()>();
  let mut join_set = JoinSet::new();

  let local_addr = listener.local_addr()
    .map_err(|err| {
      error!("Failed to get bound address: {}", err);
      2
    })?;
  info!("Master server started on port {}", local_addr.port());
  {
    let inner = SERVER_OPTIONS.lock().unwrap();
    let options = inner.borrow();
    info!("Server key: '{}'", options.server_key);
  }

  let dispatcher = Arc::new(WebmachineDispatcher {
    routes: btreemap! {
      "/" => WebmachineResource {
        allowed_methods: vec!["OPTIONS", "GET", "HEAD", "POST"],
        resource_exists: callback(&|context, _| {
          debug!("main_resource -> resource_exists");
          context.request.request_path == "/"
        }),
        render_response: callback(&|_, _| {
          debug!("main_resource -> render_response");
          let server_manager = SERVER_MANAGER.lock().unwrap();
          trace!("Unlocked server manager");
          let mock_servers = server_manager.map_mock_servers(MockServer::to_json);
          trace!("Got mock server JSON");
          let json_response = json!({ "mockServers" : mock_servers });
          trace!("Returning response");
          Some(json_response.to_string())
        }),
        process_post: callback(&|context, _| {
          debug!("main_resource -> process_post");

          let options = {
            let inner = SERVER_OPTIONS.lock().unwrap();
            inner.clone().into_inner()
          };

          let (tx, rx) = mpsc::channel();
          let (tx2, rx2) = mpsc::channel();

          if let Err(err) = tx.send(context.clone()) {
            error!("Failed to send context to start new mock server - {:?}", err);
            return Err(500)
          }
          let start_fn = move || {
            let handle = thread::current();
            debug!("starting mock server on thread {}", handle.name().unwrap_or("<unknown>"));
            let mut ctx = rx.recv().unwrap();
            let result = start_provider(&mut ctx, options);
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
              let ctx = rx2.recv().unwrap();
              context.response = ctx.response;
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
      "/mockserver" => mock_server_resource(),
      "/shutdown" => shutdown_resource()
    }
  });

  tokio::spawn(async move {
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
  });
  Ok(())
}
