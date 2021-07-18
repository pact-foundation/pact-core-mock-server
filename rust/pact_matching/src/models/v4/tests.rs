use std::{env, fs, io};
use std::fs::File;
use std::io::Read;

use expectest::prelude::*;
use maplit::*;
use serde_json::json;

use pact_models::{Consumer, PactSpecification, Provider};
use pact_models::bodies::OptionalBody;
use pact_models::matchingrules;
use pact_models::matchingrules::MatchingRule;
use pact_models::provider_states::ProviderState;
use pact_models::v4::async_message::AsynchronousMessage;
use pact_models::v4::http_parts::{HttpRequest, HttpResponse};
use pact_models::v4::message_parts::MessageContents;
use pact_models::v4::sync_message::SynchronousMessages;
use pact_models::v4::synch_http::SynchronousHttp;
use pact_models::v4::V4InteractionType;

use crate::models::{Pact, PACT_RUST_VERSION, ReadWritePact, write_pact};
use crate::models::v4::{from_json, V4Pact};

#[test]
fn load_empty_pact() {
  let pact_json = json!({});
  let pact = from_json("", &pact_json).unwrap();
  expect!(pact.provider().name).to(be_equal_to("provider"));
  expect!(pact.consumer().name).to(be_equal_to("consumer"));
  expect!(pact.interactions().iter()).to(have_count(0));
  expect!(pact.metadata().iter()).to(have_count(0));
  expect!(pact.specification_version()).to(be_equal_to(PactSpecification::V4));
}

#[test]
fn load_basic_pact() {
  let pact_json = json!({
    "provider": {
        "name": "Alice Service"
    },
    "consumer": {
        "name": "Consumer"
    },
    "interactions": [
      {
        "type": "Synchronous/HTTP",
        "description": "a retrieve Mallory request",
        "request": {
          "method": "GET",
          "path": "/mallory",
          "query": "name=ron&status=good"
        },
        "response": {
          "status": 200,
          "headers": {
            "Content-Type": "text/html"
          },
          "body": {
            "content": "\"That is some good Mallory.\""
          }
        }
      }
    ]
  });
  let pact = from_json("", &pact_json).unwrap();
  expect!(&pact.provider().name).to(be_equal_to("Alice Service"));
  expect!(&pact.consumer().name).to(be_equal_to("Consumer"));
  expect!(pact.interactions().iter()).to(have_count(1));
  let interaction = pact.interactions()[0];
  expect!(interaction.description()).to(be_equal_to("a retrieve Mallory request"));
  expect!(interaction.provider_states().iter()).to(be_empty());
  expect!(pact.specification_version()).to(be_equal_to(PactSpecification::V4));
  expect!(pact.metadata().iter()).to(have_count(0));

  let v4pact = pact.as_v4_pact().unwrap();
  let interaction = &v4pact.interactions[0];
  expect!(interaction.pending()).to(be_false());
  match interaction.as_v4_http() {
    Some(SynchronousHttp { request, response, pending, .. }) => {
      expect!(request).to(be_equal_to(HttpRequest {
        method: "GET".into(),
        path: "/mallory".into(),
        query: Some(hashmap!{ "name".to_string() => vec!["ron".to_string()], "status".to_string() => vec!["good".to_string()] }),
        headers: None,
        body: OptionalBody::Missing,
        .. HttpRequest::default()
      }));
      expect!(response).to(be_equal_to(HttpResponse {
        status: 200,
        headers: Some(hashmap!{ "Content-Type".to_string() => vec!["text/html".to_string()] }),
        body: OptionalBody::Present("\"That is some good Mallory.\"".into(), Some("text/html".into())),
        .. HttpResponse::default()
      }));
      expect!(pending).to(be_false());
    }
    _ => panic!("Was expecting an HTTP pact")
  }
}

