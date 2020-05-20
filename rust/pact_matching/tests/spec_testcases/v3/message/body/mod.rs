#[allow(unused_imports)]
use test_env_log::test;
#[allow(unused_imports)]
use pact_matching::models::PactSpecification;
#[allow(unused_imports)]
use pact_matching::models::message::Message;
#[allow(unused_imports)]
use pact_matching::match_message;
#[allow(unused_imports)]
use expectest::prelude::*;
#[allow(unused_imports)]
use serde_json;

#[test]
fn unexpected_index_with_not_null_value() {
    println!("FILE: tests/spec_testcases/v3/message/body/unexpected index with not null value.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Unexpected favourite colour",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "favouriteColours": ["red","blue","taupe"]
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn unexpected_key_with_null_value() {
    println!("FILE: tests/spec_testcases/v3/message/body/unexpected key with null value.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Unexpected phone number with null value",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator": {
              "name": "Mary"
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator": {
              "name": "Mary",
              "phoneNumber": null
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn different_value_found_at_key() {
    println!("FILE: tests/spec_testcases/v3/message/body/different value found at key.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Incorrect value at alligator name",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "name": "Mary"
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "name": "Fred"
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn not_null_found_at_key_when_null_expected() {
    println!("FILE: tests/spec_testcases/v3/message/body/not null found at key when null expected.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Name should be null",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "name": null
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "name": "Fred"
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn array_with_regular_expression_in_element() {
    println!("FILE: tests/spec_testcases/v3/message/body/array with regular expression in element.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Types and regular expressions match",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "animals": [
              {
                "phoneNumber": "0415674567"
              }
            ]
          },
          "matchingRules": {
            "body": {
              "$.animals": {
                "matchers": [
                  {
                    "min": 1,
                    "match": "type"
                  }
                ]
              },
              "$.animals[*].*": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              },
              "$.animals[*].phoneNumber": {
                "matchers": [
                  {
                    "match": "regex",
                    "regex": "\\d+"
                  }
                ]
              }
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "animals": [
              {
                "phoneNumber": "333"
              },{
                "phoneNumber": "983479823479283478923"
              }
            ]
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn number_found_in_array_when_string_expected() {
    println!("FILE: tests/spec_testcases/v3/message/body/number found in array when string expected.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite colours expected to be strings found a number",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "favouriteNumbers": ["1","2","3"]
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "favouriteNumbers": ["1",2,"3"]
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn array_in_different_order() {
    println!("FILE: tests/spec_testcases/v3/message/body/array in different order.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite colours in wrong order",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "favouriteColours": ["blue", "red"]
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn matches_with_regex_with_bracket_notation() {
    println!("FILE: tests/spec_testcases/v3/message/body/matches with regex with bracket notation.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Messages match with regex with bracket notation",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "2" : {
              "str" : "jildrdmxddnVzcQZfjCA"
            }
          },
          "matchingRules": {
            "body": {
              "$['2'].str": {
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
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "2" : {
              "str" : "saldfhksajdhffdskkjh"
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn array_with_regular_expression_that_does_not_match_in_element() {
    println!("FILE: tests/spec_testcases/v3/message/body/array with regular expression that does not match in element.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Types and regular expressions don't match",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "animals": [
              {
                "phoneNumber": "0415674567"
              }
            ]
          },
          "matchingRules": {
            "body": {
              "$.animals": {
                "matchers": [
                  {
                    "min": 1,
                    "match": "type"
                  }
                ]
              },
              "$.animals[*].*": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              },
              "$.animals[*].phoneNumber": {
                "matchers": [
                  {
                    "match": "regex",
                    "regex": "\\d+"
                  }
                ]
              }
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "animals": [
              {
                "phoneNumber": "333"
              },{
                "phoneNumber": "abc"
              }
            ]
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn array_size_less_than_required() {
    println!("FILE: tests/spec_testcases/v3/message/body/array size less than required.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Empty array",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "animals": [
              {
                "name" : "Fred"
              }
            ]
          },
          "matchingRules": {
            "body": {
              "$.animals": {
                "matchers": [
                  {
                    "min": 1,
                    "match": "type"
                  }
                ]
              }
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "animals": []
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn array_with_nested_array_that_does_not_match() {
    println!("FILE: tests/spec_testcases/v3/message/body/array with nested array that does not match.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Nested arrays do not match, age is wrong type",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "animals": [
              {
                "name" : "Fred",
                "children": [
                  {
                    "age": 9
                  }
                ]
              }
            ]
          },
          "matchingRules": {
            "body": {
              "$.animals": {
                "matchers": [
                  {
                    "min": 1,
                    "match": "type"
                  }
                ]
              },
              "$.animals[*].*": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              },
              "$.animals[*].children": {
                "matchers": [
                  {
                    "min": 1
                  }
                ]
              },
              "$.animals[*].children[*].*": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              }
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "animals": [
              {
                "name" : "Mary",
                "children": [{"age": "9"}]
              }
            ]
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn string_found_in_array_when_number_expected() {
    println!("FILE: tests/spec_testcases/v3/message/body/string found in array when number expected.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite Numbers expected to be numbers, but 2 is a string",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "favouriteNumbers": [1,2,3]
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "favouriteNumbers": [1,"2",3]
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn no_body() {
    println!("FILE: tests/spec_testcases/v3/message/body/no body.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Missing body",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator": {
              "age": 3
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn array_at_top_level() {
    println!("FILE: tests/spec_testcases/v3/message/body/array at top level.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "top level array matches",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": [
            {
              "dob": "06/10/2015",
              "name": "Rogger the Dogger",
              "id": 1014753708,
              "timestamp": "2015-06-10T20:41:37"
            },
            {
              "dob": "06/10/2015",
              "name": "Cat in the Hat",
              "id": 8858030303,
              "timestamp": "2015-06-10T20:41:37"
            }
          ]
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": [
            {
              "dob": "06/10/2015",
              "name": "Rogger the Dogger",
              "id": 1014753708,
              "timestamp": "2015-06-10T20:41:37"
            },
            {
              "dob": "06/10/2015",
              "name": "Cat in the Hat",
              "id": 8858030303,
              "timestamp": "2015-06-10T20:41:37"
            }
          ]
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn matches_with_type() {
    println!("FILE: tests/spec_testcases/v3/message/body/matches with type.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Messages match with same type",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "name": "Mary",
              "feet": 4,
              "favouriteColours": ["red","blue"]
            }
          },
          "matchingRules": {
            "body": {
              "$.alligator.name": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              },
              "$.alligator.feet": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              }
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "feet": 5,
              "name": "Harry the very hungry alligator with an extra foot",
              "favouriteColours": ["red","blue"]
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn no_body_no_content_type() {
    println!("FILE: tests/spec_testcases/v3/message/body/no body no content type.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "No body, no content-type",
        "expected": {},
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator": {
              "age": 3
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn not_null_found_in_array_when_null_expected() {
    println!("FILE: tests/spec_testcases/v3/message/body/not null found in array when null expected.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite colours expected to contain null, but not null found",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "favouriteNumbers": ["1",null,"3"]
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "favouriteNumbers": ["1","2","3"]
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn array_with_at_least_one_element_not_matching_example_type() {
    println!("FILE: tests/spec_testcases/v3/message/body/array with at least one element not matching example type.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Wrong type for name key",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "animals": [
              {
                "name" : "Fred"
              }
            ]
          },
          "matchingRules": {
            "body": {
              "$.animals": {
                "matchers": [
                  {
                    "min": 1,
                    "match": "type"
                  }
                ]
              },
              "$.animals[*].*": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              }
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "animals": [
              {
                "name" : "Mary"
              },{
                "name" : 1
              }
            ]
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn different_value_found_at_index() {
    println!("FILE: tests/spec_testcases/v3/message/body/different value found at index.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Incorrect favourite colour",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "favouriteColours": ["red","taupe"]
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn array_with_nested_array_that_matches() {
    println!("FILE: tests/spec_testcases/v3/message/body/array with nested array that matches.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Nested arrays match",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "animals": [
              {
                "name" : "Fred",
                "children": [
                  {
                    "age": 9
                  }
                ]
              }
            ]
          },
          "matchingRules": {
            "body": {
              "$.animals": {
                "matchers": [
                  {
                    "min": 1,
                    "match": "type"
                  }
                ]
              },
              "$.animals[*].*": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              },
              "$.animals[*].children": {
                "matchers": [
                  {
                    "min": 1,
                    "match": "type"
                  }
                ]
              },
              "$.animals[*].children[*].*": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              }
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "animals": [
              {
                "name" : "Mary",
                "children": [
                  {"age": 3},
                  {"age": 5},
                  {"age": 5456}
                ]
              },{
                "name" : "Jo",
                "children": [
                  {"age": 0}
                ]
              }
            ]
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn matches_with_regex() {
    println!("FILE: tests/spec_testcases/v3/message/body/matches with regex.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Messages match with regex",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "name": "Mary",
              "feet": 4,
              "favouriteColours": ["red","blue"]
            }
          },
          "matchingRules": {
            "body": {
              "$.alligator.name": {
                "matchers": [
                  {
                    "match": "regex",
                    "regex": "\\w+"
                  }
                ]
              },
              "$.alligator.favouriteColours[0]": {
                "matchers": [
                  {
                    "match": "regex",
                    "regex": "red|blue"
                  }
                ]
              },
              "$.alligator.favouriteColours[1]": {
                "matchers": [
                  {
                    "match": "regex",
                    "regex": "red|blue"
                  }
                ]
              }
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "feet": 4,
              "name": "Harry",
              "favouriteColours": ["blue", "red"]
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn number_found_at_key_when_string_expected() {
    println!("FILE: tests/spec_testcases/v3/message/body/number found at key when string expected.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Number of feet expected to be string but was number",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "feet": "4"
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "feet": 4
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn missing_index() {
    println!("FILE: tests/spec_testcases/v3/message/body/missing index.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Missing favorite colour",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator": {
              "favouriteColours": ["red"]
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn null_found_in_array_when_not_null_expected() {
    println!("FILE: tests/spec_testcases/v3/message/body/null found in array when not null expected.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Favourite colours expected to be strings found a null",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "favouriteNumbers": ["1","2","3"]
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "favouriteNumbers": ["1",null,"3"]
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn unexpected_index_with_null_value() {
    println!("FILE: tests/spec_testcases/v3/message/body/unexpected index with null value.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Unexpected favourite colour with null value",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "favouriteColours": ["red","blue", null]
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn unexpected_key_with_not_null_value() {
    println!("FILE: tests/spec_testcases/v3/message/body/unexpected key with not null value.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Unexpected phone number",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "name": "Mary"
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "name": "Mary",
              "phoneNumber": "12345678"
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn string_found_at_key_when_number_expected() {
    println!("FILE: tests/spec_testcases/v3/message/body/string found at key when number expected.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Number of feet expected to be number but was string",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "feet": 4
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "feet": "4"
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn missing_key() {
    println!("FILE: tests/spec_testcases/v3/message/body/missing key.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Missing key alligator name",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "name": "Mary",
              "age": 3
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator": {
              "age": 3
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn array_with_at_least_one_element_matching_by_example() {
    println!("FILE: tests/spec_testcases/v3/message/body/array with at least one element matching by example.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Types and regular expressions match",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "animals": [
              {
                "name" : "Fred"
              }
            ]
          },
          "matchingRules": {
            "body": {
              "$.animals": {
                "matchers": [
                  {
                    "min": 1,
                    "match": "type"
                  }
                ]
              },
              "$.animals[*].*": {
                "matchers": [
                  {
                    "match": "type"
                  }
                ]
              }
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "animals": [
              {
                "name" : "Mary"
              },{
                "name" : "Susan"
              }
            ]
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn matches() {
    println!("FILE: tests/spec_testcases/v3/message/body/matches.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Messages match",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "name": "Mary",
              "feet": 4,
              "favouriteColours": ["red","blue"]
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "feet": 4,
              "name": "Mary",
              "favouriteColours": ["red","blue"]
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn null_found_at_key_where_not_null_expected() {
    println!("FILE: tests/spec_testcases/v3/message/body/null found at key where not null expected.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Name should not be null",
        "expected": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "name": "Mary"
            }
          }
        },
        "actual": {
          "metaData": {
            "contentType": "application/json"
          },
          "contents": {
            "alligator":{
              "name": null
            }
          }
        }
      }
    "#).unwrap();

    let expected = Message::from_json(0, &pact.get("expected").unwrap(), &PactSpecification::V3).unwrap();
    println!("EXPECTED: {:?}", expected);
    let actual = Message::from_json(0, &pact.get("actual").unwrap(), &PactSpecification::V3).unwrap();
    println!("ACTUAL: {:?}", actual);
    let pact_match = pact.get("match").unwrap();
    let result = match_message(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}
