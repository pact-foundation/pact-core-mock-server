#[allow(unused_imports)]
use env_logger;
#[allow(unused_imports)]
use pact_matching::models::PactSpecification;
#[allow(unused_imports)]
use pact_matching::models::Response;
#[allow(unused_imports)]
use pact_matching::match_response;
#[allow(unused_imports)]
use expectest::prelude::*;
use expectest::*;
#[allow(unused_imports)]
use serde_json;

#[test]
fn content_type_parameters_do_not_match() {
    env_logger::init().unwrap_or(());
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Headers don't match when the parameters are different",
        "expected" : {
          "headers": {
            "Content-Type": "application/json; charset=UTF-16"
          }
        },
        "actual": {
          "headers": {
            "Content-Type": "application/json; charset=UTF-8"
          }
        }
      }
    "#).unwrap();

    let expected = Response::from_json(&pact.get("expected").unwrap(), &PactSpecification::V3);
    println!("{:?}", expected);
    let actual = Response::from_json(&pact.get("actual").unwrap(), &PactSpecification::V3);
    println!("{:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_response(expected, actual);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn empty_headers() {
    env_logger::init().unwrap_or(());
    let pact : serde_json::Value = serde_json::from_str(r#"
        {
        "match": true,
        "comment": "Empty headers match",
        "expected" : {
          "headers": {}
        },
        "actual": {
          "headers": {}
        }
      }
    "#).unwrap();

    let expected = Response::from_json(&pact.get("expected").unwrap(), &PactSpecification::V3);
    println!("{:?}", expected);
    let actual = Response::from_json(&pact.get("actual").unwrap(), &PactSpecification::V3);
    println!("{:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_response(expected, actual);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn header_name_is_different_case() {
    env_logger::init().unwrap_or(());
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Header name is case insensitive",
        "expected" : {
          "headers": {
            "Accept": "alligators"
          }
        },
        "actual": {
          "headers": {
            "ACCEPT": "alligators"
          }
        }
      }
    "#).unwrap();

    let expected = Response::from_json(&pact.get("expected").unwrap(), &PactSpecification::V3);
    println!("{:?}", expected);
    let actual = Response::from_json(&pact.get("actual").unwrap(), &PactSpecification::V3);
    println!("{:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_response(expected, actual);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn header_value_is_different_case() {
    env_logger::init().unwrap_or(());
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Headers values are case sensitive",
        "expected" : {
          "headers": {
            "Accept": "alligators"
          }
        },
        "actual": {
          "headers": {
            "Accept": "Alligators"
          }
        }
      }
    "#).unwrap();

    let expected = Response::from_json(&pact.get("expected").unwrap(), &PactSpecification::V3);
    println!("{:?}", expected);
    let actual = Response::from_json(&pact.get("actual").unwrap(), &PactSpecification::V3);
    println!("{:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_response(expected, actual);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn matches_content_type_with_charset() {
    env_logger::init().unwrap_or(());
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Headers match when the actual includes additional parameters",
        "expected" : {
          "headers": {
            "Content-Type": "application/json"
          }
        },
        "actual": {
          "headers": {
            "Content-Type": "application/json; charset=UTF-8"
          }
        }
      }
    "#).unwrap();

    let expected = Response::from_json(&pact.get("expected").unwrap(), &PactSpecification::V3);
    println!("{:?}", expected);
    let actual = Response::from_json(&pact.get("actual").unwrap(), &PactSpecification::V3);
    println!("{:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_response(expected, actual);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn matches_content_type_with_parameters_in_different_order() {
    env_logger::init().unwrap_or(());
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Headers match when the content type parameters are in a different order",
        "expected" : {
          "headers": {
            "Content-Type": "Text/x-Okie; charset=iso-8859-1;\n    declaration=\"<950118.AEB0@XIson.com>\""
          }
        },
        "actual": {
          "headers": {
            "Content-Type": "Text/x-Okie; declaration=\"<950118.AEB0@XIson.com>\";\n    charset=iso-8859-1"
          }
        }
      }
    "#).unwrap();

    let expected = Response::from_json(&pact.get("expected").unwrap(), &PactSpecification::V3);
    println!("{:?}", expected);
    let actual = Response::from_json(&pact.get("actual").unwrap(), &PactSpecification::V3);
    println!("{:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_response(expected, actual);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn matches_with_regex() {
    env_logger::init().unwrap_or(());
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Headers match with regex",
        "expected" : {
          "headers": {
            "Accept": "alligators",
            "Content-Type": "hippos"
          },
          "matchingRules": {
            "header": {
              "Accept": {
                "matchers": [
                  {
                    "match": "regex",
                    "regex": "\\w+"
                  }
                ]
              }
            }
          }
        },
        "actual": {
          "headers": {
            "Content-Type": "hippos",
            "Accept": "godzilla"
          }
        }
      }
    "#).unwrap();

    let expected = Response::from_json(&pact.get("expected").unwrap(), &PactSpecification::V3);
    println!("{:?}", expected);
    let actual = Response::from_json(&pact.get("actual").unwrap(), &PactSpecification::V3);
    println!("{:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_response(expected, actual);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn matches() {
    env_logger::init().unwrap_or(());
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Headers match",
        "expected" : {
          "headers": {
            "Accept": "alligators",
            "Content-Type": "hippos"
          }
        },
        "actual": {
          "headers": {
            "Content-Type": "hippos",
            "Accept": "alligators"
          }
        }
      }
    "#).unwrap();

    let expected = Response::from_json(&pact.get("expected").unwrap(), &PactSpecification::V3);
    println!("{:?}", expected);
    let actual = Response::from_json(&pact.get("actual").unwrap(), &PactSpecification::V3);
    println!("{:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_response(expected, actual);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn order_of_comma_separated_header_values_different() {
    env_logger::init().unwrap_or(());
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Comma separated headers out of order, order can matter http://tools.ietf.org/html/rfc2616",
        "expected" : {
          "headers": {
            "Accept": "alligators, hippos"
          }
        },
        "actual": {
          "headers": {
            "Accept": "hippos, alligators"
          }
        }
      }
    "#).unwrap();

    let expected = Response::from_json(&pact.get("expected").unwrap(), &PactSpecification::V3);
    println!("{:?}", expected);
    let actual = Response::from_json(&pact.get("actual").unwrap(), &PactSpecification::V3);
    println!("{:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_response(expected, actual);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn unexpected_header_found() {
    env_logger::init().unwrap_or(());
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Extra headers allowed",
        "expected" : {
          "headers": {}
        },
        "actual": {
          "headers": {
            "Accept": "alligators"
          }
        }
      }
    "#).unwrap();

    let expected = Response::from_json(&pact.get("expected").unwrap(), &PactSpecification::V3);
    println!("{:?}", expected);
    let actual = Response::from_json(&pact.get("actual").unwrap(), &PactSpecification::V3);
    println!("{:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_response(expected, actual);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn whitespace_after_comma_different() {
    env_logger::init().unwrap_or(());
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Whitespace between comma separated headers does not matter",
        "expected" : {
          "headers": {
            "Accept": "alligators,hippos"
          }
        },
        "actual": {
          "headers": {
            "Accept": "alligators, hippos"
          }
        }
      }
    "#).unwrap();

    let expected = Response::from_json(&pact.get("expected").unwrap(), &PactSpecification::V3);
    println!("{:?}", expected);
    let actual = Response::from_json(&pact.get("actual").unwrap(), &PactSpecification::V3);
    println!("{:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_response(expected, actual);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}