#[test]
fn load_pact_encoded_query_string() {
  let pact_json = json!({
      "provider" : {
        "name" : "test_provider"
      },
      "consumer" : {
        "name" : "test_consumer"
      },
      "interactions" : [ {
        "type": "Synchronous/HTTP",
        "description" : "test interaction",
        "request" : {
          "query" : "datetime=2011-12-03T10%3A15%3A30%2B01%3A00&description=hello+world%21"
        },
        "response" : {
          "status" : 200
        }
      } ],
      "metadata" : {
        "pactSpecification" : {
          "version" : "4.0"
        }
      }
    });
  let pact = from_json("", &pact_json).unwrap();

  expect!(pact.interactions().iter()).to(have_count(1));

  let v4pact = pact.as_v4_pact().unwrap();
  match v4pact.interactions[0].as_v4_http() {
    Some(SynchronousHttp { request, .. }) => {
      expect!(&request.query).to(be_equal_to(
        &Some(hashmap!{ "datetime".to_string() => vec!["2011-12-03T10:15:30+01:00".to_string()],
            "description".to_string() => vec!["hello world!".to_string()] })));
    }
    _ => panic!("Was expecting an HTTP pact")
  }
}

#[test]
fn load_pact_converts_methods_to_uppercase() {
  let pact_json = json!({
      "interactions" : [ {
        "type": "Synchronous/HTTP",
        "description" : "test interaction",
        "request" : {
          "method" : "get"
        },
        "response" : {
          "status" : 200
        }
      } ],
      "metadata" : {}
    });
  let pact = from_json("", &pact_json).unwrap();
  expect!(pact.interactions().iter()).to(have_count(1));

  let v4pact = pact.as_v4_pact().unwrap();
  match v4pact.interactions[0].as_v4_http() {
    Some(SynchronousHttp { request, .. }) => {
      expect!(&request.method).to(be_equal_to("GET"));
    }
    _ => panic!("Was expecting an HTTP pact")
  }
}

fn read_pact_file(file: &str) -> io::Result<String> {
  let mut f = File::open(file)?;
  let mut buffer = String::new();
  f.read_to_string(&mut buffer)?;
  Ok(buffer)
}

#[test]
fn write_pact_test() {
  let pact = V4Pact { consumer: Consumer { name: s!("write_pact_test_consumer") },
    provider: Provider { name: s!("write_pact_test_provider") },
    interactions: vec![
      Box::new(SynchronousHttp {
        id: None,
        key: None,
        description: s!("Test Interaction"),
        provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
        .. Default::default()
      })
    ],
    .. V4Pact::default() };
  let mut dir = env::temp_dir();
  let x = rand::random::<u16>();
  dir.push(format!("pact_test_{}", x));
  dir.push(pact.default_file_name());

  let result = write_pact(pact.boxed(), &dir, PactSpecification::V4, true);

  let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or_default();
  fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

  expect!(result).to(be_ok());
  expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "write_pact_test_consumer"
  }},
  "interactions": [
    {{
      "description": "Test Interaction",
      "key": "296966511eff169a",
      "pending": false,
      "providerStates": [
        {{
          "name": "Good state to be in"
        }}
      ],
      "request": {{
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }},
      "type": "Synchronous/HTTP"
    }}
  ],
  "metadata": {{
    "pactRust": {{
      "version": "{}"
    }},
    "pactSpecification": {{
      "version": "4.0"
    }}
  }},
  "provider": {{
    "name": "write_pact_test_provider"
  }}
}}"#, super::PACT_RUST_VERSION.unwrap())));
}

