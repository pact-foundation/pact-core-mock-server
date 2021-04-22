use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io;
use std::str::FromStr;

#[allow(unused_imports)] use env_logger;
use expectest::expect;
use expectest::prelude::*;
use maplit::*;
use rand;
use serde_json::json;

use crate::models::matchingrules::{matchers_from_json, MatchingRule};

use super::*;
use super::{body_from_json, headers_from_json};
use super::generators::{Generator, Generators, generators_from_json};
use super::provider_states::*;

#[test]
fn request_from_json_defaults_to_get() {
    let request_json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {}
      }
     "#).unwrap();
    let request = Request::from_json(&request_json, &PactSpecification::V1);
    expect!(request.method).to(be_equal_to("GET"));
}

#[test]
fn request_from_json_defaults_to_root_for_path() {
    let request_json : serde_json::Value = serde_json::from_str(r#"
      {
          "method": "PUT",
          "query": "",
          "headers": {}
      }
     "#).unwrap();
    println!("request_json: {}", request_json);
    let request = Request::from_json(&request_json, &PactSpecification::V1_1);
    assert_eq!(request.path, "/".to_string());
}

#[test]
fn response_from_json_defaults_to_status_200() {
    let response_json : serde_json::Value = serde_json::from_str(r#"
      {
          "headers": {}
      }
     "#).unwrap();
    let response = Response::from_json(&response_json, &PactSpecification::V1_1);
    assert_eq!(response.status, 200);
}

#[test]
fn parse_query_string_test() {
  let query = "a=b&c=d".to_string();
  let expected = hashmap!{
    "a".to_string() => vec!["b".to_string()],
    "c".to_string() => vec!["d".to_string()]
  };
  let result = parse_query_string(&query);
  expect!(result).to(be_some().value(expected));
}

#[test]
fn parse_query_string_handles_empty_string() {
    let query = "".to_string();
    let expected = None;
    let result = parse_query_string(&query);
    assert_eq!(result, expected);
}

#[test]
fn parse_query_string_handles_missing_values() {
    let query = "a=&c=d".to_string();
    let mut expected = HashMap::new();
    expected.insert("a".to_string(), vec!["".to_string()]);
    expected.insert("c".to_string(), vec!["d".to_string()]);
    let result = parse_query_string(&query);
    assert_eq!(result, Some(expected));
}

#[test]
fn parse_query_string_handles_equals_in_values() {
    let query = "a=b&c=d=e=f".to_string();
    let mut expected = HashMap::new();
    expected.insert("a".to_string(), vec!["b".to_string()]);
    expected.insert("c".to_string(), vec!["d=e=f".to_string()]);
    let result = parse_query_string(&query);
    assert_eq!(result, Some(expected));
}

#[test]
fn parse_query_string_decodes_values() {
  let query = "a=a%20b%20c".to_string();
  let expected = hashmap! {
    "a".to_string() => vec!["a b c".to_string()]
  };
  let result = parse_query_string(&query);
  expect!(result).to(be_some().value(expected));
}

#[test]
fn parse_query_string_decodes_non_ascii_values() {
  let query = "accountNumber=100&anotherValue=%E6%96%87%E4%BB%B6.txt".to_string();
  let expected = hashmap! {
    "accountNumber".to_string() => vec!["100".to_string()],
    "anotherValue".to_string() => vec!["文件.txt".to_string()]
  };
  let result = parse_query_string(&query);
  expect!(result).to(be_some().value(expected));
}

#[test]
#[ignore]
fn quickcheck_parse_query_string() {
    use quickcheck::{TestResult, quickcheck};
    use super::decode_query;
    use itertools::Itertools;
    fn prop(s: String) -> TestResult {
        if s.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '&' || c == '%') {
            let result = match parse_query_string(&s) {
            Some(map) => {
                    if map.len() == 1 && !s.contains("=") {
                        *map.keys().next().unwrap() == decode_query(&s).unwrap()
                } else {
                        let reconstructed_query = map.iter().map(|(k, v)| {
                            v.iter().map(|qv| format!("{}={}", k, qv)).join("&")
                        }).join("&");
                        let r = decode_query(&s).unwrap() == reconstructed_query;
                        // if !r {
                        //     dbg!(reconstructed_query);
                        //     dbg!(decode_query(&s) == reconstructed_query);
                        // }
                        r
                    }
                },
                None => s.is_empty()
            };

            // if !result {
            //     dbg!(s);
            //     dbg!(decode_query(&s));
            // }
            TestResult::from_bool(result)
        } else {
            TestResult::discard()
        }
    }
    quickcheck(prop as fn(_) -> _);
}

#[test]
fn request_content_type_is_based_on_the_content_type_header() {
    let request = Request {
        method: s!("GET"),
        path: s!("/"),
        query: None,
        headers: None,
        body: OptionalBody::Missing,
        ..Request::default()
    };
    expect!(request.content_type().unwrap_or_default().to_string()).to(be_equal_to("*/*"));
    expect!(Request {
        headers: Some(hashmap!{ s!("Content-Type") => vec![s!("text/html")] }), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("text/html"));
    expect!(Request {
        headers: Some(hashmap!{ s!("Content-Type") => vec![s!("application/json; charset=UTF-8")] }), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("application/json;charset=utf-8"));
    expect!(Request {
        headers: Some(hashmap!{ s!("Content-Type") => vec![s!("application/json")] }), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("application/json"));
    expect!(Request {
        headers: Some(hashmap!{ s!("CONTENT-TYPE") => vec![s!("application/json; charset=UTF-8")] }), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("application/json;charset=utf-8"));
    expect!(Request {
        body: OptionalBody::Present("{\"json\": true}".into(), None), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("application/json"));
    expect!(Request {
        body: OptionalBody::Present("{}".into(), None), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("application/json"));
    expect!(Request {
        body: OptionalBody::Present("[]".into(), None), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("application/json"));
    expect!(Request {
        body: OptionalBody::Present("[1,2,3]".into(), None), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("application/json"));
    expect!(Request {
        body: OptionalBody::Present("\"string\"".into(), None), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("application/json"));
    expect!(Request {
        body: OptionalBody::Present("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<json>false</json>".into(), None), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("application/xml"));
    expect!(Request {
        body: OptionalBody::Present("<json>false</json>".into(), None), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("application/xml"));
    expect!(Request {
        body: OptionalBody::Present("this is not json".into(), None), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("text/plain"));
    expect!(Request {
        body: OptionalBody::Present("<html><body>this is also not json</body></html>".into(), None), .. request.clone() }.content_type().unwrap_or_default().to_string())
      .to(be_equal_to("text/html"));
}

#[test]
fn content_type_struct_test() {
    let request = Request {
        method: s!("GET"),
        path: s!("/"),
        query: None,
        headers: None,
        body: OptionalBody::Missing,
        ..Request::default()
    };
    expect!(request.content_type()).to(be_none());
    expect!(Request {
        headers: Some(hashmap!{ s!("Content-Type") => vec![s!("text/html")] }), .. request.clone() }.content_type())
      .to(be_some().value(HTML.clone()));
    expect!(Request {
        headers: Some(hashmap!{ s!("Content-Type") => vec![s!("application/json")] }), .. request.clone() }.content_type())
      .to(be_some().value(JSON.clone()));
    expect!(Request {
        headers: Some(hashmap!{ s!("Content-Type") => vec![s!("application/hal+json")] }), .. request.clone() }
        .content_type().map(|c| c.base_type()))
      .to(be_some().value(JSON.clone()));
    expect!(Request {
        headers: Some(hashmap!{ s!("CONTENT-TYPE") => vec![s!("application/xml")] }), .. request.clone() }.content_type())
      .to(be_some().value(XML.clone()));
    expect!(Request {
        headers: Some(hashmap!{ s!("CONTENT-TYPE") => vec![s!("application/stuff+xml")] }), ..
        request.clone() }.content_type().map(|c| c.base_type()))
      .to(be_some().value(XML.clone()));
}

#[test]
fn http_part_has_header_test() {
    let request = Request { method: s!("GET"), path: s!("/"), query: None,
        headers: Some(hashmap!{ s!("Content-Type") => vec![s!("application/json; charset=UTF-8")] }),
        body: OptionalBody::Missing, .. Request::default() };
    expect!(request.has_header(&s!("Content-Type"))).to(be_true());
    expect!(request.lookup_header_value(&s!("Content-Type"))).to(be_some().value("application/json; charset=UTF-8"));
}

#[test]
fn loading_interaction_from_json() {
    let interaction_json = r#"{
        "description": "String",
        "providerState": "provider state"
    }"#;
    let interaction = RequestResponseInteraction::from_json(0, &serde_json::from_str(interaction_json).unwrap(), &PactSpecification::V1_1);
    expect!(interaction.description).to(be_equal_to("String"));
    expect!(interaction.provider_states).to(be_equal_to(vec![
        ProviderState { name: s!("provider state"), params: hashmap!{} } ]));
}

#[test]
fn defaults_to_number_if_no_description() {
    let interaction_json = r#"{
        "providerState": "provider state"
    }"#;
    let interaction = RequestResponseInteraction::from_json(0, &serde_json::from_str(interaction_json).unwrap(), &PactSpecification::V1_1);
    expect!(interaction.description).to(be_equal_to("Interaction 0"));
    expect!(interaction.provider_states).to(be_equal_to(vec![
        ProviderState { name: s!("provider state"), params: hashmap!{} } ]));
}

#[test]
fn defaults_to_empty_if_no_provider_state() {
    let interaction_json = r#"{
    }"#;
    let interaction = RequestResponseInteraction::from_json(0, &serde_json::from_str(interaction_json).unwrap(), &PactSpecification::V1);
    expect!(interaction.provider_states.iter()).to(be_empty());
}

#[test]
fn defaults_to_none_if_provider_state_null() {
    let interaction_json = r#"{
        "providerState": null
    }"#;
    let interaction = RequestResponseInteraction::from_json(0, &serde_json::from_str(interaction_json).unwrap(), &PactSpecification::V1);
    expect!(interaction.provider_states.iter()).to(be_empty());
}

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
fn request_to_json_with_defaults() {
    let request = Request::default();
    expect!(request.to_json(&PactSpecification::V3).to_string()).to(
        be_equal_to("{\"method\":\"GET\",\"path\":\"/\"}"));
}

#[test]
fn request_to_json_converts_methods_to_upper_case() {
    let request = Request { method: s!("post"), .. Request::default() };
    expect!(request.to_json(&PactSpecification::V3).to_string()).to(be_equal_to("{\"method\":\"POST\",\"path\":\"/\"}"));
}

#[test]
fn request_to_json_with_a_query() {
    let request = Request { query: Some(hashmap!{
        s!("a") => vec![s!("1"), s!("2")],
        s!("b") => vec![s!("3")]
    }), .. Request::default() };
    expect!(request.to_json(&PactSpecification::V2).to_string()).to(
        be_equal_to(r#"{"method":"GET","path":"/","query":"a=1&a=2&b=3"}"#)
    );
}

#[test]
fn request_to_json_with_a_query_must_encode_the_query() {
    let request = Request { query: Some(hashmap!{
        s!("datetime") => vec![s!("2011-12-03T10:15:30+01:00")],
        s!("description") => vec![s!("hello world!")] }), .. Request::default() };
    expect!(request.to_json(&PactSpecification::V2).to_string()).to(
        be_equal_to(r#"{"method":"GET","path":"/","query":"datetime=2011-12-03T10%3a15%3a30%2b01%3a00&description=hello+world%21"}"#)
    );
}

#[test]
fn request_to_json_with_a_query_must_encode_the_query_with_utf8_chars() {
    let request = Request { query: Some(hashmap!{
        s!("a") => vec![s!("b=c&d❤")]
    }), .. Request::default() };
    expect!(request.to_json(&PactSpecification::V2).to_string()).to(
        be_equal_to(r#"{"method":"GET","path":"/","query":"a=b%3dc%26d%27%64"}"#)
    );
}

#[test]
fn request_to_json_with_a_query_v3() {
    let request = Request { query: Some(hashmap!{
        s!("a") => vec![s!("1"), s!("2")],
        s!("b") => vec![s!("3")]
    }), .. Request::default() };
    expect!(request.to_json(&PactSpecification::V3).to_string()).to(
        be_equal_to(r#"{"method":"GET","path":"/","query":{"a":["1","2"],"b":["3"]}}"#)
    );
}

#[test]
fn request_to_json_with_a_query_v3_must_not_encode_the_query() {
    let request = Request { query: Some(hashmap!{
        s!("datetime") => vec![s!("2011-12-03T10:15:30+01:00")],
        s!("description") => vec![s!("hello world!")] }), .. Request::default() };
    expect!(request.to_json(&PactSpecification::V3).to_string()).to(
        be_equal_to(r#"{"method":"GET","path":"/","query":{"datetime":["2011-12-03T10:15:30+01:00"],"description":["hello world!"]}}"#)
    );
}

#[test]
fn request_to_json_with_a_query_v3_must_not_encode_the_query_with_utf8_chars() {
    let request = Request { query: Some(hashmap!{
        s!("a") => vec![s!("b=c&d❤")]
    }), .. Request::default() };
    expect!(request.to_json(&PactSpecification::V3).to_string()).to(
        be_equal_to(r#"{"method":"GET","path":"/","query":{"a":["b=c&d❤"]}}"#)
    );
}

#[test]
fn request_to_json_with_headers() {
    let request = Request { headers: Some(hashmap!{
        s!("HEADERA") => vec![s!("VALUEA")],
        s!("HEADERB") => vec![s!("VALUEB1, VALUEB2")]
    }), .. Request::default() };
    expect!(request.to_json(&PactSpecification::V3).to_string()).to(
        be_equal_to(r#"{"headers":{"HEADERA":"VALUEA","HEADERB":"VALUEB1, VALUEB2"},"method":"GET","path":"/"}"#)
    );
}

#[test]
fn request_to_json_with_json_body() {
    let request = Request { headers: Some(hashmap!{
        s!("Content-Type") => vec![s!("application/json")]
    }), body: OptionalBody::Present(r#"{"key": "value"}"#.into(), None), .. Request::default() };
    expect!(request.to_json(&PactSpecification::V3).to_string()).to(
        be_equal_to(r#"{"body":{"key":"value"},"headers":{"Content-Type":"application/json"},"method":"GET","path":"/"}"#)
    );
}


#[test]
fn request_to_json_with_non_json_body() {
    let request = Request { headers: Some(hashmap!{ s!("Content-Type") => vec![s!("text/plain")] }),
        body: OptionalBody::Present("This is some text".into(), None), .. Request::default() };
    expect!(request.to_json(&PactSpecification::V3).to_string()).to(
        be_equal_to(r#"{"body":"This is some text","headers":{"Content-Type":"text/plain"},"method":"GET","path":"/"}"#)
    );
}

#[test]
fn request_to_json_with_empty_body() {
    let request = Request { body: OptionalBody::Empty, .. Request::default() };
    expect!(request.to_json(&PactSpecification::V3).to_string()).to(
        be_equal_to(r#"{"body":"","method":"GET","path":"/"}"#)
    );
}

#[test]
fn request_to_json_with_null_body() {
    let request = Request { body: OptionalBody::Null, .. Request::default() };
    expect!(request.to_json(&PactSpecification::V3).to_string()).to(
        be_equal_to(r#"{"body":null,"method":"GET","path":"/"}"#)
    );
}

#[test]
fn response_to_json_with_defaults() {
    let response = Response::default();
    expect!(response.to_json(&PactSpecification::V3).to_string()).to(be_equal_to("{\"status\":200}"));
}

#[test]
fn response_to_json_with_headers() {
    let response = Response { headers: Some(hashmap!{
        s!("HEADERA") => vec![s!("VALUEA")],
        s!("HEADERB") => vec![s!("VALUEB1, VALUEB2")]
    }), .. Response::default() };
    expect!(response.to_json(&PactSpecification::V3).to_string()).to(
        be_equal_to(r#"{"headers":{"HEADERA":"VALUEA","HEADERB":"VALUEB1, VALUEB2"},"status":200}"#)
    );
}

#[test]
fn response_to_json_with_json_body() {
    let response = Response { headers: Some(hashmap!{
        s!("Content-Type") => vec![s!("application/json")]
    }), body: OptionalBody::Present(r#"{"key": "value"}"#.into(), None), .. Response::default() };
    expect!(response.to_json(&PactSpecification::V3).to_string()).to(
        be_equal_to(r#"{"body":{"key":"value"},"headers":{"Content-Type":"application/json"},"status":200}"#)
    );
}

#[test]
fn response_to_json_with_non_json_body() {
    let response = Response { headers: Some(hashmap!{ s!("Content-Type") => vec![s!("text/plain")] }),
        body: OptionalBody::Present("This is some text".into(), None), .. Response::default() };
    expect!(response.to_json(&PactSpecification::V3).to_string()).to(
        be_equal_to(r#"{"body":"This is some text","headers":{"Content-Type":"text/plain"},"status":200}"#)
    );
}

#[test]
fn response_to_json_with_empty_body() {
    let response = Response { body: OptionalBody::Empty, .. Response::default() };
    expect!(response.to_json(&PactSpecification::V3).to_string()).to(
        be_equal_to(r#"{"body":"","status":200}"#)
    );
}

#[test]
fn response_to_json_with_null_body() {
    let response = Response { body: OptionalBody::Null, .. Response::default() };
    expect!(response.to_json(&PactSpecification::V3).to_string()).to(
        be_equal_to(r#"{"body":null,"status":200}"#)
    );
}

#[test]
fn interaction_from_json_sets_the_id_if_loaded_from_broker() {
  let json = json!({
    "_id": "123456789",
    "description": "Test Interaction",
    "providerState": "Good state to be in",
    "request": {
      "method": "GET",
      "path": "/"
    },
    "response": {
      "status": 200
    }
  });
  expect!(RequestResponseInteraction::from_json(0, &json, &PactSpecification::V3).id).to(be_some().value("123456789".to_string()));
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

    let result = write_pact(&pact, dir.as_path(), PactSpecification::V2, true);

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

    let result = write_pact(&pact, dir.as_path(), PactSpecification::V2, false);
    let result2 = write_pact(&pact2, dir.as_path(), PactSpecification::V2, false);

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

    let result = write_pact(&pact, dir.as_path(), PactSpecification::V2, false);
    let result2 = write_pact(&pact2, dir.as_path(), PactSpecification::V2, false);

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

    let result = write_pact(&pact, dir.as_path(), PactSpecification::V2, false);
    let result2 = write_pact(&pact2, dir.as_path(), PactSpecification::V3, false);

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
    consumer: Consumer { name: s!("merge_consumer") },
    provider: Provider { name: s!("merge_provider") },
    interactions: vec![
      RequestResponseInteraction {
        description: s!("Test Interaction 2"),
        provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap! {} }],
        ..RequestResponseInteraction::default()
      }
    ],
    metadata: btreemap! {},
    specification_version: PactSpecification::V1_1,
  };
  let pact2 = V4Pact {
    consumer: Consumer { name: s!("merge_consumer") },
    provider: Provider { name: s!("merge_provider") },
    interactions: vec![
      Box::new(SynchronousHttp {
        id: None,
        key: None,
        description: s!("Test Interaction"),
        provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap! {} }],
        request: Default::default(),
        response: Default::default(),
      })
    ],
    metadata: btreemap! {},
  };
  let mut dir = env::temp_dir();
  let x = rand::random::<u16>();
  dir.push(format!("pact_test_{}", x));
  dir.push(pact.default_file_name());

  let result = write_pact(&pact, dir.as_path(), PactSpecification::V3, false);
  let result2 = write_pact(&pact2, dir.as_path(), PactSpecification::V4, false);

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
      "key": "53d3170820ad2160",
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
      "key": "4da93913a351bb8c",
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
    expect!(merged_pact.clone()).to(be_ok());
    expect!(merged_pact.clone().unwrap().interactions.len()).to(be_equal_to(2));

    let merged_pact2 = pact.merge(&pact.clone());
    expect!(merged_pact2.clone()).to(be_ok());
    expect!(merged_pact2.clone().unwrap().interactions.len()).to(be_equal_to(1));
}

#[test]
fn interactions_do_not_conflict_if_they_have_different_descriptions() {
    let interaction1 = RequestResponseInteraction {
        description: s!("Test Interaction"),
        provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
        .. RequestResponseInteraction::default()
    };
    let interaction2 = RequestResponseInteraction {
        description: s!("Test Interaction 2"),
        provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
        .. RequestResponseInteraction::default()
    };
    expect!(interaction1.conflicts_with(&interaction2).iter()).to(be_empty());
}

#[test]
fn interactions_do_not_conflict_if_they_have_different_provider_states() {
    let interaction1 = RequestResponseInteraction {
        description: s!("Test Interaction"),
        provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
        .. RequestResponseInteraction::default()
    };
    let interaction2 = RequestResponseInteraction {
        description: s!("Test Interaction"),
        provider_states: vec![ProviderState { name: s!("Bad state to be in"), params: hashmap!{} }],
        .. RequestResponseInteraction::default()
    };
    expect!(interaction1.conflicts_with(&interaction2).iter()).to(be_empty());
}

#[test]
fn interactions_do_not_conflict_if_they_have_the_same_requests_and_responses() {
    let interaction1 = RequestResponseInteraction {
        description: s!("Test Interaction"),
        provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
        .. RequestResponseInteraction::default()
    };
    let interaction2 = RequestResponseInteraction {
        description: s!("Test Interaction"),
        provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
        .. RequestResponseInteraction::default()
    };
    expect!(interaction1.conflicts_with(&interaction2).iter()).to(be_empty());
}

#[test]
fn interactions_conflict_if_they_have_different_requests() {
    let interaction1 = RequestResponseInteraction {
        description: s!("Test Interaction"),
        provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
        .. RequestResponseInteraction::default()
    };
    let interaction2 = RequestResponseInteraction {
        description: s!("Test Interaction"),
        provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
        request: Request { method: s!("POST"), .. Request::default() },
        .. RequestResponseInteraction::default()
    };
    expect!(interaction1.conflicts_with(&interaction2).iter()).to_not(be_empty());
}

#[test]
fn interactions_conflict_if_they_have_different_responses() {
    let interaction1 = RequestResponseInteraction {
        description: s!("Test Interaction"),
        provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
        .. RequestResponseInteraction::default()
    };
    let interaction2 = RequestResponseInteraction {
        description: s!("Test Interaction"),
        provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
        response: Response { status: 400, .. Response::default() },
        .. RequestResponseInteraction::default()
    };
    expect!(interaction1.conflicts_with(&interaction2).iter()).to_not(be_empty());
}

#[test]
fn request_headers_do_not_conflict_if_they_have_been_serialised_and_deserialised_to_json() {
    // headers are serialised in a hashmap; serializing and deserializing can can change the
    // internal order of the keys in the hashmap, and this can confuse the differences_from code.
    let original_request = Request {
        method: "".to_string(),
        path: "".to_string(),
        query: None,
        headers: Some(hashmap! {
          "accept".to_string() => vec!["application/xml".to_string(), "application/json".to_string()],
          "user-agent".to_string() => vec!["test".to_string(), "test2".to_string()],
          "content-type".to_string() => vec!["text/plain".to_string()]
        }),
        body: OptionalBody::Missing,
        matching_rules: Default::default(),
        generators: Default::default(),
    };

    let json = serde_json::to_string(&original_request).expect("could not serialize");

    let serialized_and_deserialized_request =
        serde_json::from_str(&json).expect("could not deserialize");

    expect!(original_request
        .differences_from(&serialized_and_deserialized_request)
        .iter())
        .to(be_empty());
}

fn hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[test]
fn hash_for_request() {
    let request1 = Request::default();
    let request2 = Request { method: s!("POST"), .. Request::default() };
    let request3 = Request { headers: Some(hashmap!{
        s!("H1") => vec![s!("A")]
    }), .. Request::default() };
    let request4 = Request { headers: Some(hashmap!{
        s!("H1") => vec![s!("B")]
    }), .. Request::default() };
    expect!(hash(&request1)).to(be_equal_to(hash(&request1)));
    expect!(hash(&request3)).to(be_equal_to(hash(&request3)));
    expect!(hash(&request1)).to_not(be_equal_to(hash(&request2)));
    expect!(hash(&request3)).to_not(be_equal_to(hash(&request4)));
}

#[test]
fn hash_for_response() {
    let response1 = Response::default();
    let response2 = Response { status: 400, .. Response::default() };
    let response3 = Response { headers: Some(hashmap!{
        s!("H1") => vec![s!("A")]
    }), .. Response::default() };
    let response4 = Response { headers: Some(hashmap!{
        s!("H1") => vec![s!("B")]
    }), .. Response::default() };
    expect!(hash(&response1)).to(be_equal_to(hash(&response1)));
    expect!(hash(&response3)).to(be_equal_to(hash(&response3)));
    expect!(hash(&response1)).to_not(be_equal_to(hash(&response2)));
    expect!(hash(&response3)).to_not(be_equal_to(hash(&response4)));
}

#[test]
fn matchers_from_json_handles_missing_matchers() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {}
      }
     "#).unwrap();
    let matchers = matchers_from_json(&json, &Some(s!("deprecatedName")));
    expect!(matchers.rules.iter()).to(be_empty());
}

#[test]
fn matchers_from_json_handles_empty_matchers() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "matchingRules": {}
      }
     "#).unwrap();
    let matchers = matchers_from_json(&json, &Some(s!("deprecatedName")));
    expect!(matchers.rules.iter()).to(be_empty());
}

#[test]
fn matchers_from_json_handles_matcher_with_no_matching_rules() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "matchingRules": {
            "body": {
                "$.*.path": {}
            }
          }
      }
     "#).unwrap();
    let matchers = matchers_from_json(&json, &Some(s!("deprecatedName")));
    expect!(matchers).to(be_equal_to(matchingrules!{
        "body" => {
            "$.*.path" => [ ]
        }
    }));
}

#[test]
fn matchers_from_json_loads_matchers_correctly() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "matchingRules": {
            "body": {
                "$.*.path": {
                    "matchers": [{
                        "match": "regex",
                        "regex": "\\d+"
                    }]
                }
            }
          }
      }
     "#).unwrap();
    let matchers = matchers_from_json(&json, &Some(s!("deprecatedName")));
    expect!(matchers).to(be_equal_to(matchingrules!{
        "body" => {
            "$.*.path" => [ MatchingRule::Regex(s!("\\d+")) ]
        }
    }));
}

#[test]
fn matchers_from_json_loads_matchers_from_deprecated_name() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "deprecatedName": {
              "body": {
                "$.*.path": {
                    "matchers": [{
                        "match": "regex",
                        "regex": "\\d+"
                    }]
                }
              }
          }
      }
     "#).unwrap();
    let matchers = matchers_from_json(&json, &Some(s!("deprecatedName")));
    expect!(matchers).to(be_equal_to(matchingrules!{
        "body" => {
            "$.*.path" => [ MatchingRule::Regex(s!(r#"\d+"#)) ]
        }
    }));
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

    let result = write_pact(&pact, dir.as_path(), PactSpecification::V2, true);

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

    let result = write_pact(&pact, dir.as_path(), PactSpecification::V3, true);

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
fn body_from_json_returns_missing_if_there_is_no_body() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "matchingRules": {
            "*.path": {}
          }
      }
     "#).unwrap();
    let body = body_from_json(&json, "body", &None);
    expect!(body).to(be_equal_to(OptionalBody::Missing));
}

#[test]
fn body_from_json_returns_null_if_the_body_is_null() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "body": null
      }
     "#).unwrap();
    let body = body_from_json(&json, "body", &None);
    expect!(body).to(be_equal_to(OptionalBody::Null));
}

#[test]
fn body_from_json_returns_json_string_if_the_body_is_json_but_not_a_string() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "body": {
            "test": true
          }
      }
     "#).unwrap();
    let body = body_from_json(&json, "body", &None);
    expect!(body).to(be_equal_to(OptionalBody::Present("{\"test\":true}".into(), None)));
}

#[test]
fn body_from_json_returns_empty_if_the_body_is_an_empty_string() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "body": ""
      }
     "#).unwrap();
    let body = body_from_json(&json, "body", &None);
    expect!(body).to(be_equal_to(OptionalBody::Empty));
}

#[test]
fn body_from_json_returns_the_body_if_the_body_is_a_string() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "body": "<?xml version=\"1.0\"?> <body></body>"
      }
     "#).unwrap();
    let body = body_from_json(&json, "body", &None);
    expect!(body).to(be_equal_to(OptionalBody::Present("<?xml version=\"1.0\"?> <body></body>".into(), Some("application/xml".into()))));
}

#[test]
fn body_from_text_plain_type_returns_the_same_formatted_body() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {"Content-Type": "text/plain"},
          "body": "\"This is a string\""
      }
     "#).unwrap();
    let headers = headers_from_json(&json);
    let body = body_from_json(&json, "body", &headers);
    expect!(body).to(be_equal_to(OptionalBody::Present("\"This is a string\"".into(), Some("text/plain".into()))));
}

#[test]
fn body_from_text_html_type_returns_the_same_formatted_body() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {"Content-Type": "text/html"},
          "body": "\"This is a string\""
      }
     "#).unwrap();
    let headers = headers_from_json(&json);
    let body = body_from_json(&json, "body", &headers);
    expect!(body).to(be_equal_to(OptionalBody::Present("\"This is a string\"".into(), Some("text/html".into()))));
}

#[test]
fn body_from_json_returns_the_a_json_formatted_body_if_the_body_is_a_string_and_the_content_type_is_json() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {"Content-Type": "application/json"},
          "body": "This is actually a JSON string"
      }
     "#).unwrap();
    let headers = headers_from_json(&json);
    let body = body_from_json(&json, "body", &headers);
    expect!(body).to(be_equal_to(OptionalBody::Present("\"This is actually a JSON string\"".into(), Some("application/json".into()))));
}

#[test]
fn body_from_json_returns_the_a_json_formatted_body_if_the_body_is_a_valid_json_string_and_the_content_type_is_json() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {"Content-Type": "application/json"},
          "body": "\"This is actually a JSON string\""
      }
     "#).unwrap();
    let headers = headers_from_json(&json);
    let body = body_from_json(&json, "body", &headers);
    expect!(body).to(be_equal_to(OptionalBody::Present("\"This is actually a JSON string\"".into(), Some("application/json".into()))));
}

