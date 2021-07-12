use expectest::prelude::*;
use serde_json:: json;

use pact_matching::models::RequestResponseInteraction;
use pact_models::PactSpecification;
use pact_models::verify_json::{PactJsonVerifier, ResultLevel};

use super::verify_json;

#[test]
fn empty_json() {
  let json = json!({});
  let results = verify_json(&json, &PactSpecification::V1, "", false);

  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(be_empty());
}

#[test]
fn empty_json_strict() {
  let json = json!({});
  let results = verify_json(&json, &PactSpecification::V1, "", true);

  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to_not(be_empty());
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(have_count(2));

  let errors: Vec<String> = results.iter()
    .filter(|result| result.level == ResultLevel::ERROR)
    .map(|result| result.message.clone())
    .collect();
  expect!(errors).to(be_equal_to(
    vec!["Missing consumer".to_string(), "Missing provider".to_string()]));
}

#[test]
fn invalid_json() {
  let json = json!(["this is a pact file!"]);
  let results = verify_json(&json, &PactSpecification::V1, "", true);

  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to_not(be_empty());
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(have_count(1));

  let errors: Vec<String> = results.iter()
    .filter(|result| result.level == ResultLevel::ERROR)
    .map(|result| result.message.clone())
    .collect();
  expect!(errors).to(be_equal_to(
    vec!["Must be an Object, got Array".to_string()]));
}

#[test]
fn with_extra_properties() {
  let json = json!({
    "consumer": {
      "name": "test"
    },
    "provider": {
      "name": "test"
    },
    "interactions": [],
    "someOther": "property"
  });
  let results = verify_json(&json, &PactSpecification::V1, "", false);

  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(be_empty());

  let messages: Vec<String> = results.iter()
    .map(|result| result.message.clone())
    .collect();
  expect!(messages).to(be_equal_to(
    vec!["Interactions is empty".to_string(), "Unexpected attribute 'someOther'".to_string()]));
}

#[test]
fn with_extra_properties_strict() {
  let json = json!({
    "consumer": {
      "name": "test"
    },
    "provider": {
      "name": "test"
    },
    "interactions": [],
    "someOther": "property"
  });
  let results = verify_json(&json, &PactSpecification::V1, "", true);

  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to_not(be_empty());
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(have_count(1));

  let errors: Vec<String> = results.iter()
    .filter(|result| result.level == ResultLevel::ERROR)
    .map(|result| result.message.clone())
    .collect();
  expect!(errors).to(be_equal_to(
    vec!["Unexpected attribute 'someOther'".to_string()]));
}

#[test]
fn with_metadata() {
  let json = json!({
    "consumer": {
      "name": "test"
    },
    "provider": {
      "name": "test"
    },
    "interactions": [],
    "metadata": {
      "a": "b",
      "c": ["d"],
      "e": "f"
    }
  });
  let results = verify_json(&json, &PactSpecification::V1, "", false);

  expect!(results.iter()).to(have_count(1));
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(be_empty());

  let messages: Vec<String> = results.iter()
    .map(|result| result.message.clone())
    .collect();
  expect!(messages).to(be_equal_to(
    vec!["Interactions is empty".to_string()]));
}

#[test]
fn with_invalid_metadata() {
  let json = json!({
    "consumer": {
      "name": "test"
    },
    "provider": {
      "name": "test"
    },
    "interactions": [],
    "metadata": []
  });
  let results = verify_json(&json, &PactSpecification::V1, "", false);

  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(have_count(1));

  let messages: Vec<String> = results.iter()
    .filter(|result| result.level == ResultLevel::ERROR)
    .map(|result| result.message.clone())
    .collect();
  expect!(messages).to(be_equal_to(
    vec!["Metadata must be an Object, got Array".to_string()]));
}

#[test]
fn with_spec_version_in_metadata() {
  let json = json!({
    "consumer": {
      "name": "test"
    },
    "provider": {
      "name": "test"
    },
    "interactions": [],
    "metadata": {
      "pactSpecification" : {
        "version" : "2.0.0"
      }
    }
  });
  let results = verify_json(&json, &PactSpecification::V1, "", false);

  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(be_empty());
}

#[test]
fn with_old_spec_version_in_metadata() {
  let json = json!({
    "consumer": {
      "name": "test"
    },
    "provider": {
      "name": "test"
    },
    "interactions": [],
    "metadata": {
      "pact-specification" : {
        "version" : "2.0.0"
      }
    }
  });
  let results = verify_json(&json, &PactSpecification::V1, "", false);

  expect!(results.iter()).to(have_count(2));
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(be_empty());

  let messages: Vec<String> = results.iter()
    .map(|result| result.message.clone())
    .collect();
  expect!(messages).to(be_equal_to(
    vec!["Interactions is empty".to_string(), "'pact-specification' is deprecated, use 'pactSpecification' instead".to_string()]));
}