#[test]
fn write_synchronous_message_pact_test() {
  let pact = V4Pact {
    consumer: Consumer { name: "write_pact_test_consumer".into() },
    provider: Provider { name: "write_pact_test_provider".into() },
    interactions: vec![
      Box::new(SynchronousMessages {
        id: None,
        key: None,
        description: "Test Interaction".into(),
        provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap!{} }],
        request: MessageContents { contents: "\"this is a message\"".into(), .. MessageContents::default() },
        response: vec![MessageContents { contents: "\"this is a response\"".into(), .. MessageContents::default() }],
        .. Default::default()
      })
    ],
    .. V4Pact::default() };
  let mut dir = env::temp_dir();
  let x = rand::random::<u16>();
  dir.push(format!("pact_test_{}", x));
  dir.push(pact.default_file_name());

  let result = write_pact(pact.boxed(), &dir, PactSpecification::V4, true);

  let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or_default();
  fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

  expect!(result).to(be_ok());
  expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "write_pact_test_consumer"
  }},
  "interactions": [
    {{
      "description": "Test Interaction",
      "key": "b341297869a4287d",
      "pending": false,
      "providerStates": [
        {{
          "name": "Good state to be in"
        }}
      ],
      "request": {{
        "contents": {{
          "content": "\"this is a message\"",
          "contentType": "*/*",
          "encoded": false
        }}
      }},
      "response": [
        {{
          "contents": {{
            "content": "\"this is a response\"",
            "contentType": "*/*",
            "encoded": false
          }}
        }}
      ],
      "type": "Synchronous/Messages"
    }}
  ],
  "metadata": {{
    "pactRust": {{
      "version": "{}"
    }},
    "pactSpecification": {{
      "version": "4.0"
    }}
  }},
  "provider": {{
    "name": "write_pact_test_provider"
  }}
}}"#, super::PACT_RUST_VERSION.unwrap())));
}

#[test]
fn write_pact_test_should_merge_pacts() {
  let pact = V4Pact {
    consumer: Consumer { name: "merge_consumer".into() },
    provider: Provider { name: "merge_provider".into() },
    interactions: vec![
      Box::new(SynchronousHttp {
        description: "Test Interaction 2".into(),
        provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap!{} }],
        .. SynchronousHttp::default()
      })
    ],
    metadata: btreemap!{}
  };
  let pact2 = V4Pact {
    consumer: Consumer { name: "merge_consumer".into() },
    provider: Provider { name: "merge_provider".into() },
    interactions: vec![
      Box::new(SynchronousHttp {
        description: "Test Interaction".into(),
        provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap!{} }],
        .. SynchronousHttp::default()
      })
    ],
    metadata: btreemap!{}
  };
  let mut dir = env::temp_dir();
  let x = rand::random::<u16>();
  dir.push(format!("pact_test_{}", x));
  dir.push(pact.default_file_name());

  let result = write_pact(pact.boxed(), dir.as_path(), PactSpecification::V4, true);
  let result2 = write_pact(pact2.boxed(), dir.as_path(), PactSpecification::V4, false);

  let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or(s!(""));
  fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

  expect!(result).to(be_ok());
  expect!(result2).to(be_ok());
  expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "merge_consumer"
  }},
  "interactions": [
    {{
      "description": "Test Interaction",
      "key": "296966511eff169a",
      "pending": false,
      "providerStates": [
        {{
          "name": "Good state to be in"
        }}
      ],
      "request": {{
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }},
      "type": "Synchronous/HTTP"
    }},
    {{
      "description": "Test Interaction 2",
      "key": "d3e13a43bc0744ac",
      "pending": false,
      "providerStates": [
        {{
          "name": "Good state to be in"
        }}
      ],
      "request": {{
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }},
      "type": "Synchronous/HTTP"
    }}
  ],
  "metadata": {{
    "pactRust": {{
      "version": "{}"
    }},
    "pactSpecification": {{
      "version": "4.0"
    }}
  }},
  "provider": {{
    "name": "merge_provider"
  }}
}}"#, PACT_RUST_VERSION.unwrap())));
}

