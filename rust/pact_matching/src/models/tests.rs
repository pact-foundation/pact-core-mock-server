use std::env;
use std::fs::{self, File};
use std::io;

use expectest::expect;
use expectest::prelude::*;
use maplit::*;
use rand;
use serde_json::json;

use pact_models::bodies::OptionalBody;
use pact_models::content_types::JSON;
use pact_models::generators;
use pact_models::generators::Generator;
use pact_models::matchingrules;
use pact_models::matchingrules::MatchingRule;
use pact_models::provider_states::*;
use pact_models::request::Request;
use pact_models::response::Response;
use pact_models::v4::synch_http::SynchronousHttp;

use super::*;

#[test]
fn load_empty_pact() {
    let pact_json = r#"{}"#;
    let pact = RequestResponsePact::from_json(&s!(""), &serde_json::from_str(pact_json).unwrap());
    expect!(pact.provider.name).to(be_equal_to("provider"));
    expect!(pact.consumer.name).to(be_equal_to("consumer"));
    expect!(pact.interactions.iter()).to(have_count(0));
    expect!(pact.metadata.iter()).to(have_count(0));
    expect!(pact.specification_version).to(be_equal_to(PactSpecification::V3));
}

#[test]
fn missing_metadata() {
    let pact_json = r#"{}"#;
    let pact = RequestResponsePact::from_json(&s!(""), &serde_json::from_str(pact_json).unwrap());
    expect!(pact.specification_version).to(be_equal_to(PactSpecification::V3));
}

#[test]
fn missing_spec_version() {
    let pact_json = r#"{
        "metadata" : {
        }
    }"#;
    let pact = RequestResponsePact::from_json(&s!(""), &serde_json::from_str(pact_json).unwrap());
    expect!(pact.specification_version).to(be_equal_to(PactSpecification::V3));
}

#[test]
fn missing_version_in_spec_version() {
    let pact_json = r#"{
        "metadata" : {
            "pact-specification": {

            }
        }
    }"#;
    let pact = RequestResponsePact::from_json(&s!(""), &serde_json::from_str(pact_json).unwrap());
    expect!(pact.specification_version).to(be_equal_to(PactSpecification::V3));
}

#[test]
fn empty_version_in_spec_version() {
    let pact_json = r#"{
        "metadata" : {
            "pact-specification": {
                "version": ""
            }
        }
    }"#;
    let pact = RequestResponsePact::from_json(&s!(""), &serde_json::from_str(pact_json).unwrap());
    expect!(pact.specification_version).to(be_equal_to(PactSpecification::Unknown));
}

#[test]
fn correct_version_in_spec_version() {
    let pact_json = r#"{
        "metadata" : {
            "pact-specification": {
                "version": "1.0.0"
            }
        }
    }"#;
    let pact = RequestResponsePact::from_json(&s!(""), &serde_json::from_str(pact_json).unwrap());
    expect!(pact.specification_version).to(be_equal_to(PactSpecification::V1));
}

#[test]
fn invalid_version_in_spec_version() {
    let pact_json = r#"{
        "metadata" : {
            "pact-specification": {
                "version": "znjclkazjs"
            }
        }
    }"#;
    let pact = RequestResponsePact::from_json(&s!(""), &serde_json::from_str(pact_json).unwrap());
    expect!(pact.specification_version).to(be_equal_to(PactSpecification::Unknown));
}


