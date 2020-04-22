//! The `pact_mock_server` crate provides a number of exported functions using C bindings for
//! controlling the mock server. These can be used in any language that supports C bindings.
//!
//! ## [create_mock_server](fn.create_mock_server_ffi.html)
//!
//! External interface to create a mock server. A pointer to the pact JSON as a C string is passed in,
//! as well as the port for the mock server to run on. A value of 0 for the port will result in a
//! port being allocated by the operating system. The port of the mock server is returned.
//!
//! ## [mock_server_matched](fn.mock_server_matched_ffi.html)
//!
//! Simple function that returns a boolean value given the port number of the mock service. This value will be true if all
//! the expectations of the pact that the mock server was created with have been met. It will return false if any request did
//! not match, an un-recognised request was received or an expected request was not received.
//!
//! ## [mock_server_mismatches](fn.mock_server_mismatches_ffi.html)
//!
//! This returns all the mismatches, un-expected requests and missing requests in JSON format, given the port number of the
//! mock server.
//!
//! **IMPORTANT NOTE:** The JSON string for the result is allocated on the rust heap, and will have to be freed once the
//! code using the mock server is complete. The [`cleanup_mock_server`](fn.cleanup_mock_server.html) function is provided for this purpose. If the mock
//! server is not cleaned up properly, this will result in memory leaks as the rust heap will not be reclaimed.
//!
//! ## [cleanup_mock_server](fn.cleanup_mock_server.html)
//!
//! This function will try terminate the mock server with the given port number and cleanup any memory allocated for it by
//! the [`mock_server_mismatches`](fn.mock_server_mismatches.html) function. Returns `true`, unless a mock server with the given port number does not exist,
//! or the function fails in some way.
//!
//! **NOTE:** Although `close()` on the listerner for the mock server is called, this does not currently work and the
//! listerner will continue handling requests. In this case, it will always return a 501 once the mock server has been
//! cleaned up.
//!
//! ## [write_pact_file](fn.write_pact_file.html)
//!
//! External interface to trigger a mock server to write out its pact file. This function should
//! be called if all the consumer tests have passed. The directory to write the file to is passed
//! as the second parameter. If a NULL pointer is passed, the current working directory is used.
//!
//! Returns 0 if the pact file was successfully written. Returns a positive code if the file can
//! not be written, or there is no mock server running on that port or the function panics.

#![warn(missing_docs)]

use std::panic::catch_unwind;
use libc::{c_char, size_t, c_ushort};
use std::ffi::CStr;
use std::ffi::CString;
use std::str;
use serde_json::json;
use pact_mock_server::{MockServerError, WritePactFileErr, MANAGER};
use pact_mock_server::server_manager::ServerManager;
use log::*;
use std::any::Any;
use pact_matching::models::{Interaction, OptionalBody, HttpPart};
use pact_matching::models::provider_states::ProviderState;
use maplit::*;
use crate::handles::InteractionPart;

pub mod handles;

/// Initialise the mock server library
#[no_mangle]
pub extern fn init() {
  env_logger::try_init().unwrap_or(());
}

/// External interface to create a mock server. A pointer to the pact JSON as a C string is passed in,
/// as well as the port for the mock server to run on. A value of 0 for the port will result in a
/// port being allocated by the operating system. The port of the mock server is returned.
///
/// # Errors
///
/// Errors are returned as negative values.
///
/// | Error | Description |
/// |-------|-------------|
/// | -1 | A null pointer was received |
/// | -2 | The pact JSON could not be parsed |
/// | -3 | The mock server could not be started |
/// | -4 | The method panicked |
/// | -5 | The address is not valid |
///
#[no_mangle]
pub extern fn create_mock_server(pact_str: *const c_char, addr_str: *const c_char) -> i32 {
    env_logger::try_init().unwrap_or(());

    let result = catch_unwind(|| {
        let c_str = unsafe {
            if pact_str.is_null() {
                log::error!("Got a null pointer instead of pact json");
                return -1;
            }
            CStr::from_ptr(pact_str)
        };

        let addr_c_str = unsafe {
            if addr_str.is_null() {
                log::error!("Got a null pointer instead of listener address");
                return -1;
            }
            CStr::from_ptr(addr_str)
        };

        if let Ok(Ok(addr)) = str::from_utf8(addr_c_str.to_bytes()).map(|s| s.parse::<std::net::SocketAddr>()) {
          match pact_mock_server::create_mock_server(str::from_utf8(c_str.to_bytes()).unwrap(), addr) {
            Ok(ms_port) => ms_port,
            Err(err) => match err {
              MockServerError::InvalidPactJson => -2,
              MockServerError::MockServerFailedToStart => -3
            }
          }
        }
        else {
          -5
        }
    });

    match result {
        Ok(val) => val,
        Err(cause) => {
            log::error!("Caught a general panic: {:?}", cause);
            -4
        }
    }
}