#[test]
fn write_pact_test_should_overwrite_pact_with_same_key() {
  let pact = V4Pact {
    consumer: Consumer { name: "write_pact_test_consumer".into() },
    provider: Provider { name: "write_pact_test_provider".into() },
    interactions: vec![
      Box::new(SynchronousHttp {
        description: "Test Interaction".into(),
        key: Some("1234567890".into()),
        provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap!{} }],
        .. SynchronousHttp::default()
      })
    ],
    metadata: btreemap!{}
  };
  let pact2 = V4Pact {
    consumer: Consumer { name: "write_pact_test_consumer".into() },
    provider: Provider { name: "write_pact_test_provider".into() },
    interactions: vec![
      Box::new(SynchronousHttp {
        description: "Test Interaction".into(),
        key: Some("1234567890".into()),
        provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap!{} }],
        response: HttpResponse { status: 400, .. HttpResponse::default() },
        .. SynchronousHttp::default()
      })
    ],
    metadata: btreemap!{}
  };
  let mut dir = env::temp_dir();
  let x = rand::random::<u16>();
  dir.push(format!("pact_test_{}", x));
  dir.push(pact.default_file_name());

  let result = write_pact(pact.boxed(), dir.as_path(), PactSpecification::V4, true);
  let result2 = write_pact(pact2.boxed(), dir.as_path(), PactSpecification::V4, false);

  let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or_default();
  fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

  expect!(result).to(be_ok());
  expect!(result2).to(be_ok());
  expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "write_pact_test_consumer"
  }},
  "interactions": [
    {{
      "description": "Test Interaction",
      "key": "1234567890",
      "pending": false,
      "providerStates": [
        {{
          "name": "Good state to be in"
        }}
      ],
      "request": {{
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 400
      }},
      "type": "Synchronous/HTTP"
    }}
  ],
  "metadata": {{
    "pactRust": {{
      "version": "{}"
    }},
    "pactSpecification": {{
      "version": "4.0"
    }}
  }},
  "provider": {{
    "name": "write_pact_test_provider"
  }}
}}"#, PACT_RUST_VERSION.unwrap())));
}

#[test]
fn pact_merge_does_not_merge_different_consumers() {
  let pact = V4Pact { consumer: Consumer { name: "test_consumer".to_string() },
    provider: Provider { name: "test_provider".to_string() },
    interactions: vec![],
    metadata: btreemap!{}
  };
  let pact2 = V4Pact { consumer: Consumer { name: "test_consumer2".to_string() },
    provider: Provider { name: "test_provider".to_string() },
    interactions: vec![],
    metadata: btreemap!{}
  };
  expect!(pact.merge(&pact2)).to(be_err());
}

#[test]
fn pact_merge_does_not_merge_different_providers() {
  let pact = V4Pact { consumer: Consumer { name: "test_consumer".to_string() },
    provider: Provider { name: "test_provider".to_string() },
    interactions: vec![],
    metadata: btreemap!{}
  };
  let pact2 = V4Pact { consumer: Consumer { name: "test_consumer".to_string() },
    provider: Provider { name: "test_provider2".to_string() },
    interactions: vec![],
    metadata: btreemap!{}
  };
  expect!(pact.merge(&pact2)).to(be_err());
}

// #[test]
// fn pact_merge_does_not_merge_where_there_are_conflicting_interactions() {
//   let pact = RequestResponsePact { consumer: Consumer { name: s!("test_consumer") },
//     provider: Provider { name: s!("test_provider") },
//     interactions: vec![
//       RequestResponseInteraction {
//         description: s!("Test Interaction"),
//         provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
//         .. RequestResponseInteraction::default()
//       }
//     ],
//     metadata: btreemap!{},
//     specification_version: PactSpecification::V1_1
//   };
//   let pact2 = RequestResponsePact { consumer: Consumer { name: s!("test_consumer") },
//     provider: Provider { name: s!("test_provider") },
//     interactions: vec![
//       RequestResponseInteraction {
//         description: s!("Test Interaction"),
//         provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
//         request: Request { path: s!("/other"), .. Request::default() },
//         .. RequestResponseInteraction::default()
//       }
//     ],
//     metadata: btreemap!{},
//     specification_version: PactSpecification::V1_1
//   };
//   expect!(pact.merge(&pact2)).to(be_err());
// }