#[test]
fn load_basic_pact() {
    let pact_json = r#"
    {
        "provider": {
            "name": "Alice Service"
        },
        "consumer": {
            "name": "Consumer"
        },
        "interactions": [
          {
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
                "body": "\"That is some good Mallory.\""
              }
          }
        ]
    }
    "#;
    let pact = RequestResponsePact::from_json(&s!(""), &serde_json::from_str(pact_json).unwrap());
    expect!(&pact.provider.name).to(be_equal_to("Alice Service"));
    expect!(&pact.consumer.name).to(be_equal_to("Consumer"));
    expect!(pact.interactions.iter()).to(have_count(1));
    let interaction = pact.interactions[0].clone();
    expect!(interaction.description).to(be_equal_to("a retrieve Mallory request"));
    expect!(interaction.provider_states.iter()).to(be_empty());
    expect!(interaction.request).to(be_equal_to(Request {
        method: s!("GET"),
        path: s!("/mallory"),
        query: Some(hashmap!{ s!("name") => vec![s!("ron")], s!("status") => vec![s!("good")] }),
        headers: None,
        body: OptionalBody::Missing,
      .. Request::default()
    }));
    expect!(interaction.response).to(be_equal_to(Response {
        status: 200,
        headers: Some(hashmap!{ s!("Content-Type") => vec![s!("text/html")] }),
        body: OptionalBody::Present("\"That is some good Mallory.\"".into(), Some("text/html".into())),
      .. Response::default()
    }));
    expect!(pact.specification_version).to(be_equal_to(PactSpecification::V3));
    expect!(pact.metadata.iter()).to(have_count(0));
}

#[test]
fn load_pact() {
    let pact_json = r#"
    {
      "provider" : {
        "name" : "test_provider"
      },
      "consumer" : {
        "name" : "test_consumer"
      },
      "interactions" : [ {
        "providerState" : "test state",
        "description" : "test interaction",
        "request" : {
          "method" : "GET",
          "path" : "/",
          "headers" : {
            "testreqheader" : "testreqheadervalue"
          },
          "query" : "q=p&q=p2&r=s",
          "body" : {
            "test" : true
          }
        },
        "response" : {
          "status" : 200,
          "headers" : {
            "testreqheader" : "testreqheaderval"
          },
          "body" : {
            "responsetest" : true
          }
        }
      } ],
      "metadata" : {
        "pact-specification" : {
          "version" : "1.0.0"
        },
        "pact-jvm" : {
          "version" : ""
        }
      }
    }
    "#;
    let pact = RequestResponsePact::from_json(&s!(""), &serde_json::from_str(pact_json).unwrap());
    expect!(&pact.provider.name).to(be_equal_to("test_provider"));
    expect!(&pact.consumer.name).to(be_equal_to("test_consumer"));
    expect!(pact.metadata.iter()).to(have_count(2));
    expect!(&pact.metadata["pactSpecification"]["version"]).to(be_equal_to("1.0.0"));
    expect!(pact.specification_version).to(be_equal_to(PactSpecification::V1));
    expect!(pact.interactions.iter()).to(have_count(1));
    let interaction = pact.interactions[0].clone();
    expect!(interaction.description).to(be_equal_to("test interaction"));
    expect!(interaction.provider_states).to(be_equal_to(vec![
        ProviderState { name: s!("test state"), params: hashmap!{} } ]));
    expect!(interaction.request).to(be_equal_to(Request {
        method: s!("GET"),
        path: s!("/"),
        query: Some(hashmap!{ s!("q") => vec![s!("p"), s!("p2")], s!("r") => vec![s!("s")] }),
        headers: Some(hashmap!{ s!("testreqheader") => vec![s!("testreqheadervalue")] }),
        body: "{\"test\":true}".into(),
      .. Request::default()
    }));
    expect!(interaction.response).to(be_equal_to(Response {
        status: 200,
        headers: Some(hashmap!{ s!("testreqheader") => vec![s!("testreqheaderval")] }),
        body: "{\"responsetest\":true}".into(),
        .. Response::default()
    }));
}