/// External interface to check if a mock server has matched all its requests. The port number is
/// passed in, and if all requests have been matched, true is returned. False is returned if there
/// is no mock server on the given port, or if any request has not been successfully matched, or
/// the method panics.
#[no_mangle]
pub extern fn mock_server_matched(mock_server_port: i32) -> bool {
  let result = catch_unwind(|| {
    pact_mock_server::mock_server_matched(mock_server_port)
  });

  match result {
    Ok(val) => val,
    Err(cause) => {
      log::error!("Caught a general panic: {:?}", cause);
      false
    }
  }
}

/// External interface to get all the mismatches from a mock server. The port number of the mock
/// server is passed in, and a pointer to a C string with the mismatches in JSON format is
/// returned.
///
/// **NOTE:** The JSON string for the result is allocated on the heap, and will have to be freed
/// once the code using the mock server is complete. The [`cleanup_mock_server`](fn.cleanup_mock_server.html) function is
/// provided for this purpose.
///
/// # Errors
///
/// If there is no mock server with the provided port number, or the function panics, a NULL
/// pointer will be returned. Don't try to dereference it, it will not end well for you.
///
#[no_mangle]
pub extern fn mock_server_mismatches(mock_server_port: i32) -> *mut c_char {
  let result = catch_unwind(|| {
    let result = MANAGER.lock().unwrap()
      .get_or_insert_with(ServerManager::new)
      .find_mock_server_by_port_mut(mock_server_port as u16, &|ref mut mock_server| {
        let mismatches = mock_server.mismatches().iter()
          .map(|mismatch| mismatch.to_json() )
          .collect::<Vec<serde_json::Value>>();
        let json = json!(mismatches);
        let s = CString::new(json.to_string()).unwrap();
        let p = s.as_ptr();
        mock_server.resources.push(s);
        p
      });
    match result {
      Some(p) => p as *mut _,
      None => 0 as *mut _
    }
  });

  match result {
    Ok(val) => val,
    Err(cause) => {
      log::error!("Caught a general panic: {:?}", cause);
      0 as *mut _
    }
  }
}

/// External interface to cleanup a mock server. This function will try terminate the mock server
/// with the given port number and cleanup any memory allocated for it. Returns true, unless a
/// mock server with the given port number does not exist, or the function panics.
///
/// **NOTE:** Although `close()` on the listener for the mock server is called, this does not
/// currently work and the listener will continue handling requests. In this
/// case, it will always return a 404 once the mock server has been cleaned up.
#[no_mangle]
pub extern fn cleanup_mock_server(mock_server_port: i32) -> bool {
  let result = catch_unwind(|| {
    MANAGER.lock().unwrap()
      .get_or_insert_with(ServerManager::new)
      .shutdown_mock_server_by_port(mock_server_port as u16)
  });

  match result {
    Ok(val) => val,
    Err(cause) => {
      log::error!("Caught a general panic: {:?}", cause);
      false
    }
  }
}