#[test]
fn pact_merge_removes_duplicates() {
  let pact = V4Pact {
    consumer: Consumer { name: "test_consumer".into() },
    provider: Provider { name: "test_provider".into() },
    interactions: vec![
      Box::new(SynchronousHttp {
        description: "Test Interaction".into(),
        key: Some("1234567890".into()),
        provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap!{} }],
        response: HttpResponse { status: 400, .. HttpResponse::default() },
        .. SynchronousHttp::default()
      })
    ],
    .. V4Pact::default()
  };
  let pact2 = V4Pact {
    consumer: Consumer { name: "test_consumer".into() },
    provider: Provider { name: "test_provider".into() },
    interactions: vec![
      Box::new(SynchronousHttp {
        description: "Test Interaction".into(),
        key: Some("1234567890".into()),
        provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap!{} }],
        response: HttpResponse { status: 400, .. HttpResponse::default() },
        .. SynchronousHttp::default()
      }),
      Box::new(SynchronousHttp {
        description: "Test Interaction 2".into(),
        key: Some("1234567891".into()),
        provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap!{} }],
        response: HttpResponse { status: 400, .. HttpResponse::default() },
        .. SynchronousHttp::default()
      })
    ],
    .. V4Pact::default()
  };

  let merged_pact = pact.merge(&pact2);
  expect!(merged_pact.unwrap().interactions().len()).to(be_equal_to(2));

  let merged_pact2 = pact.merge(&pact.clone());
  expect!(merged_pact2.unwrap().interactions().len()).to(be_equal_to(1));
}

#[test]
fn write_v2_pact_test_with_matchers() {
  let pact = V4Pact {
    consumer: Consumer { name: "write_pact_test_consumer".into() },
    provider: Provider { name: "write_pact_test_provider".into() },
    interactions: vec![
      Box::new(SynchronousHttp {
        description: "Test Interaction".into(),
        key: Some("1234567890".into()),
        provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap!{} }],
        request: HttpRequest {
          matching_rules: matchingrules!{
            "body" => {
              "$" => [ MatchingRule::Type ]
            }
          },
          .. HttpRequest::default()
        },
        .. SynchronousHttp::default()
      })
    ],
    .. V4Pact::default() };

  let mut dir = env::temp_dir();
  let x = rand::random::<u16>();
  dir.push(format!("pact_test_{}", x));
  dir.push(pact.default_file_name());

  let result = write_pact(pact.boxed(), &dir, PactSpecification::V2, true);

  let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or("".to_string());
  fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

  expect!(result).to(be_ok());
  expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "write_pact_test_consumer"
  }},
  "interactions": [
    {{
      "description": "Test Interaction",
      "providerState": "Good state to be in",
      "request": {{
        "matchingRules": {{
          "$.body": {{
            "match": "type"
          }}
        }},
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }}
    }}
  ],
  "metadata": {{
    "pactRust": {{
      "version": "{}"
    }},
    "pactSpecification": {{
      "version": "2.0.0"
    }}
  }},
  "provider": {{
    "name": "write_pact_test_provider"
  }}
}}"#, super::PACT_RUST_VERSION.unwrap())));
}