#[test]
fn load_v3_pact() {
    let pact_json = r#"
    {
      "provider" : {
        "name" : "test_provider"
      },
      "consumer" : {
        "name" : "test_consumer"
      },
      "interactions" : [ {
        "providerState" : "test state",
        "description" : "test interaction",
        "request" : {
          "method" : "GET",
          "path" : "/",
          "headers" : {
            "testreqheader" : "testreqheadervalue"
          },
          "query" : {
              "q": ["p", "p2"],
              "r": ["s"]
          },
          "body" : {
            "test" : true
          }
        },
        "response" : {
          "status" : 200,
          "headers" : {
            "testreqheader" : "testreqheaderval"
          },
          "body" : {
            "responsetest" : true
          }
        }
      } ],
      "metadata" : {
        "pact-specification" : {
          "version" : "3.0.0"
        },
        "pact-jvm" : {
          "version" : ""
        }
      }
    }
    "#;
    let pact = RequestResponsePact::from_json(&s!(""), &serde_json::from_str(pact_json).unwrap());
    expect!(&pact.provider.name).to(be_equal_to("test_provider"));
    expect!(&pact.consumer.name).to(be_equal_to("test_consumer"));
    expect!(pact.metadata.iter()).to(have_count(2));
    expect!(&pact.metadata["pactSpecification"]["version"]).to(be_equal_to("3.0.0"));
    expect!(pact.specification_version).to(be_equal_to(PactSpecification::V3));
    expect!(pact.interactions.iter()).to(have_count(1));
    let interaction = pact.interactions[0].clone();
    expect!(interaction.description).to(be_equal_to("test interaction"));
    expect!(interaction.provider_states).to(be_equal_to(vec![
        ProviderState { name: s!("test state"), params: hashmap!{} } ]));
    expect!(interaction.request).to(be_equal_to(Request {
        method: s!("GET"),
        path: s!("/"),
        query: Some(hashmap!{ s!("q") => vec![s!("p"), s!("p2")], s!("r") => vec![s!("s")] }),
        headers: Some(hashmap!{ s!("testreqheader") => vec![s!("testreqheadervalue")] }),
        body: OptionalBody::Present("{\"test\":true}".into(), None),
      .. Request::default()
    }));
    expect!(interaction.response).to(be_equal_to(Response {
        status: 200,
        headers: Some(hashmap!{ s!("testreqheader") => vec![s!("testreqheaderval")] }),
        body: OptionalBody::Present("{\"responsetest\":true}".into(), None),
        .. Response::default()
    }));
}

#[test]
fn load_pact_encoded_query_string() {
    let pact_json = r#"
    {
      "provider" : {
        "name" : "test_provider"
      },
      "consumer" : {
        "name" : "test_consumer"
      },
      "interactions" : [ {
        "providerState" : "test state",
        "description" : "test interaction",
        "request" : {
          "method" : "GET",
          "path" : "/",
          "headers" : {
            "testreqheader" : "testreqheadervalue"
          },
          "query" : "datetime=2011-12-03T10%3A15%3A30%2B01%3A00&description=hello+world%21",
          "body" : {
            "test" : true
          }
        },
        "response" : {
          "status" : 200,
          "headers" : {
            "testreqheader" : "testreqheaderval"
          },
          "body" : {
            "responsetest" : true
          }
        }
      } ],
      "metadata" : {
        "pact-specification" : {
          "version" : "2.0.0"
        },
        "pact-jvm" : {
          "version" : ""
        }
      }
    }
    "#;
    let pact = RequestResponsePact::from_json(&s!(""), &serde_json::from_str(pact_json).unwrap());
    expect!(pact.interactions.iter()).to(have_count(1));
    let interaction = pact.interactions[0].clone();
    expect!(interaction.request).to(be_equal_to(Request {
        method: s!("GET"),
        path: s!("/"),
        query: Some(hashmap!{ s!("datetime") => vec![s!("2011-12-03T10:15:30+01:00")],
            s!("description") => vec![s!("hello world!")] }),
        headers: Some(hashmap!{ s!("testreqheader") => vec![s!("testreqheadervalue")] }),
        body: OptionalBody::Present("{\"test\":true}".into(), None),
      .. Request::default()
    }));
}

#[test]
fn load_pact_converts_methods_to_uppercase() {
    let pact_json = r#"
    {
      "interactions" : [ {
        "description" : "test interaction",
        "request" : {
          "method" : "get"
        },
        "response" : {
          "status" : 200
        }
      } ],
      "metadata" : {}
    }
    "#;
    let pact = RequestResponsePact::from_json(&s!(""), &serde_json::from_str(pact_json).unwrap());
    expect!(pact.interactions.iter()).to(have_count(1));
    let interaction = pact.interactions[0].clone();
    expect!(interaction.request).to(be_equal_to(Request {
        method: s!("GET"),
        path: s!("/"),
        query: None,
        headers: None,
        body: OptionalBody::Missing,
      .. Request::default()
    }));
}

