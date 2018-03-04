use super::*;
use expectest::prelude::*;
use models::{Request, Response, OptionalBody, DetectedContentType};
use models::generators::{QueryResult, JsonHandler, ContentTypeHandler};
use std::str::FromStr;
use serde_json::Value;
use std::cell::RefCell;

#[test]
fn returns_original_response_if_there_are_no_generators() {
  let response = Response::default_response();
  expect!(generate_response(&response)).to(be_equal_to(response));
}

#[test]
fn applies_status_generator_for_status_to_the_copy_of_the_response() {
  let response = Response { status: 200, generators: generators! {
    "STATUS" => Generator::RandomInt(400, 499)
  }, .. Response::default_response() };
  expect!(generate_response(&response).status).to(be_greater_or_equal_to(400));
}

#[test]
fn applies_header_generator_for_headers_to_the_copy_of_the_response() {
  let response = Response { headers: Some(hashmap!{
      s!("A") => s!("a"),
      s!("B") => s!("b")
    }), generators: generators! {
      "HEADER" => {
        "A" => Generator::Uuid
      }
    }, .. Response::default_response()
  };
  let headers = generate_response(&response).headers.unwrap().clone();
  expect!(headers.get("A").unwrap()).to_not(be_equal_to("a"));
}

#[test]
fn returns_original_request_if_there_are_no_generators() {
  let request = Request::default_request();
  expect!(generate_request(&request)).to(be_equal_to(request));
}

#[test]
fn applies_path_generator_for_the_path_to_the_copy_of_the_request() {
  let request = Request { path: s!("/path"), generators: generators! {
    "PATH" => Generator::RandomInt(1, 10)
  }, .. Request::default_request() };
  expect!(generate_request(&request).path).to_not(be_equal_to("/path"));
}

#[test]
fn applies_header_generator_for_headers_to_the_copy_of_the_request() {
  let request = Request { headers: Some(hashmap!{
      s!("A") => s!("a"),
      s!("B") => s!("b")
    }), generators: generators! {
      "HEADER" => {
        "A" => Generator::Uuid
      }
    }, .. Request::default_request()
  };
  let headers = generate_request(&request).headers.unwrap().clone();
  expect!(headers.get("A").unwrap()).to_not(be_equal_to("a"));
}

#[test]
fn applies_query_generator_for_query_parameters_to_the_copy_of_the_request() {
  let request = Request { query: Some(hashmap!{
      s!("A") => vec![ s!("a") ],
      s!("B") => vec![ s!("b") ]
    }), generators: generators! {
      "QUERY" => {
        "A" => Generator::Uuid
      }
    }, .. Request::default_request()
  };
  let query = generate_request(&request).query.unwrap().clone();
  let query_val = &query.get("A").unwrap()[0];
  expect!(query_val).to_not(be_equal_to("a"));
}

#[test]
fn apply_generator_to_empty_body_test() {
  let generators = Generators::default();
  expect!(generators.apply_body_generators(&OptionalBody::Empty, DetectedContentType::Text)).to(be_equal_to(OptionalBody::Empty));
  expect!(generators.apply_body_generators(&OptionalBody::Null, DetectedContentType::Text)).to(be_equal_to(OptionalBody::Null));
  expect!(generators.apply_body_generators(&OptionalBody::Missing, DetectedContentType::Text)).to(be_equal_to(OptionalBody::Missing));
}

#[test]
fn apply_generator_to_text_body_test() {
  let generators = Generators::default();
  let body = OptionalBody::Present("some text".into());
  expect!(generators.apply_body_generators(&body, DetectedContentType::Text)).to(be_equal_to(body));
}

#[test]
#[ignore]
fn applies_body_generator_to_the_copy_of_the_request() {
  let request = Request { body: OptionalBody::Present("{\"a\": 100, \"b\": \"B\"}".into()),
    generators: generators! {
      "BODY" => {
        "a" => Generator::RandomInt(1, 10)
      }
    }, .. Request::default_request()
  };
  let body: Value = serde_json::from_str(generate_request(&request).body.str_value()).unwrap();
  expect!(&body["a"]).to_not(be_equal_to(&json!(100)));
  expect!(&body["b"]).to(be_equal_to(&json!("B")));
}

#[test]
#[ignore]
fn applies_body_generator_to_the_copy_of_the_response() {
  let response = Response { body: OptionalBody::Present("{\"a\": 100, \"b\": \"B\"}".into()),
    generators: generators! {
      "BODY" => {
        "a" => Generator::RandomInt(1, 10)
      }
    }, .. Response::default_response()
  };
  let body: Value = serde_json::from_str(generate_response(&response).body.str_value()).unwrap();
  expect!(&body["a"]).to_not(be_equal_to(&json!(100)));
  expect!(&body["b"]).to(be_equal_to(&json!("B")));
}

#[test]
fn does_not_change_body_if_there_are_no_generators() {
  let body = OptionalBody::Present("{\"a\": 100, \"b\": \"B\"}".into());
  let generators = generators!{};
  let processed = generators.apply_body_generators(&body, DetectedContentType::Json);
  expect!(processed).to(be_equal_to(body));
}

#[test]
fn applies_the_generator_to_a_json_map_entry() {
  let map = json!({"a": 100, "b": "B", "c": "C"});
  let json_handler = JsonHandler { value: map.clone() };
  let mut query_result = QueryResult::default(map);
  json_handler.apply_key(query_result.clone(), &s!("$.b"), &Generator::RandomInt(0, 10));
  p!(query_result);
  expect!(&query_result.value["b"]).to(be_equal_to(&json!("B")));
}