#[test]
fn write_pact_v3_test_with_matchers() {
  let pact = V4Pact { consumer: Consumer { name: s!("write_pact_test_consumer_v3") },
    provider: Provider { name: s!("write_pact_test_provider_v3") },
    interactions: vec![
      Box::new(SynchronousHttp {
        description: "Test Interaction".into(),
        key: Some("1234567890".into()),
        provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap!{} }],
        request: HttpRequest {
          matching_rules: matchingrules!{
            "body" => {
              "$" => [ MatchingRule::Type ]
            },
            "header" => {
              "HEADER_A" => [ MatchingRule::Include(s!("ValA")), MatchingRule::Include(s!("ValB")) ]
            }
          },
          .. HttpRequest::default()
        },
        .. SynchronousHttp::default()
      })
    ],
    .. V4Pact::default() };
  let mut dir = env::temp_dir();
  let x = rand::random::<u16>();
  dir.push(format!("pact_test_{}", x));
  dir.push(pact.default_file_name());

  let result = write_pact(pact.boxed(), &dir, PactSpecification::V3, true);

  let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or(s!(""));
  fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

  expect!(result).to(be_ok());
  expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "write_pact_test_consumer_v3"
  }},
  "interactions": [
    {{
      "description": "Test Interaction",
      "providerStates": [
        {{
          "name": "Good state to be in"
        }}
      ],
      "request": {{
        "matchingRules": {{
          "body": {{
            "$": {{
              "combine": "AND",
              "matchers": [
                {{
                  "match": "type"
                }}
              ]
            }}
          }},
          "header": {{
            "HEADER_A": {{
              "combine": "AND",
              "matchers": [
                {{
                  "match": "include",
                  "value": "ValA"
                }},
                {{
                  "match": "include",
                  "value": "ValB"
                }}
              ]
            }}
          }}
        }},
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }}
    }}
  ],
  "metadata": {{
    "pactRust": {{
      "version": "{}"
    }},
    "pactSpecification": {{
      "version": "3.0.0"
    }}
  }},
  "provider": {{
    "name": "write_pact_test_provider_v3"
  }}
}}"#, super::PACT_RUST_VERSION.unwrap())));
}

// #[test]
// fn write_v3_pact_test() {
//   let pact = RequestResponsePact { consumer: Consumer { name: s!("write_pact_test_consumer") },
//     provider: Provider { name: s!("write_pact_test_provider") },
//     interactions: vec![
//       RequestResponseInteraction {
//         description: s!("Test Interaction"),
//         provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
//         request: Request {
//           query: Some(hashmap!{
//                         s!("a") => vec![s!("1"), s!("2"), s!("3")],
//                         s!("b") => vec![s!("bill"), s!("bob")],
//                     }),
//           .. Request::default()
//         },
//         .. RequestResponseInteraction::default()
//       }
//     ],
//     .. RequestResponsePact::default() };
//   let mut dir = env::temp_dir();
//   let x = rand::random::<u16>();
//   dir.push(format!("pact_test_{}", x));
//   dir.push(pact.default_file_name());
//
//   let result = pact.write_pact(dir.as_path(), PactSpecification::V3);
//
//   let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or(s!(""));
//   fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());
//
//   expect!(result).to(be_ok());
//   expect!(pact_file).to(be_equal_to(format!(r#"{{
//   "consumer": {{
//     "name": "write_pact_test_consumer"
//   }},
//   "interactions": [
//     {{
//       "description": "Test Interaction",
//       "providerStates": [
//         {{
//           "name": "Good state to be in"
//         }}
//       ],
//       "request": {{
//         "method": "GET",
//         "path": "/",
//         "query": {{
//           "a": [
//             "1",
//             "2",
//             "3"
//           ],
//           "b": [
//             "bill",
//             "bob"
//           ]
//         }}
//       }},
//       "response": {{
//         "status": 200
//       }}
//     }}
//   ],
//   "metadata": {{
//     "pactRust": {{
//       "version": "{}"
//     }},
//     "pactSpecification": {{
//       "version": "3.0.0"
//     }}
//   }},
//   "provider": {{
//     "name": "write_pact_test_provider"
//   }}
// }}"#, super::VERSION.unwrap())));
// }
//
// #[test]
// fn write_pact_test_with_generators() {
//   let pact = RequestResponsePact { consumer: Consumer { name: s!("write_pact_test_consumer") },
//     provider: Provider { name: s!("write_pact_test_provider") },
//     interactions: vec![
//       RequestResponseInteraction {
//         description: s!("Test Interaction with generators"),
//         provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
//         request: Request {
//           generators: generators!{
//                         "BODY" => {
//                           "$" => Generator::RandomInt(1, 10)
//                         },
//                         "HEADER" => {
//                           "A" => Generator::RandomString(20)
//                         }
//                     },
//           .. Request::default()
//         },
//         .. RequestResponseInteraction::default()
//       }
//     ],
//     .. RequestResponsePact::default() };
//   let mut dir = env::temp_dir();
//   let x = rand::random::<u16>();
//   dir.push(format!("pact_test_{}", x));
//   dir.push(pact.default_file_name());
//
//   let result = pact.write_pact(dir.as_path(), PactSpecification::V3);
//
//   let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or(s!(""));
//   fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());
//
//   expect!(result).to(be_ok());
//   expect!(pact_file).to(be_equal_to(format!(r#"{{
//   "consumer": {{
//     "name": "write_pact_test_consumer"
//   }},
//   "interactions": [
//     {{
//       "description": "Test Interaction with generators",
//       "providerStates": [
//         {{
//           "name": "Good state to be in"
//         }}
//       ],
//       "request": {{
//         "generators": {{
//           "body": {{
//             "$": {{
//               "max": 10,
//               "min": 1,
//               "type": "RandomInt"
//             }}
//           }},
//           "header": {{
//             "A": {{
//               "size": 20,
//               "type": "RandomString"
//             }}
//           }}
//         }},
//         "method": "GET",
//         "path": "/"
//       }},
//       "response": {{
//         "status": 200
//       }}
//     }}
//   ],
//   "metadata": {{
//     "pactRust": {{
//       "version": "{}"
//     }},
//     "pactSpecification": {{
//       "version": "3.0.0"
//     }}
//   }},
//   "provider": {{
//     "name": "write_pact_test_provider"
//   }}
// }}"#, super::VERSION.unwrap())));
// }