#[test]
fn default_file_name_is_based_in_the_consumer_and_provider() {
    let pact = RequestResponsePact { consumer: Consumer { name: s!("consumer") },
        provider: Provider { name: s!("provider") },
        interactions: vec![],
        metadata: btreemap!{},
        specification_version: PactSpecification::V1_1
    };
    expect!(pact.default_file_name()).to(be_equal_to("consumer-provider.json"));
}

fn read_pact_file(file: &str) -> io::Result<String> {
    let mut f = File::open(file)?;
    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;
    Ok(buffer)
}

#[test]
fn write_pact_test() {
    let pact = RequestResponsePact { consumer: Consumer { name: s!("write_pact_test_consumer") },
        provider: Provider { name: s!("write_pact_test_provider") },
        interactions: vec![
            RequestResponseInteraction {
                description: s!("Test Interaction"),
                provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
                .. RequestResponseInteraction::default()
            }
        ],
        .. RequestResponsePact::default() };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), dir.as_path(), PactSpecification::V2, true);

    let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or(s!(""));
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
fn write_pact_test_should_merge_pacts() {
    let pact = RequestResponsePact { consumer: Consumer { name: s!("merge_consumer") },
        provider: Provider { name: s!("merge_provider") },
        interactions: vec![
            RequestResponseInteraction {
                description: s!("Test Interaction 2"),
                provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
                .. RequestResponseInteraction::default()
            }
        ],
        metadata: btreemap!{},
        specification_version: PactSpecification::V1_1
    };
    let pact2 = RequestResponsePact { consumer: Consumer { name: s!("merge_consumer") },
        provider: Provider { name: s!("merge_provider") },
        interactions: vec![
            RequestResponseInteraction {
                description: s!("Test Interaction"),
                provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
                .. RequestResponseInteraction::default()
            }
        ],
        metadata: btreemap!{},
        specification_version: PactSpecification::V1_1
    };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), dir.as_path(), PactSpecification::V2, false);
    let result2 = write_pact(pact2.boxed(), dir.as_path(), PactSpecification::V2, false);

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
      "providerState": "Good state to be in",
      "request": {{
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }}
    }},
    {{
      "description": "Test Interaction 2",
      "providerState": "Good state to be in",
      "request": {{
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
    "name": "merge_provider"
  }}
}}"#, super::PACT_RUST_VERSION.unwrap())));
}

#[test]
fn write_pact_test_should_not_merge_pacts_with_conflicts() {
    let pact = RequestResponsePact { consumer: Consumer { name: s!("write_pact_test_consumer") },
        provider: Provider { name: s!("write_pact_test_provider") },
        interactions: vec![
            RequestResponseInteraction {
                description: s!("Test Interaction"),
                provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
                .. RequestResponseInteraction::default()
            }
        ],
        metadata: btreemap!{},
        specification_version: PactSpecification::V1_1
    };
    let pact2 = RequestResponsePact { consumer: Consumer { name: s!("write_pact_test_consumer") },
        provider: Provider { name: s!("write_pact_test_provider") },
        interactions: vec![
            RequestResponseInteraction {
                description: s!("Test Interaction"),
                provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
                response: Response { status: 400, .. Response::default() },
                .. RequestResponseInteraction::default()
            }
        ],
        metadata: btreemap!{},
        specification_version: PactSpecification::V1_1
    };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), dir.as_path(), PactSpecification::V2, false);
    let result2 = write_pact(pact2.boxed(), dir.as_path(), PactSpecification::V2, false);

    let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or(s!(""));
    fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

    expect!(result).to(be_ok());
    expect!(result2).to(be_err());
    expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "write_pact_test_consumer"
  }},
  "interactions": [
    {{
      "description": "Test Interaction",
      "providerState": "Good state to be in",
      "request": {{
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
fn write_pact_test_should_upgrade_older_pacts_when_merging() {
    let pact = RequestResponsePact { consumer: Consumer { name: s!("merge_consumer") },
        provider: Provider { name: s!("merge_provider") },
        interactions: vec![
            RequestResponseInteraction {
                description: s!("Test Interaction 2"),
                provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
                .. RequestResponseInteraction::default()
            }
        ],
        metadata: btreemap!{},
        specification_version: PactSpecification::V1_1
    };
    let pact2 = RequestResponsePact { consumer: Consumer { name: s!("merge_consumer") },
        provider: Provider { name: s!("merge_provider") },
        interactions: vec![
            RequestResponseInteraction {
                description: s!("Test Interaction"),
                provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
                .. RequestResponseInteraction::default()
            }
        ],
        metadata: btreemap!{},
        specification_version: PactSpecification::V3
    };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), dir.as_path(), PactSpecification::V2, false);
    let result2 = write_pact(pact2.boxed(), dir.as_path(), PactSpecification::V3, false);

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
      }}
    }},
    {{
      "description": "Test Interaction 2",
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
    "name": "merge_provider"
  }}
}}"#, super::PACT_RUST_VERSION.unwrap())));
}