#[test]
fn json_generator_handles_invalid_path_expressions() {
  let map = json!({"a": 100, "b": "B", "c": "C"});
  let json_handler = JsonHandler { value: map.clone() };
  let mut query_result = QueryResult::default(map.clone());
  json_handler.apply_key(query_result.clone(), &s!("$["), &Generator::RandomInt(0, 10));

  expect!(query_result.value).to(be_equal_to(map));
}

#[test]
fn does_not_apply_the_generator_when_field_is_not_in_map() {
  let map = json!({"a": 100, "b": "B", "c": "C"});
  let expected = map.clone();
  let json_handler = JsonHandler { value: map.clone() };
  let mut query_result = QueryResult::default(map.clone());
  json_handler.apply_key(query_result.clone(), &s!("$.d"), &Generator::RandomInt(0, 10));
  expect!(query_result.value).to(be_equal_to(expected));
}

/*

  def 'does not apply the generator when field is not in map'() {
    given:
    def map = [a: 'A', b: 'B', c: 'C']
    QueryResult body = new QueryResult(map, null, null)
    def key = '$.d'
    def generator = { 'X' } as Generator

    when:
    JsonContentTypeHandler.INSTANCE.applyKey(body, key, generator)

    then:
    body.value == [a: 'A', b: 'B', c: 'C']
  }

  def 'does not apply the generator when not a map'() {
    given:
    QueryResult body = new QueryResult(100, null, null)
    def key = '$.d'
    def generator = { 'X' } as Generator

    when:
    JsonContentTypeHandler.INSTANCE.applyKey(body, key, generator)

    then:
    body.value == 100
  }

  def 'applies the generator to a list item'() {
    given:
    def list = ['A', 'B', 'C']
    QueryResult body = new QueryResult(list, null, null)
    def key = '$[1]'
    def generator = { 'X' } as Generator

    when:
    JsonContentTypeHandler.INSTANCE.applyKey(body, key, generator)

    then:
    body.value == ['A', 'X', 'C']
  }

  def 'does not apply the generator if the index is not in the list'() {
    given:
    def list = ['A', 'B', 'C']
    QueryResult body = new QueryResult(list, null, null)
    def key = '$[3]'
    def generator = { 'X' } as Generator

    when:
    JsonContentTypeHandler.INSTANCE.applyKey(body, key, generator)

    then:
    body.value == ['A', 'B', 'C']
  }

  def 'does not apply the generator when not a list'() {
    given:
    QueryResult body = new QueryResult(100, null, null)
    def key = '$[3]'
    def generator = { 'X' } as Generator

    when:
    JsonContentTypeHandler.INSTANCE.applyKey(body, key, generator)

    then:
    body.value == 100
  }

  def 'applies the generator to the root'() {
    given:
    def bodyValue = 100
    QueryResult body = new QueryResult(bodyValue, null, null)
    def key = '$'
    def generator = { 'X' } as Generator

    when:
    JsonContentTypeHandler.INSTANCE.applyKey(body, key, generator)

    then:
    body.value == 'X'
  }

  def 'applies the generator to the object graph'() {
    given:
    def graph = [a: ['A', [a: 'A', b: ['1': '1', '2': '2'], c: 'C'], 'C'], b: 'B', c: 'C']
    QueryResult body = new QueryResult(graph, null, null)
    def key = '$.a[1].b[\'2\']'
    def generator = { 'X' } as Generator

    when:
    JsonContentTypeHandler.INSTANCE.applyKey(body, key, generator)

    then:
    body.value == [a: ['A', [a: 'A', b: ['1': '1', '2': 'X'], c: 'C'], 'C'], b: 'B', c: 'C']
  }

  def 'does not apply the generator to the object graph when the expression does not match'() {
    given:
    def graph = [d: 'A', b: 'B', c: 'C']
    QueryResult body = new QueryResult(graph, null, null)
    def key = '$.a[1].b[\'2\']'
    def generator = { 'X' } as Generator

    when:
    JsonContentTypeHandler.INSTANCE.applyKey(body, key, generator)

    then:
    body.value == [d: 'A', b: 'B', c: 'C']
  }

  def 'applies the generator to all map entries'() {
    given:
    def map = [a: 'A', b: 'B', c: 'C']
    QueryResult body = new QueryResult(map, null, null)
    def key = '$.*'
    def generator = { 'X' } as Generator

    when:
    JsonContentTypeHandler.INSTANCE.applyKey(body, key, generator)

    then:
    body.value == [a: 'X', b: 'X', c: 'X']
  }

  def 'applies the generator to all list items'() {
    given:
    def list = ['A', 'B', 'C']
    QueryResult body = new QueryResult(list, null, null)
    def key = '$[*]'
    def generator = { 'X' } as Generator

    when:
    JsonContentTypeHandler.INSTANCE.applyKey(body, key, generator)

    then:
    body.value == ['X', 'X', 'X']
  }

  def 'applies the generator to the object graph with wildcard'() {
    given:
    def graph = [a: ['A', [a: 'A', b: ['1', '2'], c: 'C'], 'C'], b: 'B', c: 'C']
    QueryResult body = new QueryResult(graph, null, null)
    def key = '$.*[1].b[*]'
    def generator = { 'X' } as Generator

    when:
    JsonContentTypeHandler.INSTANCE.applyKey(body, key, generator)

    then:
    body.value == [a: ['A', [a: 'A', b: ['X', 'X'], c: 'C'], 'C'], b: 'B', c: 'C']
  }
*/