#[test]
fn write_v4_pact_test_with_comments() {
  let pact = V4Pact { consumer: Consumer { name: s!("write_v4pact_test_consumer") },
    provider: Provider { name: "write_v4pact_test_provider".into() },
    interactions: vec![
      Box::new(SynchronousHttp {
        id: None,
        key: None,
        description: "Test Interaction".into(),
        comments: hashmap! {
          "text".to_string() => json!([
            "This allows me to specify just a bit more information about the interaction",
            "It has no functional impact, but can be displayed in the broker HTML page, and potentially in the test output",
            "It could even contain the name of the running test on the consumer side to help marry the interactions back to the test case"
          ]),
          "testname".to_string() => json!("example_test.groovy")
        },
        .. Default::default()
      })
    ],
    .. V4Pact::default() };
  let mut dir = env::temp_dir();
  let x = rand::random::<u16>();
  dir.push(format!("pact_test_{}", x));
  dir.push(pact.default_file_name());

  let result = write_pact(pact.boxed(), &dir, PactSpecification::V4, true);

  let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or_default();
  fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

  expect!(result).to(be_ok());
  expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "write_v4pact_test_consumer"
  }},
  "interactions": [
    {{
      "comments": {{
        "testname": "example_test.groovy",
        "text": [
          "This allows me to specify just a bit more information about the interaction",
          "It has no functional impact, but can be displayed in the broker HTML page, and potentially in the test output",
          "It could even contain the name of the running test on the consumer side to help marry the interactions back to the test case"
        ]
      }},
      "description": "Test Interaction",
      "key": "7e202f73d7d6d607",
      "pending": false,
      "request": {{
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }},
      "type": "Synchronous/HTTP"
    }}
  ],
  "metadata": {{
    "pactRust": {{
      "version": "{}"
    }},
    "pactSpecification": {{
      "version": "4.0"
    }}
  }},
  "provider": {{
    "name": "write_v4pact_test_provider"
  }}
}}"#, super::PACT_RUST_VERSION.unwrap())));
}