/// External interface to trigger a mock server to write out its pact file. This function should
/// be called if all the consumer tests have passed. The directory to write the file to is passed
/// as the second parameter. If a NULL pointer is passed, the current working directory is used.
///
/// Returns 0 if the pact file was successfully written. Returns a positive code if the file can
/// not be written, or there is no mock server running on that port or the function panics.
///
/// # Errors
///
/// Errors are returned as positive values.
///
/// | Error | Description |
/// |-------|-------------|
/// | 1 | A general panic was caught |
/// | 2 | The pact file was not able to be written |
/// | 3 | A mock server with the provided port was not found |
#[no_mangle]
pub extern fn write_pact_file(mock_server_port: i32, directory: *const c_char) -> i32 {
  let result = catch_unwind(|| {
    let dir = unsafe {
      if directory.is_null() {
        log::warn!("Directory to write to is NULL, defaulting to the current working directory");
        None
      } else {
        let c_str = CStr::from_ptr(directory);
        let dir_str = str::from_utf8(c_str.to_bytes()).unwrap();
        if dir_str.is_empty() {
          None
        } else {
          Some(dir_str.to_string())
        }
      }
    };

    pact_mock_server::write_pact_file(mock_server_port, dir)
  });

  match result {
    Ok(val) => match val {
      Ok(_) => 0,
      Err(err) => match err {
        WritePactFileErr::IOError => 2,
        WritePactFileErr::NoMockServer => 3
      }
    },
    Err(cause) => {
      log::error!("Caught a general panic: {:?}", cause);
      1
    }
  }
}

/// Creates a new Pact model and returns a handle to it
#[no_mangle]
pub extern fn new_pact(consumer_name: *const c_char, provider_name: *const c_char) -> handles::PactHandle {
  let consumer = convert_cstr(consumer_name, "Consumer");
  let provider = convert_cstr(provider_name, "Provider");
  handles::PactHandle::new(consumer, provider)
}

/// Creates a new Interaction and returns a handle to it
#[no_mangle]
pub extern fn new_interaction(pact: handles::PactHandle, description: *const c_char) -> handles::InteractionHandle {
  let description = convert_cstr(description, "ERROR");
  pact.with_pact(&|_, inner| {
    let interaction = Interaction {
      description: description.to_string(),
      .. Interaction::default()
    };
    inner.interactions.push(interaction);
    handles::InteractionHandle::new(pact.clone(), inner.interactions.len())
  }).unwrap_or(handles::InteractionHandle::new(pact.clone(), 0))
}

/// Sets the description for the Interaction
#[no_mangle]
pub extern fn upon_receiving(interaction: handles::InteractionHandle, description: *const c_char) {
  let description = convert_cstr(description, "ERROR");
  interaction.with_interaction(&|_, inner| {
    inner.description = description.to_string();
  });
}

/// Adds a provider state to the Interaction
#[no_mangle]
pub extern fn given(interaction: handles::InteractionHandle, description: *const c_char) {
  let description = convert_cstr(description, "ERROR");
  interaction.with_interaction(&|_, inner| {
    inner.provider_states.push(ProviderState::default(&description.to_string()));
  });
}

/// Configures the request for the Interaction
#[no_mangle]
pub extern fn with_request(interaction: handles::InteractionHandle, method: *const c_char, path: *const c_char) {
  let method = convert_cstr(method, "GET");
  let path = convert_cstr(path, "/");
  interaction.with_interaction(&|_, inner| {
    inner.request.method = method.to_string();
    inner.request.path = path.to_string();
  });
}

/// Configures a query parameter for the Interaction
#[no_mangle]
pub extern fn with_query_parameter(interaction: handles::InteractionHandle,
                                   name: *const c_char, index: size_t, value: *const c_char) {
  let name = convert_cstr(name, "");
  let value = convert_cstr(value, "");
  if name.len() > 0 {
    interaction.with_interaction(&|_, inner| {
      inner.request.query = inner.request.query.clone().map(|mut q| {
        if q.contains_key(name) {
          let values = q.get_mut(name).unwrap();
          if index > values.len() {
            values.resize_with(index + 1, Default::default);
          }
          values[index] = value.to_string();
        } else {
          let mut values: Vec<String> = Vec::new();
          if index > 0 {
            values.resize_with(index + 1, Default::default);
          }
          values[index] = value.to_string();
          q.insert(name.to_string(), values);
        };
        q
      }).or_else(|| {
        let mut values: Vec<String> = Vec::new();
        if index > 0 {
          values.resize_with(index + 1, Default::default);
          values[index] = value.to_string();
        } else {
          values.push(value.to_string());
        }
        Some(hashmap!{ name.to_string() => values })
      });
    });
  } else {
    warn!("Ignoring query parameter with empty or null name");
  }
}