#[test]
fn write_pact_test_upgrades_older_pacts_to_v4_when_merging() {
  let pact = RequestResponsePact {
    consumer: Consumer { name: "merge_consumer".into() },
    provider: Provider { name: "merge_provider".into() },
    interactions: vec![
      RequestResponseInteraction {
        description: "Test Interaction 2".into(),
        provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap! {} }],
        ..RequestResponseInteraction::default()
      }
    ],
    metadata: btreemap! {},
    specification_version: PactSpecification::V1_1,
  };
  let pact2 = V4Pact {
    consumer: Consumer { name: "merge_consumer".into() },
    provider: Provider { name: "merge_provider".into() },
    interactions: vec![
      Box::new(SynchronousHttp {
        id: None,
        key: None,
        description: "Test Interaction".into(),
        provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap! {} }],
        .. Default::default()
      })
    ],
    metadata: btreemap! {},
  };
  let mut dir = env::temp_dir();
  let x = rand::random::<u16>();
  dir.push(format!("pact_test_{}", x));
  dir.push(pact.default_file_name());

  let result = write_pact(pact.boxed(), dir.as_path(), PactSpecification::V3, false);
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
}}"#, super::PACT_RUST_VERSION.unwrap())));
}

#[test]
fn pact_merge_does_not_merge_different_consumers() {
    let pact = RequestResponsePact { consumer: Consumer { name: s!("test_consumer") },
        provider: Provider { name: s!("test_provider") },
        interactions: vec![],
        metadata: btreemap!{},
        specification_version: PactSpecification::V1
    };
    let pact2 = RequestResponsePact { consumer: Consumer { name: s!("test_consumer2") },
        provider: Provider { name: s!("test_provider") },
        interactions: vec![],
        metadata: btreemap!{},
        specification_version: PactSpecification::V1_1
    };
    expect!(pact.merge(&pact2)).to(be_err());
}

#[test]
fn pact_merge_does_not_merge_different_providers() {
    let pact = RequestResponsePact { consumer: Consumer { name: s!("test_consumer") },
        provider: Provider { name: s!("test_provider") },
        interactions: vec![],
        metadata: btreemap!{},
        specification_version: PactSpecification::V1_1
    };
    let pact2 = RequestResponsePact { consumer: Consumer { name: s!("test_consumer") },
        provider: Provider { name: s!("test_provider2") },
        interactions: vec![],
        metadata: btreemap!{},
        specification_version: PactSpecification::V1_1
    };
    expect!(pact.merge(&pact2)).to(be_err());
}

#[test]
fn pact_merge_does_not_merge_where_there_are_conflicting_interactions() {
    let pact = RequestResponsePact { consumer: Consumer { name: s!("test_consumer") },
        provider: Provider { name: s!("test_provider") },
        interactions: vec![
            RequestResponseInteraction {
                description: s!("Test Interaction"),
                provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
                .. RequestResponseInteraction::default()
            }
        ],
        metadata: btreemap!{},
        specification_version: PactSpecification::V1_1
    };
    let pact2 = RequestResponsePact { consumer: Consumer { name: s!("test_consumer") },
        provider: Provider { name: s!("test_provider") },
        interactions: vec![
            RequestResponseInteraction {
                description: s!("Test Interaction"),
                provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
                request: Request { path: s!("/other"), .. Request::default() },
                .. RequestResponseInteraction::default()
            }
        ],
        metadata: btreemap!{},
        specification_version: PactSpecification::V1_1
    };
    expect!(pact.merge(&pact2)).to(be_err());
}