#[test]
fn has_interactions_test() {
  let pact1 = V4Pact {
    interactions: vec![],
    .. V4Pact::default() };
  let pact2 = V4Pact {
    interactions: vec![
      Box::new(SynchronousHttp::default())
    ],
    .. V4Pact::default() };
  let pact3 = V4Pact {
    interactions: vec![
      Box::new(AsynchronousMessage::default())
    ],
    .. V4Pact::default() };
  let pact4 = V4Pact {
    interactions: vec![
      Box::new(SynchronousMessages::default())
    ],
    .. V4Pact::default() };
  let pact5 = V4Pact {
    interactions: vec![
      Box::new(SynchronousHttp::default()),
      Box::new(SynchronousMessages::default())
    ],
    .. V4Pact::default() };

  expect!(pact1.has_interactions(V4InteractionType::Synchronous_HTTP)).to(be_false());
  expect!(pact1.has_interactions(V4InteractionType::Asynchronous_Messages)).to(be_false());
  expect!(pact1.has_interactions(V4InteractionType::Synchronous_Messages)).to(be_false());

  expect!(pact2.has_interactions(V4InteractionType::Synchronous_HTTP)).to(be_true());
  expect!(pact2.has_interactions(V4InteractionType::Asynchronous_Messages)).to(be_false());
  expect!(pact2.has_interactions(V4InteractionType::Synchronous_Messages)).to(be_false());

  expect!(pact3.has_interactions(V4InteractionType::Synchronous_HTTP)).to(be_false());
  expect!(pact3.has_interactions(V4InteractionType::Asynchronous_Messages)).to(be_true());
  expect!(pact3.has_interactions(V4InteractionType::Synchronous_Messages)).to(be_false());

  expect!(pact4.has_interactions(V4InteractionType::Synchronous_HTTP)).to(be_false());
  expect!(pact4.has_interactions(V4InteractionType::Asynchronous_Messages)).to(be_false());
  expect!(pact4.has_interactions(V4InteractionType::Synchronous_Messages)).to(be_true());

  expect!(pact5.has_interactions(V4InteractionType::Synchronous_HTTP)).to(be_true());
  expect!(pact5.has_interactions(V4InteractionType::Asynchronous_Messages)).to(be_false());
  expect!(pact5.has_interactions(V4InteractionType::Synchronous_Messages)).to(be_true());
}

#[test]
fn has_mixed_interactions_test() {
  let pact1 = V4Pact {
    interactions: vec![],
    .. V4Pact::default() };
  let pact2 = V4Pact {
    interactions: vec![
      Box::new(SynchronousHttp::default())
    ],
    .. V4Pact::default() };
  let pact3 = V4Pact {
    interactions: vec![
      Box::new(AsynchronousMessage::default())
    ],
    .. V4Pact::default() };
  let pact4 = V4Pact {
    interactions: vec![
      Box::new(SynchronousMessages::default())
    ],
    .. V4Pact::default() };
  let pact5 = V4Pact {
    interactions: vec![
      Box::new(SynchronousHttp::default()),
      Box::new(SynchronousMessages::default())
    ],
    .. V4Pact::default() };

  expect!(pact1.has_mixed_interactions()).to(be_false());
  expect!(pact2.has_mixed_interactions()).to(be_false());
  expect!(pact3.has_mixed_interactions()).to(be_false());
  expect!(pact4.has_mixed_interactions()).to(be_false());
  expect!(pact5.has_mixed_interactions()).to(be_true());
}

#[test]
fn load_pending_pact() {
  let pact_json = json!({
      "interactions" : [ {
        "type": "Synchronous/HTTP",
        "description" : "test interaction",
        "pending": true,
        "request" : {
          "method" : "get"
        },
        "response" : {
          "status" : 200
        }
      } ],
      "metadata" : {}
    });
  let pact = from_json("", &pact_json).unwrap();
  expect!(pact.interactions().iter()).to(have_count(1));

  let v4pact = pact.as_v4_pact().unwrap();
  let interaction = &v4pact.interactions[0];
  expect(interaction.pending()).to(be_true());
  match interaction.as_v4_http() {
    Some(SynchronousHttp { request, .. }) => {
      expect!(&request.method).to(be_equal_to("GET"));
    }
    _ => panic!("Was expecting an HTTP pact")
  }
}
