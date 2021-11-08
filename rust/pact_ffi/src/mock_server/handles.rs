//! Handles wrapping Rust models

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Mutex;

use bytes::Bytes;
use itertools::Itertools;
use lazy_static::*;
use libc::{c_char, c_ushort, size_t};
use log::*;
use maplit::*;
use serde_json::{json, Value};

use pact_models::{Consumer, PactSpecification, Provider};
use pact_models::bodies::OptionalBody;
use pact_models::generators::Generators;
use pact_models::http_parts::HttpPart;
use pact_models::json_utils::json_to_string;
use pact_models::matchingrules::{MatchingRuleCategory, MatchingRules};
use pact_models::path_exp::DocPath;
use pact_models::prelude::Pact;
use pact_models::prelude::v4::V4Pact;
use pact_models::provider_states::ProviderState;
use pact_models::v4::interaction::V4Interaction;
use pact_models::v4::synch_http::SynchronousHttp;

use crate::convert_cstr;
use crate::mock_server::bodies::{process_json, process_object};
use crate::mock_server::xml;

#[derive(Debug, Clone)]
/// Pact handle inner struct
pub struct PactHandleInner {
  pub(crate) pact: V4Pact,
  pub(crate) mock_server_started: bool,
  pub(crate) specification_version: PactSpecification
}