#[test]
fn pact_merge_removes_duplicates() {
    let pact = RequestResponsePact { consumer: Consumer { name: s!("test_consumer") },
        provider: Provider { name: s!("test_provider") },
        interactions: vec![
            RequestResponseInteraction {
                description: s!("Test Interaction"),
                provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
                .. RequestResponseInteraction::default()
            }
        ],
        .. RequestResponsePact::default()
    };
    let pact2 = RequestResponsePact { consumer: Consumer { name: s!("test_consumer") },
        provider: Provider { name: s!("test_provider") },
        interactions: vec![
            RequestResponseInteraction {
                description: s!("Test Interaction"),
                provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
                .. RequestResponseInteraction::default()
            },
            RequestResponseInteraction {
                description: s!("Test Interaction 2"),
                provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
                .. RequestResponseInteraction::default()
            }
        ],
        .. RequestResponsePact::default()
    };

    let merged_pact = pact.merge(&pact2);
    expect!(merged_pact.unwrap().interactions().len()).to(be_equal_to(2));

    let merged_pact2 = pact.merge(&pact.clone());
    expect!(merged_pact2.unwrap().interactions().len()).to(be_equal_to(1));
}

#[test]
fn write_pact_test_with_matchers() {
    let pact = RequestResponsePact { consumer: Consumer { name: s!("write_pact_test_consumer") },
        provider: Provider { name: s!("write_pact_test_provider") },
        interactions: vec![
            RequestResponseInteraction {
                description: s!("Test Interaction"),
                provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
                request: Request {
                    matching_rules: matchingrules!{
                        "body" => {
                            "$" => [ MatchingRule::Type ]
                        }
                    },
                    .. Request::default()
                },
                .. RequestResponseInteraction::default()
            }
        ],
        .. RequestResponsePact::default() };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), dir.as_path(), PactSpecification::V2, true);

    let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or(s!(""));
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
    let pact = RequestResponsePact { consumer: Consumer { name: s!("write_pact_test_consumer_v3") },
        provider: Provider { name: s!("write_pact_test_provider_v3") },
        interactions: vec![
            RequestResponseInteraction {
            description: s!("Test Interaction"),
            provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
            request: Request {
                matching_rules: matchingrules!{
                        "body" => {
                            "$" => [ MatchingRule::Type ]
                        },
                        "header" => {
                          "HEADER_A" => [ MatchingRule::Include(s!("ValA")), MatchingRule::Include(s!("ValB")) ]
                        }
                    },
                .. Request::default()
            },
            .. RequestResponseInteraction::default()
        }
        ],
        .. RequestResponsePact::default() };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), dir.as_path(), PactSpecification::V3, true);

    let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or(s!(""));
    fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

    expect!(result).to(be_ok());
    expect!(pact_file.parse::<Value>().unwrap()).to(be_equal_to(json!({
      "consumer": {
        "name": "write_pact_test_consumer_v3"
      },
      "interactions": [
        {
          "description": "Test Interaction",
          "providerStates": [
            {
              "name": "Good state to be in"
            }
          ],
          "request": {
            "matchingRules": {
              "body": {
                "$": {
                  "combine": "AND",
                  "matchers": [
                    {
                      "match": "type"
                    }
                  ]
                }
              },
              "header": {
                "HEADER_A": {
                  "combine": "AND",
                  "matchers": [
                    {
                      "match": "include",
                      "value": "ValA"
                    },
                    {
                      "match": "include",
                      "value": "ValB"
                    }
                  ]
                }
              }
            },
            "method": "GET",
            "path": "/"
          },
          "response": {
            "status": 200
          }
        }
      ],
      "metadata": {
        "pactRust": {
          "version": super::PACT_RUST_VERSION
        },
        "pactSpecification": {
          "version": "3.0.0"
        }
      },
      "provider": {
        "name": "write_pact_test_provider_v3"
      }
    })));
}

