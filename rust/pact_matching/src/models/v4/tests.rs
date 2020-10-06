use expectest::prelude::*;
use serde_json::json;
use maplit::*;

use crate::models::v4::http_parts::{HttpRequest, HttpResponse};
use crate::models::v4::{V4Interaction, interaction_from_json, from_json};
use crate::models::v4::http_parts::body_from_json;
use crate::models::provider_states::ProviderState;
use crate::models::{Interaction, PactSpecification, OptionalBody, headers_from_json};

#[test]
fn synchronous_http_request_from_json_defaults_to_get() {
  let request_json : serde_json::Value = serde_json::from_str(r#"
    {
        "path": "/",
        "query": "",
        "headers": {}
    }
   "#).unwrap();
  let request = HttpRequest::from_json(&request_json);
  expect!(request.method).to(be_equal_to("GET"));
}

#[test]
fn synchronous_http_request_from_json_defaults_to_root_for_path() {
  let request_json : serde_json::Value = serde_json::from_str(r#"
      {
          "method": "PUT",
          "query": "",
          "headers": {}
      }
     "#).unwrap();
  let request = HttpRequest::from_json(&request_json);
  assert_eq!(request.path, "/".to_string());
}

#[test]
fn synchronous_http_response_from_json_defaults_to_status_200() {
  let response_json : serde_json::Value = serde_json::from_str(r#"
    {
        "headers": {}
    }
   "#).unwrap();
  let response = HttpResponse::from_json(&response_json);
  assert_eq!(response.status, 200);
}

#[test]
fn synchronous_http_request_content_type_falls_back_the_content_type_header_and_then_the_contents() {
  let request_json = json!({
    "headers": {},
    "body": {
      "content": "string"
    }
  });
  let request = HttpRequest::from_json(&request_json);
  expect!(request.body.content_type().unwrap()).to(be_equal_to("text/plain"));

  let request_json = json!({
    "headers": {
      "Content-Type": ["text/html"]
    },
    "body": {
      "content": "string"
    }
  });
  let request = HttpRequest::from_json(&request_json);
  expect!(request.body.content_type().unwrap()).to(be_equal_to("text/html"));

  let request_json = json!({
    "headers": {
      "Content-Type": ["application/json; charset=UTF-8"]
    },
    "body": {
      "content": "string"
    }
  });
  let request = HttpRequest::from_json(&request_json);
  expect!(request.body.content_type().unwrap()).to(be_equal_to("application/json;charset=utf-8"));

  let request_json = json!({
    "headers": {
      "CONTENT-TYPE": ["application/json; charset=UTF-8"]
    },
    "body": {
      "content": "string"
    }
  });
  let request = HttpRequest::from_json(&request_json);
  expect!(request.body.content_type().unwrap()).to(be_equal_to("application/json;charset=utf-8"));

  let request_json = json!({
    "body": {
      "content": { "json": true }
    }
  });
  let request = HttpRequest::from_json(&request_json);
  expect!(request.body.content_type().unwrap()).to(be_equal_to("application/json"));
}

#[test]
fn loading_interaction_from_json() {
  let interaction_json = json!({
    "type": "Synchronous/HTTP",
    "description": "String",
    "providerStates": [{ "name": "provider state" }]
  });
  let interaction = interaction_from_json("", 0, &interaction_json).unwrap();
  expect!(interaction.description()).to(be_equal_to("String"));
  expect!(interaction.provider_states()).to(be_equal_to(vec![
    ProviderState { name: "provider state".into(), params: hashmap!{} } ]));
}

#[test]
fn defaults_to_number_if_no_description() {
  let interaction_json = json!({
    "type": "Synchronous/HTTP"
  });
  let interaction = interaction_from_json("", 0, &interaction_json).unwrap();
  expect!(interaction.description()).to(be_equal_to("Interaction 0"));
}

#[test]
fn defaults_to_empty_if_no_provider_state() {
  let interaction_json = json!({
    "type": "Synchronous/HTTP"
  });
  let interaction = interaction_from_json("", 0, &interaction_json).unwrap();
  expect!(interaction.provider_states().iter()).to(be_empty());
}

#[test]
fn defaults_to_none_if_provider_state_null() {
  let interaction_json = json!({
    "type": "Synchronous/HTTP",
    "description": "String",
    "providerStates": null
  });
  let interaction = interaction_from_json("", 0, &interaction_json).unwrap();
  expect!(interaction.provider_states().iter()).to(be_empty());
}

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
  match &v4pact.interactions[0] {
    V4Interaction::SynchronousHttp { request, response, .. } => {
      expect!(request).to(be_equal_to(&HttpRequest {
        method: "GET".into(),
        path: "/mallory".into(),
        query: Some(hashmap!{ "name".to_string() => vec!["ron".to_string()], "status".to_string() => vec!["good".to_string()] }),
        headers: None,
        body: OptionalBody::Missing,
        .. HttpRequest::default()
      }));
      expect!(response).to(be_equal_to(&HttpResponse {
        status: 200,
        headers: Some(hashmap!{ "Content-Type".to_string() => vec!["text/html".to_string()] }),
        body: OptionalBody::Present("\"That is some good Mallory.\"".into(), Some("text/html".into())),
        .. HttpResponse::default()
      }));
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
  match &v4pact.interactions[0] {
    V4Interaction::SynchronousHttp { request, .. } => {
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
  match &v4pact.interactions[0] {
    V4Interaction::SynchronousHttp { request, .. } => {
      expect!(&request.method).to(be_equal_to("GET"));
    }
    _ => panic!("Was expecting an HTTP pact")
  }
}

// #[test]
// fn request_to_json_with_defaults() {
//   let request = Request::default();
//   expect!(request.to_json(&PactSpecification::V3).to_string()).to(
//     be_equal_to("{\"method\":\"GET\",\"path\":\"/\"}"));
// }
//
// #[test]
// fn request_to_json_converts_methods_to_upper_case() {
//   let request = Request { method: s!("post"), .. Request::default() };
//   expect!(request.to_json(&PactSpecification::V3).to_string()).to(be_equal_to("{\"method\":\"POST\",\"path\":\"/\"}"));
// }
//
// #[test]
// fn request_to_json_with_a_query() {
//   let request = Request { query: Some(hashmap!{
//         s!("a") => vec![s!("1"), s!("2")],
//         s!("b") => vec![s!("3")]
//     }), .. Request::default() };
//   expect!(request.to_json(&PactSpecification::V2).to_string()).to(
//     be_equal_to(r#"{"method":"GET","path":"/","query":"a=1&a=2&b=3"}"#)
//   );
// }
//
// #[test]
// fn request_to_json_with_a_query_must_encode_the_query() {
//   let request = Request { query: Some(hashmap!{
//         s!("datetime") => vec![s!("2011-12-03T10:15:30+01:00")],
//         s!("description") => vec![s!("hello world!")] }), .. Request::default() };
//   expect!(request.to_json(&PactSpecification::V2).to_string()).to(
//     be_equal_to(r#"{"method":"GET","path":"/","query":"datetime=2011-12-03T10%3a15%3a30%2b01%3a00&description=hello+world%21"}"#)
//   );
// }
//
// #[test]
// fn request_to_json_with_a_query_must_encode_the_query_with_utf8_chars() {
//   let request = Request { query: Some(hashmap!{
//         s!("a") => vec![s!("b=c&d❤")]
//     }), .. Request::default() };
//   expect!(request.to_json(&PactSpecification::V2).to_string()).to(
//     be_equal_to(r#"{"method":"GET","path":"/","query":"a=b%3dc%26d%27%64"}"#)
//   );
// }
//
// #[test]
// fn request_to_json_with_a_query_v3() {
//   let request = Request { query: Some(hashmap!{
//         s!("a") => vec![s!("1"), s!("2")],
//         s!("b") => vec![s!("3")]
//     }), .. Request::default() };
//   expect!(request.to_json(&PactSpecification::V3).to_string()).to(
//     be_equal_to(r#"{"method":"GET","path":"/","query":{"a":["1","2"],"b":["3"]}}"#)
//   );
// }
//
// #[test]
// fn request_to_json_with_a_query_v3_must_not_encode_the_query() {
//   let request = Request { query: Some(hashmap!{
//         s!("datetime") => vec![s!("2011-12-03T10:15:30+01:00")],
//         s!("description") => vec![s!("hello world!")] }), .. Request::default() };
//   expect!(request.to_json(&PactSpecification::V3).to_string()).to(
//     be_equal_to(r#"{"method":"GET","path":"/","query":{"datetime":["2011-12-03T10:15:30+01:00"],"description":["hello world!"]}}"#)
//   );
// }
//
// #[test]
// fn request_to_json_with_a_query_v3_must_not_encode_the_query_with_utf8_chars() {
//   let request = Request { query: Some(hashmap!{
//         s!("a") => vec![s!("b=c&d❤")]
//     }), .. Request::default() };
//   expect!(request.to_json(&PactSpecification::V3).to_string()).to(
//     be_equal_to(r#"{"method":"GET","path":"/","query":{"a":["b=c&d❤"]}}"#)
//   );
// }
//
// #[test]
// fn request_to_json_with_headers() {
//   let request = Request { headers: Some(hashmap!{
//         s!("HEADERA") => vec![s!("VALUEA")],
//         s!("HEADERB") => vec![s!("VALUEB1, VALUEB2")]
//     }), .. Request::default() };
//   expect!(request.to_json(&PactSpecification::V3).to_string()).to(
//     be_equal_to(r#"{"headers":{"HEADERA":"VALUEA","HEADERB":"VALUEB1, VALUEB2"},"method":"GET","path":"/"}"#)
//   );
// }
//
// #[test]
// fn request_to_json_with_json_body() {
//   let request = Request { headers: Some(hashmap!{
//         s!("Content-Type") => vec![s!("application/json")]
//     }), body: OptionalBody::Present(r#"{"key": "value"}"#.into(), None), .. Request::default() };
//   expect!(request.to_json(&PactSpecification::V3).to_string()).to(
//     be_equal_to(r#"{"body":{"key":"value"},"headers":{"Content-Type":"application/json"},"method":"GET","path":"/"}"#)
//   );
// }
//
//
// #[test]
// fn request_to_json_with_non_json_body() {
//   let request = Request { headers: Some(hashmap!{ s!("Content-Type") => vec![s!("text/plain")] }),
//     body: OptionalBody::Present("This is some text".into(), None), .. Request::default() };
//   expect!(request.to_json(&PactSpecification::V3).to_string()).to(
//     be_equal_to(r#"{"body":"This is some text","headers":{"Content-Type":"text/plain"},"method":"GET","path":"/"}"#)
//   );
// }
//
// #[test]
// fn request_to_json_with_empty_body() {
//   let request = Request { body: OptionalBody::Empty, .. Request::default() };
//   expect!(request.to_json(&PactSpecification::V3).to_string()).to(
//     be_equal_to(r#"{"body":"","method":"GET","path":"/"}"#)
//   );
// }
//
// #[test]
// fn request_to_json_with_null_body() {
//   let request = Request { body: OptionalBody::Null, .. Request::default() };
//   expect!(request.to_json(&PactSpecification::V3).to_string()).to(
//     be_equal_to(r#"{"body":null,"method":"GET","path":"/"}"#)
//   );
// }
//
// #[test]
// fn response_to_json_with_defaults() {
//   let response = Response::default();
//   expect!(response.to_json(&PactSpecification::V3).to_string()).to(be_equal_to("{\"status\":200}"));
// }
//
// #[test]
// fn response_to_json_with_headers() {
//   let response = Response { headers: Some(hashmap!{
//         s!("HEADERA") => vec![s!("VALUEA")],
//         s!("HEADERB") => vec![s!("VALUEB1, VALUEB2")]
//     }), .. Response::default() };
//   expect!(response.to_json(&PactSpecification::V3).to_string()).to(
//     be_equal_to(r#"{"headers":{"HEADERA":"VALUEA","HEADERB":"VALUEB1, VALUEB2"},"status":200}"#)
//   );
// }
//
// #[test]
// fn response_to_json_with_json_body() {
//   let response = Response { headers: Some(hashmap!{
//         s!("Content-Type") => vec![s!("application/json")]
//     }), body: OptionalBody::Present(r#"{"key": "value"}"#.into(), None), .. Response::default() };
//   expect!(response.to_json(&PactSpecification::V3).to_string()).to(
//     be_equal_to(r#"{"body":{"key":"value"},"headers":{"Content-Type":"application/json"},"status":200}"#)
//   );
// }
//
// #[test]
// fn response_to_json_with_non_json_body() {
//   let response = Response { headers: Some(hashmap!{ s!("Content-Type") => vec![s!("text/plain")] }),
//     body: OptionalBody::Present("This is some text".into(), None), .. Response::default() };
//   expect!(response.to_json(&PactSpecification::V3).to_string()).to(
//     be_equal_to(r#"{"body":"This is some text","headers":{"Content-Type":"text/plain"},"status":200}"#)
//   );
// }
//
// #[test]
// fn response_to_json_with_empty_body() {
//   let response = Response { body: OptionalBody::Empty, .. Response::default() };
//   expect!(response.to_json(&PactSpecification::V3).to_string()).to(
//     be_equal_to(r#"{"body":"","status":200}"#)
//   );
// }
//
// #[test]
// fn response_to_json_with_null_body() {
//   let response = Response { body: OptionalBody::Null, .. Response::default() };
//   expect!(response.to_json(&PactSpecification::V3).to_string()).to(
//     be_equal_to(r#"{"body":null,"status":200}"#)
//   );
// }
//
// #[test]
// fn interaction_from_json_sets_the_id_if_loaded_from_broker() {
//   let json = json!({
//     "_id": "123456789",
//     "description": "Test Interaction",
//     "providerState": "Good state to be in",
//     "request": {
//       "method": "GET",
//       "path": "/"
//     },
//     "response": {
//       "status": 200
//     }
//   });
//   expect!(RequestResponseInteraction::from_json(0, &json, &PactSpecification::V3).id).to(be_some().value("123456789".to_string()));
// }
//
// fn read_pact_file(file: &str) -> io::Result<String> {
//   let mut f = File::open(file)?;
//   let mut buffer = String::new();
//   f.read_to_string(&mut buffer)?;
//   Ok(buffer)
// }
//
// #[test]
// fn write_pact_test() {
//   let pact = RequestResponsePact { consumer: Consumer { name: s!("write_pact_test_consumer") },
//     provider: Provider { name: s!("write_pact_test_provider") },
//     interactions: vec![
//       RequestResponseInteraction {
//         description: s!("Test Interaction"),
//         provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
//         .. RequestResponseInteraction::default()
//       }
//     ],
//     .. RequestResponsePact::default() };
//   let mut dir = env::temp_dir();
//   let x = rand::random::<u16>();
//   dir.push(format!("pact_test_{}", x));
//   dir.push(pact.default_file_name());
//
//   let result = pact.write_pact(dir.as_path(), PactSpecification::V2);
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
//       "providerState": "Good state to be in",
//       "request": {{
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
//       "version": "2.0.0"
//     }}
//   }},
//   "provider": {{
//     "name": "write_pact_test_provider"
//   }}
// }}"#, super::VERSION.unwrap())));
// }
//
// #[test]
// fn write_pact_test_should_merge_pacts() {
//   let pact = RequestResponsePact { consumer: Consumer { name: s!("merge_consumer") },
//     provider: Provider { name: s!("merge_provider") },
//     interactions: vec![
//       RequestResponseInteraction {
//         description: s!("Test Interaction 2"),
//         provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
//         .. RequestResponseInteraction::default()
//       }
//     ],
//     metadata: btreemap!{},
//     specification_version: PactSpecification::V1_1
//   };
//   let pact2 = RequestResponsePact { consumer: Consumer { name: s!("merge_consumer") },
//     provider: Provider { name: s!("merge_provider") },
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
//   let mut dir = env::temp_dir();
//   let x = rand::random::<u16>();
//   dir.push(format!("pact_test_{}", x));
//   dir.push(pact.default_file_name());
//
//   let result = pact.write_pact(dir.as_path(), PactSpecification::V2);
//   let result2 = pact2.write_pact(dir.as_path(), PactSpecification::V2);
//
//   let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or(s!(""));
//   fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());
//
//   expect!(result).to(be_ok());
//   expect!(result2).to(be_ok());
//   expect!(pact_file).to(be_equal_to(format!(r#"{{
//   "consumer": {{
//     "name": "merge_consumer"
//   }},
//   "interactions": [
//     {{
//       "description": "Test Interaction",
//       "providerState": "Good state to be in",
//       "request": {{
//         "method": "GET",
//         "path": "/"
//       }},
//       "response": {{
//         "status": 200
//       }}
//     }},
//     {{
//       "description": "Test Interaction 2",
//       "providerState": "Good state to be in",
//       "request": {{
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
//       "version": "2.0.0"
//     }}
//   }},
//   "provider": {{
//     "name": "merge_provider"
//   }}
// }}"#, super::VERSION.unwrap())));
// }
//
// #[test]
// fn write_pact_test_should_not_merge_pacts_with_conflicts() {
//   let pact = RequestResponsePact { consumer: Consumer { name: s!("write_pact_test_consumer") },
//     provider: Provider { name: s!("write_pact_test_provider") },
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
//   let pact2 = RequestResponsePact { consumer: Consumer { name: s!("write_pact_test_consumer") },
//     provider: Provider { name: s!("write_pact_test_provider") },
//     interactions: vec![
//       RequestResponseInteraction {
//         description: s!("Test Interaction"),
//         provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
//         response: Response { status: 400, .. Response::default() },
//         .. RequestResponseInteraction::default()
//       }
//     ],
//     metadata: btreemap!{},
//     specification_version: PactSpecification::V1_1
//   };
//   let mut dir = env::temp_dir();
//   let x = rand::random::<u16>();
//   dir.push(format!("pact_test_{}", x));
//   dir.push(pact.default_file_name());
//
//   let result = pact.write_pact(dir.as_path(), PactSpecification::V2);
//   let result2 = pact2.write_pact(dir.as_path(), PactSpecification::V2);
//
//   let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or(s!(""));
//   fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());
//
//   expect!(result).to(be_ok());
//   expect!(result2).to(be_err());
//   expect!(pact_file).to(be_equal_to(format!(r#"{{
//   "consumer": {{
//     "name": "write_pact_test_consumer"
//   }},
//   "interactions": [
//     {{
//       "description": "Test Interaction",
//       "providerState": "Good state to be in",
//       "request": {{
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
//       "version": "2.0.0"
//     }}
//   }},
//   "provider": {{
//     "name": "write_pact_test_provider"
//   }}
// }}"#, super::VERSION.unwrap())));
// }
//
// #[test]
// fn pact_merge_does_not_merge_different_consumers() {
//   let pact = RequestResponsePact { consumer: Consumer { name: s!("test_consumer") },
//     provider: Provider { name: s!("test_provider") },
//     interactions: vec![],
//     metadata: btreemap!{},
//     specification_version: PactSpecification::V1
//   };
//   let pact2 = RequestResponsePact { consumer: Consumer { name: s!("test_consumer2") },
//     provider: Provider { name: s!("test_provider") },
//     interactions: vec![],
//     metadata: btreemap!{},
//     specification_version: PactSpecification::V1_1
//   };
//   expect!(pact.merge(&pact2)).to(be_err());
// }
//
// #[test]
// fn pact_merge_does_not_merge_different_providers() {
//   let pact = RequestResponsePact { consumer: Consumer { name: s!("test_consumer") },
//     provider: Provider { name: s!("test_provider") },
//     interactions: vec![],
//     metadata: btreemap!{},
//     specification_version: PactSpecification::V1_1
//   };
//   let pact2 = RequestResponsePact { consumer: Consumer { name: s!("test_consumer") },
//     provider: Provider { name: s!("test_provider2") },
//     interactions: vec![],
//     metadata: btreemap!{},
//     specification_version: PactSpecification::V1_1
//   };
//   expect!(pact.merge(&pact2)).to(be_err());
// }
//
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
//
// #[test]
// fn pact_merge_removes_duplicates() {
//   let pact = RequestResponsePact { consumer: Consumer { name: s!("test_consumer") },
//     provider: Provider { name: s!("test_provider") },
//     interactions: vec![
//       RequestResponseInteraction {
//         description: s!("Test Interaction"),
//         provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
//         .. RequestResponseInteraction::default()
//       }
//     ],
//     .. RequestResponsePact::default()
//   };
//   let pact2 = RequestResponsePact { consumer: Consumer { name: s!("test_consumer") },
//     provider: Provider { name: s!("test_provider") },
//     interactions: vec![
//       RequestResponseInteraction {
//         description: s!("Test Interaction"),
//         provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
//         .. RequestResponseInteraction::default()
//       },
//       RequestResponseInteraction {
//         description: s!("Test Interaction 2"),
//         provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
//         .. RequestResponseInteraction::default()
//       }
//     ],
//     .. RequestResponsePact::default()
//   };
//
//   let merged_pact = pact.merge(&pact2);
//   expect!(merged_pact.clone()).to(be_ok());
//   expect!(merged_pact.clone().unwrap().interactions.len()).to(be_equal_to(2));
//
//   let merged_pact2 = pact.merge(&pact.clone());
//   expect!(merged_pact2.clone()).to(be_ok());
//   expect!(merged_pact2.clone().unwrap().interactions.len()).to(be_equal_to(1));
// }
//
// #[test]
// fn interactions_do_not_conflict_if_they_have_different_descriptions() {
//   let interaction1 = RequestResponseInteraction {
//     description: s!("Test Interaction"),
//     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
//     .. RequestResponseInteraction::default()
//   };
//   let interaction2 = RequestResponseInteraction {
//     description: s!("Test Interaction 2"),
//     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
//     .. RequestResponseInteraction::default()
//   };
//   expect!(interaction1.conflicts_with(&interaction2).iter()).to(be_empty());
// }
//
// #[test]
// fn interactions_do_not_conflict_if_they_have_different_provider_states() {
//   let interaction1 = RequestResponseInteraction {
//     description: s!("Test Interaction"),
//     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
//     .. RequestResponseInteraction::default()
//   };
//   let interaction2 = RequestResponseInteraction {
//     description: s!("Test Interaction"),
//     provider_states: vec![ProviderState { name: s!("Bad state to be in"), params: hashmap!{} }],
//     .. RequestResponseInteraction::default()
//   };
//   expect!(interaction1.conflicts_with(&interaction2).iter()).to(be_empty());
// }
//
// #[test]
// fn interactions_do_not_conflict_if_they_have_the_same_requests_and_responses() {
//   let interaction1 = RequestResponseInteraction {
//     description: s!("Test Interaction"),
//     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
//     .. RequestResponseInteraction::default()
//   };
//   let interaction2 = RequestResponseInteraction {
//     description: s!("Test Interaction"),
//     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
//     .. RequestResponseInteraction::default()
//   };
//   expect!(interaction1.conflicts_with(&interaction2).iter()).to(be_empty());
// }
//
// #[test]
// fn interactions_conflict_if_they_have_different_requests() {
//   let interaction1 = RequestResponseInteraction {
//     description: s!("Test Interaction"),
//     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
//     .. RequestResponseInteraction::default()
//   };
//   let interaction2 = RequestResponseInteraction {
//     description: s!("Test Interaction"),
//     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
//     request: Request { method: s!("POST"), .. Request::default() },
//     .. RequestResponseInteraction::default()
//   };
//   expect!(interaction1.conflicts_with(&interaction2).iter()).to_not(be_empty());
// }
//
// #[test]
// fn interactions_conflict_if_they_have_different_responses() {
//   let interaction1 = RequestResponseInteraction {
//     description: s!("Test Interaction"),
//     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
//     .. RequestResponseInteraction::default()
//   };
//   let interaction2 = RequestResponseInteraction {
//     description: s!("Test Interaction"),
//     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
//     response: Response { status: 400, .. Response::default() },
//     .. RequestResponseInteraction::default()
//   };
//   expect!(interaction1.conflicts_with(&interaction2).iter()).to_not(be_empty());
// }
//
// fn hash<T: Hash>(t: &T) -> u64 {
//   let mut s = DefaultHasher::new();
//   t.hash(&mut s);
//   s.finish()
// }
//
// #[test]
// fn hash_for_request() {
//   let request1 = Request::default();
//   let request2 = Request { method: s!("POST"), .. Request::default() };
//   let request3 = Request { headers: Some(hashmap!{
//         s!("H1") => vec![s!("A")]
//     }), .. Request::default() };
//   let request4 = Request { headers: Some(hashmap!{
//         s!("H1") => vec![s!("B")]
//     }), .. Request::default() };
//   expect!(hash(&request1)).to(be_equal_to(hash(&request1)));
//   expect!(hash(&request3)).to(be_equal_to(hash(&request3)));
//   expect!(hash(&request1)).to_not(be_equal_to(hash(&request2)));
//   expect!(hash(&request3)).to_not(be_equal_to(hash(&request4)));
// }
//
// #[test]
// fn hash_for_response() {
//   let response1 = Response::default();
//   let response2 = Response { status: 400, .. Response::default() };
//   let response3 = Response { headers: Some(hashmap!{
//         s!("H1") => vec![s!("A")]
//     }), .. Response::default() };
//   let response4 = Response { headers: Some(hashmap!{
//         s!("H1") => vec![s!("B")]
//     }), .. Response::default() };
//   expect!(hash(&response1)).to(be_equal_to(hash(&response1)));
//   expect!(hash(&response3)).to(be_equal_to(hash(&response3)));
//   expect!(hash(&response1)).to_not(be_equal_to(hash(&response2)));
//   expect!(hash(&response3)).to_not(be_equal_to(hash(&response4)));
// }
//
// #[test]
// fn write_pact_test_with_matchers() {
//   let pact = RequestResponsePact { consumer: Consumer { name: s!("write_pact_test_consumer") },
//     provider: Provider { name: s!("write_pact_test_provider") },
//     interactions: vec![
//       RequestResponseInteraction {
//         description: s!("Test Interaction"),
//         provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
//         request: Request {
//           matching_rules: matchingrules!{
//                         "body" => {
//                             "$" => [ MatchingRule::Type ]
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
//   let result = pact.write_pact(dir.as_path(), PactSpecification::V2);
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
//       "providerState": "Good state to be in",
//       "request": {{
//         "matchingRules": {{
//           "$.body": {{
//             "match": "type"
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
//       "version": "2.0.0"
//     }}
//   }},
//   "provider": {{
//     "name": "write_pact_test_provider"
//   }}
// }}"#, super::VERSION.unwrap())));
// }
//
// #[test]
// fn write_pact_v3_test_with_matchers() {
//   let pact = RequestResponsePact { consumer: Consumer { name: s!("write_pact_test_consumer_v3") },
//     provider: Provider { name: s!("write_pact_test_provider_v3") },
//     interactions: vec![
//       RequestResponseInteraction {
//         description: s!("Test Interaction"),
//         provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
//         request: Request {
//           matching_rules: matchingrules!{
//                         "body" => {
//                             "$" => [ MatchingRule::Type ]
//                         },
//                         "header" => {
//                           "HEADER_A" => [ MatchingRule::Include(s!("ValA")), MatchingRule::Include(s!("ValB")) ]
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
//     "name": "write_pact_test_consumer_v3"
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
//         "matchingRules": {{
//           "body": {{
//             "$": {{
//               "combine": "AND",
//               "matchers": [
//                 {{
//                   "match": "type"
//                 }}
//               ]
//             }}
//           }},
//           "header": {{
//             "HEADER_A": {{
//               "combine": "AND",
//               "matchers": [
//                 {{
//                   "match": "include",
//                   "value": "ValA"
//                 }},
//                 {{
//                   "match": "include",
//                   "value": "ValB"
//                 }}
//               ]
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
//     "name": "write_pact_test_provider_v3"
//   }}
// }}"#, super::VERSION.unwrap())));
// }

#[test]
fn body_from_json_returns_missing_if_there_is_no_body() {
  let json = json!({});
  let body = body_from_json(&json, "body", &None);
  expect!(body).to(be_equal_to(OptionalBody::Missing));
}

#[test]
fn body_from_json_returns_null_if_the_body_is_null() {
  let json = json!({
    "path": "/",
    "query": "",
    "headers": {},
    "body": null
  });
  let body = body_from_json(&json, "body", &None);
  expect!(body).to(be_equal_to(OptionalBody::Null));
}

#[test]
fn body_from_json_returns_json_string_if_the_body_is_json_but_not_a_string() {
  let json = json!({
    "path": "/",
    "query": "",
    "headers": {},
    "body": {
      "content": {
        "test": true
      }
    }
  });
  let body = body_from_json(&json, "body", &None);
  expect!(body).to(be_equal_to(OptionalBody::Present("{\"test\":true}".into(),
                                                     Some("application/json".to_string()))));
}

#[test]
fn body_from_json_returns_empty_if_the_body_is_an_empty_string() {
  let json = json!({
    "path": "/",
    "query": "",
    "headers": {},
    "body": {
      "content": ""
    }
  });
  let body = body_from_json(&json, "body", &None);
  expect!(body).to(be_equal_to(OptionalBody::Empty));
}

#[test]
fn body_from_json_returns_the_body_if_the_body_is_a_string() {
  let json = json!({
    "path": "/",
    "query": "",
    "headers": {},
    "body": {
      "content": "<?xml version=\"1.0\"?> <body></body>"
    }
  });
  let body = body_from_json(&json, "body", &None);
  expect!(body).to(be_equal_to(
    OptionalBody::Present("<?xml version=\"1.0\"?> <body></body>".into(),
                          Some("application/xml".into()))));
}

#[test]
fn body_from_text_plain_type_returns_the_same_formatted_body() {
  let json = json!({
    "path": "/",
    "query": "",
    "headers": {"Content-Type": "text/plain"},
    "body": {
      "content": "\"This is a string\""
    }
  });
  let headers = headers_from_json(&json);
  let body = body_from_json(&json, "body", &headers);
  expect!(body).to(be_equal_to(OptionalBody::Present("\"This is a string\"".into(), Some("text/plain".into()))));
}

#[test]
fn body_from_text_html_type_returns_the_same_formatted_body() {
  let json = json!({
    "path": "/",
    "query": "",
    "headers": {"Content-Type": "text/html"},
    "body": {
      "content": "\"This is a string\""
    }
  });
  let headers = headers_from_json(&json);
  let body = body_from_json(&json, "body", &headers);
  expect!(body).to(be_equal_to(OptionalBody::Present("\"This is a string\"".into(), Some("text/html".into()))));
}

#[test]
fn body_from_json_returns_the_a_json_formatted_body_if_the_body_is_a_string_and_encoding_is_json() {
  let json = json!({
    "body": {
      "content": "This is actually a JSON string",
      "contentType": "application/json",
      "encoded": "json"
    }
  });
  let body = body_from_json(&json, "body", &None);
  expect!(body).to(be_equal_to(OptionalBody::Present("\"This is actually a JSON string\"".into(), Some("application/json".into()))));
}

// #[test]
// fn body_from_json_returns_the_a_json_formatted_body_if_the_body_is_a_valid_json_string_and_the_content_type_is_json() {
//   let json : serde_json::Value = serde_json::from_str(r#"
//       {
//           "path": "/",
//           "query": "",
//           "headers": {"Content-Type": "application/json"},
//           "body": "\"This is actually a JSON string\""
//       }
//      "#).unwrap();
//   let headers = headers_from_json(&json);
//   let body = body_from_json(&json, "body", &headers);
//   expect!(body).to(be_equal_to(OptionalBody::Present("\"This is actually a JSON string\"".into(), Some("application/json".into()))));
// }
//
// #[test]
// fn body_from_json_returns_the_body_if_the_content_type_is_json() {
//   let json : serde_json::Value = serde_json::from_str(r#"
//       {
//           "path": "/",
//           "query": "",
//           "headers": {"Content-Type": "application/json"},
//           "body": "{\"test\":true}"
//       }
//      "#).unwrap();
//   let headers = headers_from_json(&json);
//   let body = body_from_json(&json, "body", &headers);
//   expect!(body).to(be_equal_to(OptionalBody::Present("{\"test\":true}".into(), Some("application/json".into()))));
// }
//
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
// fn generators_from_json_handles_missing_generators() {
//   let json : serde_json::Value = serde_json::from_str(r#"
//       {
//           "path": "/",
//           "query": "",
//           "headers": {}
//       }
//      "#).unwrap();
//   let generators = generators_from_json(&json);
//   expect!(generators.categories.iter()).to(be_empty());
// }
//
// #[test]
// fn generators_from_json_handles_empty_generators() {
//   let json : serde_json::Value = serde_json::from_str(r#"
//       {
//           "path": "/",
//           "query": "",
//           "headers": {},
//           "generators": {}
//       }
//      "#).unwrap();
//   let generators = generators_from_json(&json);
//   expect!(generators.categories.iter()).to(be_empty());
// }
//
// #[test]
// fn generators_from_json_handles_generator_with_no_rules() {
//   let json : serde_json::Value = serde_json::from_str(r#"
//       {
//           "path": "/",
//           "query": "",
//           "headers": {},
//           "generators": {
//             "body": {
//                 "$.*.path": {}
//             }
//           }
//       }
//      "#).unwrap();
//   let generators = generators_from_json(&json);
//   expect!(generators).to(be_equal_to(Generators::default()));
// }
//
// #[test]
// fn generators_from_json_ignores_invalid_generators() {
//   let json : serde_json::Value = serde_json::from_str(r#"
//       {
//           "path": "/",
//           "query": "",
//           "headers": {},
//           "generators": {
//             "body": {
//                 "$.*.path": {
//                   "type": "invalid"
//                 },
//                 "$.invalid": {
//                   "type": 100
//                 },
//                 "$.other": null
//             },
//             "invalid": {
//                 "path": "path"
//             },
//             "more_invalid": 100
//           }
//       }
//      "#).unwrap();
//   let generators = generators_from_json(&json);
//   expect!(generators).to(be_equal_to(Generators::default()));
// }
//
// #[test]
// fn generators_from_json_loads_generators_correctly() {
//   let json : serde_json::Value = serde_json::from_str(r#"
//       {
//         "path": "/",
//         "query": "",
//         "headers": {},
//         "generators": {
//           "body": {
//               "$.*.path": {
//                   "type": "RandomInt",
//                   "min": 1,
//                   "max": 10
//               }
//           },
//           "path": {
//             "type": "RandomString"
//           }
//         }
//       }
//      "#).unwrap();
//   let generators = generators_from_json(&json);
//   expect!(generators).to(be_equal_to(generators!{
//         "BODY" => {
//             "$.*.path" => Generator::RandomInt(1, 10)
//         },
//         "PATH" => { "" => Generator::RandomString(10) }
//     }));
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