lazy_static! {
  static ref PACT_HANDLES: Mutex<HashMap<usize, RefCell<PactHandleInner>>> = Mutex::new(hashmap![]);
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// Wraps a Pact model struct
pub struct PactHandle {
  /// Pact reference
  pub pact: usize
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// Wraps a Pact model struct
pub struct InteractionHandle {
  /// Pact reference
  pub pact: usize,
  /// Interaction reference
  pub interaction: usize
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// Request or Response enum
pub enum InteractionPart {
  /// Request part
  Request,
  /// Response part
  Response
}

impl PactHandle {
  /// Creates a new handle to a Pact model
  pub fn new(consumer: &str, provider: &str) -> Self {
    let mut handles = PACT_HANDLES.lock().unwrap();
    let id = handles.len() + 1;
    let mut pact = V4Pact {
      consumer: Consumer { name: consumer.to_string() },
      provider: Provider { name: provider.to_string() },
      ..V4Pact::default()
    };
    pact.add_md_version("ffi", option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"));
    handles.insert(id, RefCell::new(PactHandleInner {
      pact,
      mock_server_started: false,
      specification_version: PactSpecification::V3
    }));
    PactHandle {
      pact: id
    }
  }

  /// Invokes the closure with the inner Pact model
  pub(crate) fn with_pact<R>(&self, f: &dyn Fn(usize, &mut PactHandleInner) -> R) -> Option<R> {
    let mut handles = PACT_HANDLES.lock().unwrap();
    handles.get_mut(&self.pact).map(|inner| f(self.pact - 1, &mut inner.borrow_mut()))
  }
}

impl InteractionHandle {
  /// Creates a new handle to an Interaction
  pub fn new(pact: PactHandle, interaction: usize) -> InteractionHandle {
    InteractionHandle {
      pact: pact.pact,
      interaction
    }
  }

  /// Invokes the closure with the inner Pact model
  pub fn with_pact<R>(&self, f: &dyn Fn(usize, &mut PactHandleInner) -> R) -> Option<R> {
    let mut handles = PACT_HANDLES.lock().unwrap();
    handles.get_mut(&self.pact).map(|inner| f(self.pact - 1, &mut inner.borrow_mut()))
  }

  /// Invokes the closure with the inner Interaction model
  pub fn with_interaction<R>(&self, f: &dyn Fn(usize, bool, &mut dyn V4Interaction) -> R) -> Option<R> {
    let mut handles = PACT_HANDLES.lock().unwrap();
    handles.get_mut(&self.pact).map(|inner| {
      let inner_mut = &mut *inner.borrow_mut();
      let interactions = &mut inner_mut.pact.interactions;
      match interactions.get_mut(self.interaction - 1) {
        Some(inner_i) => {
          Some(f(self.interaction - 1, inner_mut.mock_server_started, inner_i.as_mut()))
        },
        None => None
      }
    }).flatten()
  }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// Wraps a Pact model struct
pub struct MessagePactHandle {
  /// Pact reference
  pub pact: usize
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// Wraps a Pact model struct
pub struct MessageHandle {
  /// Message Pact reference
  pub pact: usize,
  /// Interaction reference
  pub message: usize
}

impl MessagePactHandle {
  /// Creates a new handle to a Pact model
  pub fn new(consumer: &str, provider: &str) -> Self {
    let mut handles = PACT_HANDLES.lock().unwrap();
    let id = handles.len() + 1;
    let mut pact = V4Pact {
      consumer: Consumer { name: consumer.to_string() },
      provider: Provider { name: provider.to_string() },
      ..V4Pact::default()
    };
    pact.add_md_version("ffi", option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"));
    handles.insert(id, RefCell::new(PactHandleInner {
      pact,
      mock_server_started: false,
      specification_version: PactSpecification::V3
    }));
    MessagePactHandle {
      pact: id
    }
  }

  /// Invokes the closure with the inner model
  pub fn with_pact<R>(&self, f: &dyn Fn(usize, &mut V4Pact, PactSpecification) -> R) -> Option<R> {
    let mut handles = PACT_HANDLES.lock().unwrap();
    handles.get_mut(&self.pact).map(|inner| {
      let mut ref_mut = inner.borrow_mut();
      let specification = ref_mut.specification_version;
      f(self.pact - 1, &mut ref_mut.pact, specification)
    })
  }
}

impl MessageHandle {
  /// Creates a new handle to a message
  pub fn new(pact: MessagePactHandle, message: usize) -> MessageHandle {
    MessageHandle {
      pact: pact.pact,
      message
    }
  }

  /// Invokes the closure with the inner model
  pub fn with_pact<R>(&self, f: &dyn Fn(usize, &mut V4Pact, PactSpecification) -> R) -> Option<R> {
    let mut handles = PACT_HANDLES.lock().unwrap();
    handles.get_mut(&self.pact).map(|inner| {
      let mut ref_mut = inner.borrow_mut();
      let specification = ref_mut.specification_version;
      f(self.pact - 1, & mut ref_mut.pact, specification)
    })
  }

  /// Invokes the closure with the inner Interaction model
  pub fn with_message<R>(&self, f: &dyn Fn(usize, &mut dyn V4Interaction, PactSpecification) -> R) -> Option<R> {
    let mut handles = PACT_HANDLES.lock().unwrap();
    handles.get_mut(&self.pact).map(|inner| {
      let mut ref_mut = inner.borrow_mut();
      let specification = ref_mut.specification_version;
      ref_mut.pact.interactions.get_mut(self.message - 1)
        .map(|inner_i| {
          if inner_i.is_message() {
            Some(f(self.message - 1, inner_i.as_mut(), specification))
          } else {
            error!("Interaction {} is not a message interaction, it is {}", self.message, inner_i.type_of());
            None
          }
        }).flatten()
    }).flatten()
  }
}


/// Creates a new Pact model and returns a handle to it.
///
/// * `consumer_name` - The name of the consumer for the pact.
/// * `provider_name` - The name of the provider for the pact.
///
/// Returns a new `PactHandle`.
#[no_mangle]
pub extern fn pactffi_new_pact(consumer_name: *const c_char, provider_name: *const c_char) -> PactHandle {
  let consumer = convert_cstr("consumer_name", consumer_name).unwrap_or("Consumer");
  let provider = convert_cstr("provider_name", provider_name).unwrap_or("Provider");
  PactHandle::new(consumer, provider)
}

/// Creates a new HTTP Interaction and returns a handle to it.
///
/// * `description` - The interaction description. It needs to be unique for each interaction.
///
/// Returns a new `InteractionHandle`.
#[no_mangle]
pub extern fn pactffi_new_interaction(pact: PactHandle, description: *const c_char) -> InteractionHandle {
  if let Some(description) = convert_cstr("description", description) {
    pact.with_pact(&|_, inner| {
      let interaction = SynchronousHttp {
        description: description.to_string(),
        ..SynchronousHttp::default()
      };
      inner.pact.interactions.push(interaction.boxed_v4());
      InteractionHandle::new(pact, inner.pact.interactions.len())
    }).unwrap_or_else(|| InteractionHandle::new(pact, 0))
  } else {
    InteractionHandle::new(pact, 0)
  }
}

/// Sets the description for the Interaction. Returns false if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started)
///
/// * `description` - The interaction description. It needs to be unique for each interaction.
#[no_mangle]
pub extern fn pactffi_upon_receiving(interaction: InteractionHandle, description: *const c_char) -> bool {
  if let Some(description) = convert_cstr("description", description) {
    interaction.with_interaction(&|_, mock_server_started, inner| {
      inner.set_description(description);
      !mock_server_started
    }).unwrap_or(false)
  } else {
    false
  }
}

/// Adds a provider state to the Interaction. Returns false if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started)
///
/// * `description` - The provider state description. It needs to be unique.
#[no_mangle]
pub extern fn pactffi_given(interaction: InteractionHandle, description: *const c_char) -> bool {
  if let Some(description) = convert_cstr("description", description) {
    interaction.with_interaction(&|_, mock_server_started, inner| {
      inner.provider_states_mut().push(ProviderState::default(&description.to_string()));
      !mock_server_started
    }).unwrap_or(false)
  } else {
    false
  }
}

/// Adds a provider state to the Interaction with a parameter key and value. Returns false if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started)
///
/// * `description` - The provider state description. It needs to be unique.
/// * `name` - Parameter name.
/// * `value` - Parameter value.
#[no_mangle]
pub extern fn pactffi_given_with_param(interaction: InteractionHandle, description: *const c_char,
                                       name: *const c_char, value: *const c_char) -> bool {
  if let Some(description) = convert_cstr("description", description) {
    if let Some(name) = convert_cstr("name", name) {
      let value = convert_cstr("value", value).unwrap_or_default();
      interaction.with_interaction(&|_, mock_server_started, inner| {
        let value = match serde_json::from_str(value) {
          Ok(json) => json,
          Err(_) => json!(value)
        };
        match inner.provider_states().iter().find_position(|state| state.name == description) {
          Some((index, _)) => {
            inner.provider_states_mut().get_mut(index).unwrap().params.insert(name.to_string(), value);
          },
          None => inner.provider_states_mut().push(ProviderState {
            name: description.to_string(),
            params: hashmap!{ name.to_string() => value }
          })
        };
        !mock_server_started
      }).unwrap_or(false)
    } else {
      false
    }
  } else {
    false
  }
}

/// Configures the request for the Interaction. Returns false if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started)
///
/// * `method` - The request method. Defaults to GET.
/// * `path` - The request path. Defaults to `/`.
#[no_mangle]
pub extern fn pactffi_with_request(
  interaction: InteractionHandle,
  method: *const c_char,
  path: *const c_char
) -> bool {
  let method = convert_cstr("method", method).unwrap_or("GET");
  let path = convert_cstr("path", path).unwrap_or("/");

  interaction.with_interaction(&|_, mock_server_started, inner| {
    if let Some(reqres) = inner.as_v4_http_mut() {
      let path = from_integration_json(&mut reqres.request.matching_rules, &mut reqres.request.generators, &path.to_string(), DocPath::empty(), "path");
      reqres.request.method = method.to_string();
      reqres.request.path = path;
      !mock_server_started
    } else {
      error!("Interaction is not an HTTP interaction, is {}", inner.type_of());
      false
    }
  }).unwrap_or(false)
}

/// Configures a query parameter for the Interaction. Returns false if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started)
///
/// * `name` - the query parameter name.
/// * `value` - the query parameter value.
/// * `index` - the index of the value (starts at 0). You can use this to create a query parameter with multiple values
#[no_mangle]
pub extern fn pactffi_with_query_parameter(
  interaction: InteractionHandle,
  name: *const c_char,
  index: size_t,
  value: *const c_char
) -> bool {
  if let Some(name) = convert_cstr("name", name) {
    let value = convert_cstr("value", value).unwrap_or_default();
    interaction.with_interaction(&|_, mock_server_started, inner| {
      if let Some(reqres) = inner.as_v4_http_mut() {
        reqres.request.query = reqres.request.query.clone().map(|mut q| {
          let mut path = DocPath::root();
          path.push_field(name).push_index(index);
          let value = from_integration_json(&mut reqres.request.matching_rules, &mut reqres.request.generators, &value.to_string(), path, "query");
          if q.contains_key(name) {
            let values = q.get_mut(name).unwrap();
            if index >= values.len() {
              values.resize_with(index + 1, Default::default);
            }
            values[index] = value;
          } else {
            let mut values: Vec<String> = Vec::new();
            values.resize_with(index + 1, Default::default);
            values[index] = value;
            q.insert(name.to_string(), values);
          };
          q
        }).or_else(|| {
          let mut path = DocPath::root();
          path.push_field(name).push_index(index);
          let value = from_integration_json(&mut reqres.request.matching_rules, &mut reqres.request.generators, &value.to_string(), path, "query");
          let mut values: Vec<String> = Vec::new();
          values.resize_with(index + 1, Default::default);
          values[index] = value;
          Some(hashmap! { name.to_string() => values })
        });
        !mock_server_started
      } else {
        error!("Interaction is not an HTTP interaction, is {}", inner.type_of());
        false
      }
    }).unwrap_or(false)
  } else {
    warn!("Ignoring query parameter with empty or null name");
    false
  }
}

/// Convert JSON matching rule structures into their internal representation (excl. bodies)
///
/// For non-body values (headers, query, path etc.) extract out the value from any matchers
/// and apply the matchers/generators to the model
fn from_integration_json(
  rules: &mut MatchingRules,
  generators: &mut Generators,
  value: &str,
  path: DocPath,
  category: &str,
) -> String {
  let category = rules.add_category(category);

  match serde_json::from_str(value) {
    Ok(json) => match json {
      serde_json::Value::Object(ref map) => {
        let json: serde_json::Value = process_object(map, category, generators, path, false, false);
        // These are simple JSON primitives (strings), so we must unescape them
        json_to_string(&json)
      },
      _ => value.to_string()
    },
    Err(_) => value.to_string()
  }
}

pub(crate) fn process_xml(body: String, matching_rules: &mut MatchingRuleCategory, generators: &mut Generators) -> Result<Vec<u8>, String> {
  trace!("process_xml");
  match serde_json::from_str(&body) {
    Ok(json) => match json {
      Value::Object(ref map) => xml::generate_xml_body(map, matching_rules, generators),
      _ => Err(format!("JSON document is invalid (expected an Object), have {}", json))
    },
    Err(err) => Err(format!("Failed to parse XML builder document: {}", err))
  }
}

/// Sets the specification version for a given Pact model. Returns false if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started) or the version is invalid
///
/// * `pact` - Handle to a Pact model
/// * `version` - the spec version to use
#[no_mangle]
pub extern fn pactffi_with_specification(pact: PactHandle, version: PactSpecification) -> bool {
  pact.with_pact(&|_, inner| {
    inner.specification_version = version.into();
    !inner.mock_server_started
  }).unwrap_or(false)
}

/// Sets the additional metadata on the Pact file. Common uses are to add the client library details such as the name and version
/// Returns false if the interaction or Pact can't be modified (i.e. the mock server for it has already started)
///
/// * `pact` - Handle to a Pact model
/// * `namespace` - the top level metadat key to set any key values on
/// * `name` - the key to set
/// * `value` - the value to set
#[no_mangle]
pub extern fn pactffi_with_pact_metadata(
  pact: PactHandle,
  namespace: *const c_char,
  name: *const c_char,
  value: *const c_char
) -> bool {
  pact.with_pact(&|_, inner| {
    let namespace = convert_cstr("namespace", namespace).unwrap_or_default();
    let name = convert_cstr("name", name).unwrap_or_default();
    let value = convert_cstr("value", value).unwrap_or_default();

    if !namespace.is_empty() {
      inner.pact.metadata.insert(namespace.to_string(), json!({ name: value }));
    } else {
      log::warn!("no namespace provided for metadata {:?} => {:?}. Ignoring", name, value);
    }
    !inner.mock_server_started
  }).unwrap_or(false)
}

/// Configures a header for the Interaction. Returns false if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started)
///
/// * `part` - The part of the interaction to add the header to (Request or Response).
/// * `name` - the header name.
/// * `value` - the header value.
/// * `index` - the index of the value (starts at 0). You can use this to create a header with multiple values
#[no_mangle]
pub extern fn pactffi_with_header(
  interaction: InteractionHandle,
  part: InteractionPart,
  name: *const c_char,
  index: size_t,
  value: *const c_char
) -> bool {
  if let Some(name) = convert_cstr("name", name) {
    let value = convert_cstr("value", value).unwrap_or_default();
    interaction.with_interaction(&|_, mock_server_started, inner| {
      if let Some(reqres) = inner.as_v4_http_mut() {
        let headers = match part {
          InteractionPart::Request => reqres.request.headers.clone(),
          InteractionPart::Response => reqres.response.headers.clone()
        };

        let mut path = DocPath::root();
        path.push_field(name);
        let value = match part {
          InteractionPart::Request => from_integration_json(
            &mut reqres.request.matching_rules,
            &mut reqres.request.generators,
            &value.to_string(),
            path,
            "header"),
          InteractionPart::Response => from_integration_json(
            &mut reqres.response.matching_rules,
            &mut reqres.response.generators,
            &value.to_string(),
            path,
            "header")
        };

        let updated_headers = headers.map(|mut h| {
          if h.contains_key(name) {
            let values = h.get_mut(name).unwrap();
            if index >= values.len() {
              values.resize_with(index + 1, Default::default);
            }
            values[index] = value.to_string();
          } else {
            let mut values: Vec<String> = Vec::new();
            values.resize_with(index + 1, Default::default);
            values[index] = value.to_string();
            h.insert(name.to_string(), values);
          };
          h
        }).or_else(|| {
          let mut values: Vec<String> = Vec::new();
          values.resize_with(index + 1, Default::default);
          values[index] = value.to_string();
          Some(hashmap! { name.to_string() => values })
        });
        match part {
          InteractionPart::Request => reqres.request.headers = updated_headers,
          InteractionPart::Response => reqres.response.headers = updated_headers
        };
        !mock_server_started
      } else {
        error!("Interaction is not an HTTP interaction, is {}", inner.type_of());
        false
      }
    }).unwrap_or(false)
  } else {
    warn!("Ignoring header with empty or null name");
    false
  }
}

/// Configures the response for the Interaction. Returns false if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started)
///
/// * `status` - the response status. Defaults to 200.
#[no_mangle]
pub extern fn pactffi_response_status(interaction: InteractionHandle, status: c_ushort) -> bool {
  interaction.with_interaction(&|_, mock_server_started, inner| {
    if let Some(reqres) = inner.as_v4_http_mut() {
      reqres.response.status = status;
      !mock_server_started
    } else {
      error!("Interaction is not an HTTP interaction, is {}", inner.type_of());
      false
    }
  }).unwrap_or(false)
}

/// Adds the body for the interaction. Returns false if the interaction or Pact can't be
/// modified (i.e. the mock server for it has already started)
///
/// * `part` - The part of the interaction to add the body to (Request or Response).
/// * `content_type` - The content type of the body. Defaults to `text/plain`. Will be ignored if a content type
///   header is already set.
/// * `body` - The body contents. For JSON payloads, matching rules can be embedded in the body.
#[no_mangle]
pub extern fn pactffi_with_body(
  interaction: InteractionHandle,
  part: InteractionPart,
  content_type: *const c_char,
  body: *const c_char
) -> bool {
  let content_type = convert_cstr("content_type", content_type).unwrap_or("text/plain");
  let body = convert_cstr("body", body).unwrap_or_default();
  let content_type_header = "Content-Type".to_string();
  interaction.with_interaction(&|_, mock_server_started, inner| {
    if let Some(reqres) = inner.as_v4_http_mut() {
      match part {
        InteractionPart::Request => {
          if !reqres.request.has_header(&content_type_header) {
            match reqres.request.headers {
              Some(ref mut headers) => {
                headers.insert(content_type_header.clone(), vec![content_type.to_string()]);
              },
              None => {
                reqres.request.headers = Some(hashmap! { content_type_header.clone() => vec![ content_type.to_string() ]});
              }
            }
          }
          let body = if reqres.request.content_type().unwrap_or_default().is_json() {
            let category = reqres.request.matching_rules.add_category("body");
            OptionalBody::from(process_json(body.to_string(), category, &mut reqres.request.generators))
          } else if reqres.request.content_type().unwrap_or_default().is_xml() {
            let category = reqres.request.matching_rules.add_category("body");
            OptionalBody::Present(Bytes::from(process_xml(body.to_string(), category, &mut reqres.request.generators).unwrap_or(vec![])),
                                  Some("application/xml".into()), None)
          } else {
            OptionalBody::from(body)
          };
          reqres.request.body = body;
        },
        InteractionPart::Response => {
          if !reqres.response.has_header(&content_type_header) {
            match reqres.response.headers {
              Some(ref mut headers) => {
                headers.insert(content_type_header.clone(), vec![content_type.to_string()]);
              },
              None => {
                reqres.response.headers = Some(hashmap! { content_type_header.clone() => vec![ content_type.to_string() ]});
              }
            }
          }
          let body = if reqres.response.content_type().unwrap_or_default().is_json() {
            let category = reqres.response.matching_rules.add_category("body");
            OptionalBody::from(process_json(body.to_string(), category, &mut reqres.response.generators))
          } else if reqres.response.content_type().unwrap_or_default().is_xml() {
            let category = reqres.request.matching_rules.add_category("body");
            OptionalBody::Present(Bytes::from(process_xml(body.to_string(), category, &mut reqres.request.generators).unwrap_or(vec![])),
                                  Some("application/xml".into()), None)
          } else {
            OptionalBody::from(body)
          };
          reqres.response.body = body;
        }
      };
      !mock_server_started
    } else {
      error!("Interaction is not an HTTP interaction, is {}", inner.type_of());
      false
    }
  }).unwrap_or(false)
}