/// Configures a header for the Interaction
#[no_mangle]
pub extern fn with_header(interaction: handles::InteractionHandle, part: InteractionPart,
                          name: *const c_char, index: size_t, value: *const c_char) {
  let name = convert_cstr(name, "");
  let value = convert_cstr(value, "");
  if name.len() > 0 {
    interaction.with_interaction(&|_, inner| {
      let headers = match part {
        InteractionPart::Request => inner.request.headers.clone(),
        InteractionPart::Response => inner.response.headers.clone()
      };
      let updated_headers = headers.clone().map(|mut h| {
        if h.contains_key(name) {
          let values = h.get_mut(name).unwrap();
          if index > values.len() {
            values.resize_with(index + 1, Default::default);
          }
          values[index] = value.to_string();
        } else {
          let mut values: Vec<String> = Vec::new();
          if index > 0 {
            values.resize_with(index + 1, Default::default);
          }
          values[index] = value.to_string();
          h.insert(name.to_string(), values);
        };
        h
      }).or_else(|| {
        let mut values: Vec<String> = Vec::new();
        if index > 0 {
          values.resize_with(index + 1, Default::default);
          values[index] = value.to_string();
        } else {
          values.push(value.to_string());
        }
        Some(hashmap!{ name.to_string() => values })
      });
      match part {
        InteractionPart::Request => inner.request.headers = updated_headers,
        InteractionPart::Response => inner.response.headers = updated_headers
      };
    });
  } else {
    warn!("Ignoring header with empty or null name");
  }
}

/// Configures the response for the Interaction
#[no_mangle]
pub extern fn response_status(interaction: handles::InteractionHandle, status: c_ushort) {
  interaction.with_interaction(&|_, inner| {
    inner.response.status = status;
  });
}

/// Adds the body for the interaction
#[no_mangle]
pub extern fn with_body(interaction: handles::InteractionHandle, part: InteractionPart,
                        content_type: *const c_char, body: *const c_char) {
  let content_type = convert_cstr(content_type, "text/plain");
  let body = convert_cstr(body, "");
  let content_type_header = "Content-Type".to_string();
  interaction.with_interaction(&|_, inner| {
    let body = OptionalBody::from(dbg!(body));
    match part {
      InteractionPart::Request => {
        inner.request.body = body;
        if !inner.request.has_header(&content_type_header) {
          match inner.request.headers {
            Some(ref mut headers) => {
              headers.insert(content_type_header.clone(), vec![ content_type.to_string() ]);
            },
            None => {
              inner.request.headers = Some(hashmap! { content_type_header.clone() => vec![ content_type.to_string() ]});
            }
          }
        }
      },
      InteractionPart::Response => {
        inner.response.body = body;
        if !inner.response.has_header(&content_type_header) {
          match inner.response.headers {
            Some(ref mut headers) => {
              headers.insert(content_type_header.clone(), vec![ content_type.to_string() ]);
            },
            None => {
              inner.response.headers = Some(hashmap! { content_type_header.clone() => vec![ content_type.to_string() ]});
            }
          }
        }
      }
    };
  });
}

fn error_message(err: Box<dyn Any>, method: &str) -> String {
  if let Some(err) = err.downcast_ref::<&str>() {
    format!("{} failed with an error - {}", method, err)
  } else if let Some(err) = err.downcast_ref::<String>() {
    format!("{} failed with an error - {}", method, err)
  } else {
    format!("{} failed with an unknown error", method)
  }
}

fn convert_cstr(name: *const c_char, default: &str) -> &str {
  unsafe {
    if name.is_null() {
      warn!("{} name is NULL, defaulting to '{}'", default, default);
      default
    } else {
      let c_str = CStr::from_ptr(name);
      match c_str.to_str() {
        Ok(str) => str,
        Err(err) => {
          warn!("Failed to parse {} name as a UTF-8 string, defaulting to '{}': {}", default, default, err);
          default
        }
      }
    }
  }
}