#[test]
fn with_missing_spec_version_in_metadata() {
  let json = json!({
    "consumer": {
      "name": "test"
    },
    "provider": {
      "name": "test"
    },
    "interactions": [],
    "metadata": {
      "pactSpecification" : {
      }
    }
  });
  let results = verify_json(&json, &PactSpecification::V1, "", false);

  expect!(results.iter()).to(have_count(2));
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(be_empty());

  let messages: Vec<String> = results.iter()
    .map(|result| result.message.clone())
    .collect();
  expect!(messages).to(be_equal_to(
    vec!["Interactions is empty".to_string(), "pactSpecification is missing the version attribute".to_string()]));
}

#[test]
fn with_null_spec_version_in_metadata() {
  let json = json!({
    "consumer": {
      "name": "test"
    },
    "provider": {
      "name": "test"
    },
    "interactions": [],
    "metadata": {
      "pactSpecification" : {
        "version": null
      }
    }
  });
  let results = verify_json(&json, &PactSpecification::V1, "", false);

  expect!(results.iter()).to(have_count(2));
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(be_empty());

  let messages: Vec<String> = results.iter()
    .map(|result| result.message.clone())
    .collect();
  expect!(messages).to(be_equal_to(
    vec!["Interactions is empty".to_string(), "pactSpecification version is NULL".to_string()]));
}

#[test]
fn with_incorrect_spec_version_in_metadata() {
  let json = json!({
    "consumer": {
      "name": "test"
    },
    "provider": {
      "name": "test"
    },
    "interactions": [],
    "metadata": {
      "pactSpecification" : {
        "version": json!([1, 2, 3])
      }
    }
  });
  let results = verify_json(&json, &PactSpecification::V1, "", false);

  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(have_count(1));

  let messages: Vec<String> = results.iter()
    .filter(|result| result.level == ResultLevel::ERROR)
    .map(|result| result.message.clone())
    .collect();
  expect!(messages).to(be_equal_to(
    vec!["Version must be a String, got Array".to_string()]));
}

#[test]
fn with_missing_consumer_name() {
  let json = json!({
    "consumer": {},
    "provider": {
      "name": "test"
    },
    "interactions": []
  });
  let results = verify_json(&json, &PactSpecification::V1, "", false);

  expect!(results.iter()).to(have_count(2));
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(be_empty());

  let messages: Vec<(&str, &str)> = results.iter()
    .map(|result| (result.message.as_str(), result.path.as_str()))
    .collect();
  expect!(messages).to(be_equal_to(
    vec![("Missing name", "/consumer/name"), ("Interactions is empty", "/interactions")]));

  let results = verify_json(&json, &PactSpecification::V1, "", true);

  expect!(results.iter()).to(have_count(2));
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(have_count(1));

  let messages: Vec<(&str, &str)> = results.iter()
    .filter(|result| result.level == ResultLevel::ERROR)
    .map(|result| (result.message.as_str(), result.path.as_str()))
    .collect();
  expect!(messages).to(be_equal_to(vec![("Missing name", "/consumer/name")]));
}

#[test]
fn with_additional_consumer_properties() {
  let json = json!({
    "consumer": {
      "name": "test",
      "other_name": "test"
    },
    "provider": {
      "name": "test"
    },
    "interactions": []
  });
  let results = verify_json(&json, &PactSpecification::V1, "", false);

  expect!(results.iter()).to(have_count(2));
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(be_empty());

  let messages: Vec<(&str, &str)> = results.iter()
    .map(|result| (result.message.as_str(), result.path.as_str()))
    .collect();
  expect!(messages).to(be_equal_to(
    vec![("Unknown attribute 'other_name'", "/consumer"), ("Interactions is empty", "/interactions")]));

  let results = verify_json(&json, &PactSpecification::V1, "", true);

  expect!(results.iter()).to(have_count(2));
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(have_count(1));

  let messages: Vec<&str> = results.iter()
    .filter(|result| result.level == ResultLevel::ERROR)
    .map(|result| result.message.as_str())
    .collect();
  expect!(messages).to(be_equal_to(vec!["Unknown attribute 'other_name'"]));
}

