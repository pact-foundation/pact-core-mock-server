//! Module for handling content types

use std::collections::BTreeMap;
use mime::Mime;
use log::*;
use maplit::btreemap;
use itertools::Itertools;
use lazy_static::*;
use std::str::FromStr;

#[cfg(test)]
use expectest::prelude::*;

/// Content type of a body
#[derive(Debug, Clone, PartialEq)]
pub struct ContentType {
  /// Main content type
  pub main_type: String,
  /// Sub content type
  pub sub_type: String,
  /// Content type attributes
  pub attributes: BTreeMap<String, String>,
  /// Suffix
  pub suffix: Option<String>
}

lazy_static! {
  /// XML Content Type
  pub static ref XML: ContentType = ContentType {
    main_type: "application".into(),
    sub_type: "xml".into(),
    .. ContentType::default()
  };

  /// HTML Content Type
  pub static ref HTML: ContentType = ContentType {
    main_type: "text".into(),
    sub_type: "html".into(),
    .. ContentType::default()
  };

  /// JSON Content Type
  pub static ref JSON: ContentType = ContentType {
    main_type: "application".into(),
    sub_type: "json".into(),
    .. ContentType::default()
  };

  /// TEXT Content Type
  pub static ref TEXT: ContentType = ContentType {
    main_type: "text".into(),
    sub_type: "plain".into(),
    .. ContentType::default()
  };
}

impl ContentType {
  /// Parses a string into a ContentType
  pub fn parse<'a, S: Into<&'a str>>(content_type: S) -> Result<ContentType, String> {
    let content_type = content_type.into();
    match Mime::from_str(content_type) {
      Ok(mime) => {
        Ok(ContentType {
          main_type: mime.type_().to_string(),
          sub_type: mime.subtype().to_string(),
          attributes: mime.params().map(|(key, value)| (key.to_string(), value.to_string())).collect(),
          suffix: mime.suffix().map(|name| name.to_string())
        })
      },
      Err(err) => {
        let message = format!("Failed to parse '{}' as a content type: {}",
                              content_type, err);
        warn!("{}", message);
        Err(message)
      }
    }
  }

  /// If it is a JSON type
  pub fn is_json(&self) -> bool {
    self.main_type == "application" && (self.sub_type == "json" ||
      self.suffix.as_ref().unwrap_or(&String::default()) == "json")
  }

  /// If it is a XML type
  pub fn is_xml(&self) -> bool {
    self.main_type == "application" && (self.sub_type == "xml" ||
      self.suffix.as_ref().unwrap_or(&String::default()) == "xml")
  }
}

impl ToString for ContentType {
  fn to_string(&self) -> String {
    if self.attributes.is_empty() {
      format!("{}/{}", self.main_type, self.sub_type)
    } else {
      format!("{}/{};{}", self.main_type, self.sub_type, self.attributes.iter()
        .map(|(key, value)| format!("{}={}", key, value)).join(";"))
    }
  }
}

impl Default for ContentType {
  fn default() -> Self {
    ContentType {
      main_type: "text".into(),
      sub_type: "plain".into(),
      attributes: btreemap!{},
      suffix: None
    }
  }
}

#[test]
fn parse_test() {
  let content_type = &ContentType::parse("application/json").unwrap();
  expect!(&content_type.main_type).to(be_equal_to(&"application".to_string()));
  expect!(&content_type.sub_type).to(be_equal_to(&"json".to_string()));
  expect!(content_type.attributes.iter()).to(be_empty());
  expect!(content_type.clone().suffix).to(be_none());

  let content_type = &ContentType::parse("application/json;charset=UTF-16").unwrap();
  expect!(&content_type.main_type).to(be_equal_to(&"application".to_string()));
  expect!(&content_type.sub_type).to(be_equal_to(&"json".to_string()));
  expect!(content_type.clone().attributes).to(be_equal_to(btreemap!{
    "charset".to_string() => "utf-16".to_string()
  }));
  expect!(content_type.clone().suffix).to(be_none());

  let content_type = &ContentType::parse("application/hal+json; charset=UTF-8").unwrap();
  expect!(&content_type.main_type).to(be_equal_to(&"application".to_string()));
  expect!(&content_type.sub_type).to(be_equal_to(&"hal".to_string()));
  expect!(content_type.clone().attributes).to(be_equal_to(btreemap!{
    "charset".to_string() => "utf-8".to_string()
  }));
  expect!(content_type.clone().suffix).to(be_some().value("json".to_string()));
}

#[test]
fn to_string_test() {
  let content_type = ContentType {
    main_type: "application".into(),
    sub_type: "hal+json".into(),
    .. ContentType::default()
  };
  expect!(content_type.to_string()).to(be_equal_to("application/hal+json".to_string()));

  let content_type = ContentType {
    main_type: "application".into(),
    sub_type: "hal+json".into(),
    attributes: btreemap! {
      "charset".to_string() => "UTF-32".to_string(),
      "b".to_string() => "c".to_string()
    },
    suffix: None
  };
  expect!(content_type.to_string()).to(be_equal_to("application/hal+json;b=c;charset=UTF-32".to_string()));
}

#[test]
fn is_json_test() {
  let content_type = ContentType {
    main_type: "application".into(),
    sub_type: "hal".into(),
    suffix: Some("json".to_string()),
    .. ContentType::default()
  };
  expect!(content_type.is_json()).to(be_true());

  let content_type = ContentType {
    main_type: "text".into(),
    sub_type: "javascript".into(),
    .. ContentType::default()
  };
  expect!(content_type.is_json()).to(be_false());

  let content_type = ContentType {
    main_type: "application".into(),
    sub_type: "json".into(),
    .. ContentType::default()
  };
  expect!(content_type.is_json()).to(be_true());
}

#[test]
fn is_xml_test() {
  let content_type = ContentType::parse("application/atom+xml").unwrap();
  expect!(content_type.is_xml()).to(be_true());

  let content_type = ContentType {
    main_type: "text".into(),
    sub_type: "javascript".into(),
    .. ContentType::default()
  };
  expect!(content_type.is_xml()).to(be_false());

  let content_type = ContentType {
    main_type: "application".into(),
    sub_type: "xml".into(),
    .. ContentType::default()
  };
  expect!(content_type.is_xml()).to(be_true());
}