#[test]
fn write_v3_pact_test() {
    let pact = RequestResponsePact { consumer: Consumer { name: s!("write_pact_test_consumer") },
        provider: Provider { name: s!("write_pact_test_provider") },
        interactions: vec![
            RequestResponseInteraction {
                description: s!("Test Interaction"),
                provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
                request: Request {
                    query: Some(hashmap!{
                        s!("a") => vec![s!("1"), s!("2"), s!("3")],
                        s!("b") => vec![s!("bill"), s!("bob")],
                    }),
                    .. Request::default()
                },
                .. RequestResponseInteraction::default()
            }
        ],
        .. RequestResponsePact::default() };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), dir.as_path(), PactSpecification::V3, true);

    let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or(s!(""));
    fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

    expect!(result).to(be_ok());
    expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "write_pact_test_consumer"
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
        "method": "GET",
        "path": "/",
        "query": {{
          "a": [
            "1",
            "2",
            "3"
          ],
          "b": [
            "bill",
            "bob"
          ]
        }}
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
    "name": "write_pact_test_provider"
  }}
}}"#, super::PACT_RUST_VERSION.unwrap())));
}

#[test]
fn write_pact_test_with_generators() {
    let pact = RequestResponsePact { consumer: Consumer { name: s!("write_pact_test_consumer") },
        provider: Provider { name: s!("write_pact_test_provider") },
        interactions: vec![
            RequestResponseInteraction {
                description: s!("Test Interaction with generators"),
                provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
                request: Request {
                    generators: generators!{
                        "BODY" => {
                          "$" => Generator::RandomInt(1, 10)
                        },
                        "HEADER" => {
                          "A" => Generator::RandomString(20)
                        }
                    },
                    .. Request::default()
                },
                .. RequestResponseInteraction::default()
            }
        ],
        .. RequestResponsePact::default() };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), dir.as_path(), PactSpecification::V3, true);

    let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or(s!(""));
    fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

    expect!(result).to(be_ok());
    expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "write_pact_test_consumer"
  }},
  "interactions": [
    {{
      "description": "Test Interaction with generators",
      "providerStates": [
        {{
          "name": "Good state to be in"
        }}
      ],
      "request": {{
        "generators": {{
          "body": {{
            "$": {{
              "max": 10,
              "min": 1,
              "type": "RandomInt"
            }}
          }},
          "header": {{
            "A": {{
              "size": 20,
              "type": "RandomString"
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
    "name": "write_pact_test_provider"
  }}
}}"#, super::PACT_RUST_VERSION.unwrap())));
}

#[test]
fn merge_pact_test() {
  let pact = RequestResponsePact {
    interactions: vec![
      RequestResponseInteraction {
        description: s!("Test Interaction with matcher"),
        request: Request {
          body: OptionalBody::Present(json!({ "related": [1, 2, 3] }).to_string().into(), Some(JSON.clone())),
          matching_rules: matchingrules!{
            "body" => {
              "$.related" => [ MatchingRule::MinMaxType(0, 5) ]
            }
          },
          .. Request::default()
        },
        .. RequestResponseInteraction::default()
      }
    ],
    .. RequestResponsePact::default() };
  let updated_pact = RequestResponsePact {
    interactions: vec![
      RequestResponseInteraction {
        description: s!("Test Interaction with matcher"),
        request: Request {
          body: OptionalBody::Present(json!({ "related": [1, 2, 3] }).to_string().into(), Some(JSON.clone())),
          matching_rules: matchingrules!{
            "body" => {
              "$.related" => [ MatchingRule::MinMaxType(1, 10) ]
            }
          },
          .. Request::default()
        },
        .. RequestResponseInteraction::default()
      }
    ],
    .. RequestResponsePact::default() };
  let merged_pact = pact.merge(&updated_pact);
  expect(merged_pact.unwrap().as_request_response_pact().unwrap()).to(be_equal_to(updated_pact));
}