#[test]
fn body_from_json_returns_the_body_if_the_content_type_is_json() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {"Content-Type": "application/json"},
          "body": "{\"test\":true}"
      }
     "#).unwrap();
    let headers = headers_from_json(&json);
    let body = body_from_json(&json, "body", &headers);
    expect!(body).to(be_equal_to(OptionalBody::Present("{\"test\":true}".into(), Some("application/json".into()))));
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

    let result = write_pact(&pact, dir.as_path(), PactSpecification::V3, true);

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
fn generators_from_json_handles_missing_generators() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {}
      }
     "#).unwrap();
    let generators = generators_from_json(&json);
    expect!(generators.categories.iter()).to(be_empty());
}

#[test]
fn generators_from_json_handles_empty_generators() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "generators": {}
      }
     "#).unwrap();
    let generators = generators_from_json(&json);
    expect!(generators.categories.iter()).to(be_empty());
}

#[test]
fn generators_from_json_handles_generator_with_no_rules() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "generators": {
            "body": {
                "$.*.path": {}
            }
          }
      }
     "#).unwrap();
    let generators = generators_from_json(&json);
    expect!(generators).to(be_equal_to(Generators::default()));
}

#[test]
fn generators_from_json_ignores_invalid_generators() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "generators": {
            "body": {
                "$.*.path": {
                  "type": "invalid"
                },
                "$.invalid": {
                  "type": 100
                },
                "$.other": null
            },
            "invalid": {
                "path": "path"
            },
            "more_invalid": 100
          }
      }
     "#).unwrap();
    let generators = generators_from_json(&json);
    expect!(generators).to(be_equal_to(Generators::default()));
}

#[test]
fn generators_from_json_loads_generators_correctly() {
    let json : serde_json::Value = serde_json::from_str(r#"
      {
        "path": "/",
        "query": "",
        "headers": {},
        "generators": {
          "body": {
              "$.*.path": {
                  "type": "RandomInt",
                  "min": 1,
                  "max": 10
              }
          },
          "path": {
            "type": "RandomString"
          }
        }
      }
     "#).unwrap();
    let generators = generators_from_json(&json);
    expect!(generators).to(be_equal_to(generators!{
        "BODY" => {
            "$.*.path" => Generator::RandomInt(1, 10)
        },
        "PATH" => { "" => Generator::RandomString(10) }
    }));
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

    let result = write_pact(&pact, dir.as_path(), PactSpecification::V3, true);

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
  expect(merged_pact).to(be_ok().value(updated_pact));
}