#[test]
fn with_missing_provider_name() {
  let json = json!({
    "consumer": {
      "name": "test"
    },
    "provider": {
    },
    "interactions": []
  });
  let results = verify_json(&json, &PactSpecification::V1, "", false);

  expect!(results.iter()).to(have_count(2));
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(be_empty());

  let messages: Vec<(&str, &str)> = results.iter()
    .map(|result| (result.message.as_str(), result.path.as_str()))
    .collect();
  expect!(messages).to(be_equal_to(
    vec![("Missing name", "/provider/name"), ("Interactions is empty", "/interactions")]));

  let results = verify_json(&json, &PactSpecification::V1, "", true);

  expect!(results.iter()).to(have_count(2));
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(have_count(1));

  let messages: Vec<(&str, &str)> = results.iter()
    .filter(|result| result.level == ResultLevel::ERROR)
    .map(|result| (result.message.as_str(), result.path.as_str()))
    .collect();
  expect!(messages).to(be_equal_to(vec![("Missing name", "/provider/name")]));
}

#[test]
fn with_additional_provider_properties() {
  let json = json!({
    "consumer": {
      "name": "test"
    },
    "provider": {
      "name": "test",
      "other_name": "test"
    },
    "interactions": []
  });
  let results = verify_json(&json, &PactSpecification::V1, "", false);

  expect!(results.iter()).to(have_count(2));
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(be_empty());

  let messages: Vec<(&str, &str)> = results.iter()
    .map(|result| (result.message.as_str(), result.path.as_str()))
    .collect();
  expect!(messages).to(be_equal_to(
    vec![("Unknown attribute 'other_name'", "/provider"), ("Interactions is empty", "/interactions")]));

  let results = verify_json(&json, &PactSpecification::V1, "", true);

  expect!(results.iter()).to(have_count(2));
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(have_count(1));

  let messages: Vec<&str> = results.iter()
    .filter(|result| result.level == ResultLevel::ERROR)
    .map(|result| result.message.as_str())
    .collect();
  expect!(messages).to(be_equal_to(vec!["Unknown attribute 'other_name'"]));
}

#[test]
fn verify_interaction_no_description() {
  let interation_json = json!({});
  let results = RequestResponseInteraction::verify_json("/interactions/0", &interation_json, false);
  expect!(results.iter()).to(have_count(1));
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(be_empty());
  let messages: Vec<(&str, &str)> = results.iter()
    .map(|result| (result.message.as_str(), result.path.as_str()))
    .collect();
  expect!(messages).to(be_equal_to(
    vec![("Missing description", "/interactions/0")]));

  let results = RequestResponseInteraction::verify_json("/interactions/0", &interation_json, true);
  expect!(results.iter()).to(have_count(1));
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(have_count(1));
  let messages: Vec<(&str, &str)> = results.iter()
    .map(|result| (result.message.as_str(), result.path.as_str()))
    .collect();
  expect!(messages).to(be_equal_to(
    vec![("Missing description", "/interactions/0")]));
}

#[test]
fn verify_interaction_invalid_description() {
  let interation_json = json!({
    "description": [1, 2, 3]
  });
  let results = RequestResponseInteraction::verify_json("/interactions/0", &interation_json, false);
  expect!(results.iter()).to(have_count(1));
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(have_count(1));
  let messages: Vec<(&str, &str)> = results.iter()
    .map(|result| (result.message.as_str(), result.path.as_str()))
    .collect();
  expect!(messages).to(be_equal_to(
    vec![("Must be a String, got Object", "/interactions/0/description")]));
}

#[test]
fn verify_interaction_extra_attributes() {
  let interation_json = json!({
    "description": "test",
    "other": "test"
  });
  let results = RequestResponseInteraction::verify_json("/interactions/0", &interation_json, false);
  expect!(results.iter()).to(have_count(1));
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(be_empty());
  let messages: Vec<(&str, &str)> = results.iter()
    .map(|result| (result.message.as_str(), result.path.as_str()))
    .collect();
  expect!(messages).to(be_equal_to(
    vec![("Unexpected attribute 'other'", "/interactions/0")]));

  let results = RequestResponseInteraction::verify_json("/interactions/0", &interation_json, true);
  expect!(results.iter()).to(have_count(1));
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(have_count(1));
  let messages: Vec<(&str, &str)> = results.iter()
    .map(|result| (result.message.as_str(), result.path.as_str()))
    .collect();
  expect!(messages).to(be_equal_to(
    vec![("Unexpected attribute 'other'", "/interactions/0")]));
}

#[test]
fn verify_interaction_invalid_interaction() {
  let interation_json = json!([1, 2, 3]);
  let results = RequestResponseInteraction::verify_json("/interactions/0", &interation_json, false);
  expect!(results.iter()).to(have_count(1));
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(have_count(1));
  let messages: Vec<(&str, &str)> = results.iter()
    .map(|result| (result.message.as_str(), result.path.as_str()))
    .collect();
  expect!(messages).to(be_equal_to(
    vec![("Must be an Object, got Array", "/interactions/0")]));
}
