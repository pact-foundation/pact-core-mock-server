use super::Mismatch;
use super::DiffConfig;
use sxd_document::dom::*;
use std::collections::btree_map::BTreeMap;
use itertools::{Itertools, EitherOrBoth};
use crate::models::matchingrules::*;
use crate::matchers::*;
use onig::Regex;
use crate::models::xml_utils::parse_bytes;
use crate::models::{HttpPart, OptionalBody};
use sxd_document::QName;
use maplit::*;

pub fn match_xml(expected: &dyn HttpPart, actual: &dyn HttpPart, config: DiffConfig,
  mismatches: &mut Vec<super::Mismatch>, matchers: &MatchingRules) {
  match (expected.body(), actual.body()) {
    (OptionalBody::Missing, _) => (),
    (OptionalBody::Empty, _) => (),
    (OptionalBody::Null, _) => (),
    (OptionalBody::Present(expected_body, _), OptionalBody::Present(actual_body, _)) => {
      let expected_result = parse_bytes(expected_body);
      let actual_result = parse_bytes(actual_body);

      if expected_result.is_err() || actual_result.is_err() {
        match expected_result {
          Err(e) => {
            mismatches.push(Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.body().value().clone().into()),
              actual: Some(actual.body().value().clone().into()),
              mismatch: format!("Failed to parse the expected body: '{:?}'", e)});
          },
          _ => ()
        }
        match actual_result {
          Err(e) => {
            mismatches.push(Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.body().value().clone().into()),
              actual: Some(actual.body().value().clone().into()),
              mismatch: format!("Failed to parse the actual body: '{:?}'", e)});
          },
          _ => ()
        }
      } else {
        let expected_package = expected_result.unwrap();
        let expected_root = expected_package.as_document().root();
        let expected_root_node = expected_root.children().iter().cloned().find(|n| n.element().is_some());
        let actual_package = actual_result.unwrap();
        let actual_root = actual_package.as_document().root();
        let actual_root_node = actual_root.children().iter().cloned().find(|n| n.element().is_some());
        let element = expected_root_node.unwrap().element().unwrap();
        let path = vec!["$".to_string(), name(element.name())];
        compare_element(&path, &element,
                        &actual_root_node.unwrap().element().unwrap(), config, mismatches, matchers);
      }
    },
    _ => {
      mismatches.push(Mismatch::BodyMismatch { path: "$".into(), expected: Some(expected.body().value().clone().into()),
        actual: None, mismatch: format!("Expected an XML body {} but was missing", expected.body())});
    }
  }
}

fn name(name: QName) -> String {
  if let Some(namespace) = name.namespace_uri() {
    format!("{}:{}", namespace, name.local_part())
  } else {
    name.local_part().to_string()
  }
}

impl<'a> Matches<Element<'a>> for Element<'a> {
    fn matches(&self, actual: &Element, matcher: &MatchingRule) -> Result<(), String> {
        let result = match *matcher {
          MatchingRule::Regex(ref regex) => {
            match Regex::new(regex) {
              Ok(re) => {
                if re.is_match(actual.name().local_part()) {
                  Ok(())
                } else {
                  Err(format!("Expected '{}' to match '{}'", name(actual.name()), regex))
                }
              },
              Err(err) => Err(format!("'{}' is not a valid regular expression - {}", regex, err))
            }
          },
          MatchingRule::Type => if self.name() == actual.name() {
             Ok(())
          } else {
             Err(format!("Expected '{}' to be the same type as '{}'", name(self.name()),
                         name(actual.name())))
          },
          MatchingRule::MinType(min) => if actual.children().len() < min {
             Err(format!("Expected '{}' to have at least {} children", name(actual.name()), min))
          } else {
             Ok(())
          },
          MatchingRule::MaxType(max) => if actual.children().len() > max {
             Err(format!("Expected '{}' to have at most {} children", name(actual.name()), max))
          } else {
             Ok(())
          },
          MatchingRule::MinMaxType(min, max) => if actual.children().len() < min {
            Err(format!("Expected '{}' to have at least {} children", name(actual.name()), min))
          } else if actual.children().len() > max {
            Err(format!("Expected '{}' to have at most {} children", name(actual.name()), max))
          } else {
            Ok(())
          },
          MatchingRule::Equality => {
             if self.name() == actual.name() {
                 Ok(())
             } else {
                  Err(format!("Expected '{}' to be equal to '{}'", name(self.name()), name(actual.name())))
             }
          },
          _ => Err(format!("Unable to match {:?} using {:?}", self, matcher))
        };
        log::debug!("Comparing '{:?}' to '{:?}' using {:?} -> {:?}", self, actual, matcher, result);
        result
    }
}

fn path_to_string(path: &Vec<String>) -> String {
    path.iter().enumerate().map(|(i, p)| {
        if i > 0 && !p.starts_with("[") {
            s!(".") + p
        } else {
            p.clone()
        }
    }).collect()
}

fn compare_element(path: &Vec<String>, expected: &Element, actual: &Element, config: DiffConfig,
  mismatches: &mut Vec<super::Mismatch>, matchers: &MatchingRules) {
  let matcher_result = if matchers.matcher_is_defined("body", &path) {
    log::debug!("calling match_values {:?} on {:?}", path, actual);
    match_values("body", &path, matchers.clone(), expected, actual)
  } else {
    expected.matches(actual, &MatchingRule::Equality).map_err(|err| vec![err])
  };
  log::debug!("Comparing '{:?}' to '{:?}' at path '{}' -> {:?}", expected, actual,
    path_to_string(&path), matcher_result);
  match matcher_result {
    Err(messages) => {
      for message in messages {
        mismatches.push(Mismatch::BodyMismatch {
          path: path_to_string(&path),
          expected: Some(name(expected.name()).into()),
          actual: Some(name(actual.name()).into()),
          mismatch: message.clone()
        })
      }
    },
    Ok(_) => {
      compare_attributes(&path, expected, actual, config.clone(), mismatches, matchers);
      compare_children(&path, expected, actual, config.clone(), mismatches, matchers);
      compare_text(&path, expected, actual, mismatches, matchers);
    }
  }
}

fn compare_attributes(path: &Vec<String>, expected: &Element, actual: &Element, config: DiffConfig,
    mismatches: &mut Vec<super::Mismatch>, matchers: &MatchingRules) {
    let expected_attributes: BTreeMap<String, String> = expected.attributes()
        .iter().map(|attr| (name(attr.name()), s!(attr.value()))).collect();
    let actual_attributes: BTreeMap<String, String> = actual.attributes()
        .iter().map(|attr| (name(attr.name()), s!(attr.value()))).collect();
    if expected_attributes.is_empty() && !actual_attributes.is_empty() && config == DiffConfig::NoUnexpectedKeys {
      mismatches.push(Mismatch::BodyMismatch { path: path_to_string(path),
          expected: Some(format!("{:?}", expected_attributes).into()),
          actual: Some(format!("{:?}", actual_attributes).into()),
          mismatch: format!("Did not expect any attributes but received {:?}", actual_attributes)});
    } else {
        match config {
            DiffConfig::AllowUnexpectedKeys if expected_attributes.len() > actual_attributes.len() => {
                mismatches.push(Mismatch::BodyMismatch { path: path_to_string(path),
                    expected: Some(format!("{:?}", expected_attributes).into()),
                    actual: Some(format!("{:?}", actual_attributes).into()),
                    mismatch: format!("Expected at least {} attribute(s) but received {} attribute(s)",
                    expected_attributes.len(), actual_attributes.len())});
            },
            DiffConfig::NoUnexpectedKeys if expected_attributes.len() != actual_attributes.len() => {
                mismatches.push(Mismatch::BodyMismatch { path: path_to_string(path),
                    expected: Some(format!("{:?}", expected_attributes).into()),
                    actual: Some(format!("{:?}", actual_attributes).into()),
                    mismatch: format!("Expected {} attribute(s) but received {} attribute(s)",
                    expected_attributes.len(), actual_attributes.len())});
            },
            _ => ()
        }

        for (key, value) in expected_attributes.iter() {
          let mut p = path.to_vec();
          p.push(s!("@") + key);
          if actual_attributes.contains_key(key) {
            compare_value(&p, value, &actual_attributes[key], mismatches, matchers);
          } else {
            mismatches.push(Mismatch::BodyMismatch { path: path_to_string(&p),
              expected: Some(key.as_bytes().to_vec()),
              actual: None,
              mismatch: format!("Expected attribute '{}'='{}' but was missing", key, value)});
          }
        }
    }
}

fn children<'a>(element: &Element<'a>) -> Vec<Element<'a>> {
  element.children().iter().cloned()
    .map(|child| child.element())
    .filter(|child| child.is_some())
    .map(|child| child.unwrap())
    .collect()
}

fn desc_children(children: &Vec<Element>) -> String {
  children.iter().map(|child| name(child.name())).join(", ")
}

fn compare_children(path: &Vec<String>, expected: &Element, actual: &Element, config: DiffConfig,
  mismatches: &mut Vec<super::Mismatch>, matchers: &MatchingRules) {
  let expected_children = children(expected);
  let actual_children = children(actual);

  if expected_children.is_empty() && !actual_children.is_empty() && config == DiffConfig::NoUnexpectedKeys {
    mismatches.push(Mismatch::BodyMismatch {
      path: path_to_string(path),
      expected: Some(desc_children(&expected_children).into()),
      actual: Some(desc_children(&actual_children).into()),
      mismatch: format!("Expected no children but received [{}]", desc_children(&actual_children))
    });
  } else {
    let mut expected_children_by_name: BTreeMap<String, Vec<Element>> = btreemap!{};
    for child in &expected_children {
      let key = name(child.name());
      if expected_children_by_name.contains_key(&key) {
        expected_children_by_name.get_mut(&key).unwrap().push(child.clone());
      } else {
        expected_children_by_name.insert(key, vec![ child.clone() ]);
      }
    }
    let mut actual_children_by_name: BTreeMap<String, Vec<Element>> = btreemap!{};
    for child in &actual_children {
      let key = name(child.name());
      if actual_children_by_name.contains_key(&key) {
        actual_children_by_name.get_mut(&key).unwrap().push(child.clone());
      } else {
        actual_children_by_name.insert(key, vec![ child.clone() ]);
      }
    }
    for (key, group) in actual_children_by_name {
      let mut p = path.to_vec();
      p.push(key.to_string());
      if expected_children_by_name.contains_key(&key) {
        let expected_children = expected_children_by_name.remove(&key).unwrap();
        let expected = expected_children.iter().next().unwrap();
        if matchers.type_matcher_defined("body", &p) {
          log::debug!("Matcher defined for path {}", path_to_string(&p));
          for child in group {
            compare_element(&p, &expected, &child, config.clone(), mismatches, matchers);
          }
        } else {
          for pair in expected_children.iter().zip_longest(group) {
            match pair {
              EitherOrBoth::Right(actual) => if config == DiffConfig::NoUnexpectedKeys {
                mismatches.push(Mismatch::BodyMismatch { path: path_to_string(&p),
                  expected: Some(desc_children(&expected_children).into()),
                  actual: Some(desc_children(&actual_children).into()),
                  mismatch: format!("Unexpected child <{}/>", name(actual.name()))});
              },
              EitherOrBoth::Left(expected) => {
                mismatches.push(Mismatch::BodyMismatch { path: path_to_string(&p),
                  expected: Some(desc_children(&expected_children.clone()).into()),
                  actual: Some(desc_children(&actual_children.clone()).into()),
                  mismatch: format!("Expected child <{}/> but was missing", name(expected.name()))});
              },
              EitherOrBoth::Both(expected, actual) => {
                compare_element(&p, &expected, &actual, config.clone(), mismatches, matchers);
              }
            }
          }
        }
      } else if config == DiffConfig::NoUnexpectedKeys || matchers.type_matcher_defined("body", &p) {
        mismatches.push(Mismatch::BodyMismatch { path: path_to_string(path),
          expected: Some(desc_children(&expected_children.clone()).into()),
          actual: Some(desc_children(&actual_children.clone()).into()),
          mismatch: format!("Unexpected child <{}/>", key)});
      }
    }

    if !expected_children_by_name.is_empty() {
      for key in expected_children_by_name.keys() {
        mismatches.push(Mismatch::BodyMismatch { path: path_to_string(path),
          expected: Some(desc_children(&expected_children.clone()).into()),
          actual: Some(desc_children(&actual_children.clone()).into()),
          mismatch: format!("Expected child <{}/> but was missing", key)});
      }
    }
  }
}

fn compare_text(path: &Vec<String>, expected: &Element, actual: &Element,
    mismatches: &mut Vec<super::Mismatch>, matchers: &MatchingRules) {
    let expected_text = s!(expected.children().iter().cloned()
        .filter(|child| child.text().is_some())
        .map(|child| child.text().unwrap().text())
        .collect::<String>().trim());
    let actual_text = s!(actual.children().iter().cloned()
        .filter(|child| child.text().is_some())
        .map(|child| child.text().unwrap().text())
        .collect::<String>().trim());
    let mut p = path.to_vec();
    p.push(s!("#text"));
    let matcher_result = if matchers.matcher_is_defined("body", &p) {
      match_values("body", &p, matchers.clone(), &expected_text, &actual_text)
    } else {
      expected_text.matches(&actual_text, &MatchingRule::Equality).map_err(|err| vec![err])
    };
    log::debug!("Comparing text '{}' to '{}' at path '{}' -> {:?}", expected_text, actual_text,
        path_to_string(path), matcher_result);
    match matcher_result {
        Err(messages) => {
          for message in messages {
            mismatches.push(Mismatch::BodyMismatch {
              path: path_to_string(path) + ".#text",
              expected: Some(expected_text.clone().into()),
              actual: Some(actual_text.clone().into()),
              mismatch: message.clone()
            })
          }
        },
        Ok(_) => ()
    }
}

fn compare_value(path: &Vec<String>, expected: &String, actual: &String,
    mismatches: &mut Vec<super::Mismatch>, matchers: &MatchingRules) {
    let matcher_result = if matchers.matcher_is_defined("body", &path) {
      match_values("body", path, matchers.clone(), expected, actual)
    } else {
      expected.matches(actual, &MatchingRule::Equality).map_err(|err| vec![err])
    };
    log::debug!("Comparing '{}' to '{}' at path '{}' -> {:?}", expected, actual, path_to_string(path), matcher_result);
    match matcher_result {
        Err(messages) => {
          for message in messages {
            mismatches.push(Mismatch::BodyMismatch {
              path: path_to_string(path),
              expected: Some(expected.clone().into()),
              actual: Some(actual.clone().into()),
              mismatch: message.clone()
            })
          }
        },
        Ok(_) => ()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expectest::prelude::*;
    use expectest::expect;
    use crate::Mismatch;
    use crate::DiffConfig;
    use test_env_log::test;
    use crate::models::{Request, OptionalBody};
    use ntest::test_case;

    macro_rules! request {
      ($e:expr) => (Request { body: OptionalBody::Present($e.as_bytes().to_vec(), None), .. Request::default() })
    }

  #[test]
  fn match_xml_comparing_missing_bodies() {
      let mut mismatches = vec![];
      let expected = Request { body: OptionalBody::Missing, .. Request::default() };
      let actual = Request { body: OptionalBody::Missing, .. Request::default() };
      match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
      expect!(mismatches.iter()).to(be_empty());
  }

  #[test]
  fn match_xml_comparing_empty_bodies() {
    let mut mismatches = vec![];
    let expected = Request { body: OptionalBody::Empty, .. Request::default() };
    let actual = Request { body: OptionalBody::Empty, .. Request::default() };
    match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
    expect!(mismatches.iter()).to(be_empty());
  }

  #[test]
  fn match_xml_when_comparing_a_missing_body_to_anything() {
    let mut mismatches = vec![];
    let expected = Request { body: OptionalBody::Empty, .. Request::default() };
    let actual = request!("<blah/>");
    match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
    expect!(mismatches.iter()).to(be_empty());
  }

  #[test]
  fn match_xml_when_comparing_anything_to_an_empty_body_returns_a_mismatch() {
    let mut mismatches = vec![];
    let expected = request!("<blah/>");
    let actual = Request { body: OptionalBody::Empty, .. Request::default() };
    match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
    expect!(mismatches.iter()).to_not(be_empty());
    let mismatch = mismatches[0].clone().description();
    expect!(&mismatch).to(be_equal_to("$ -> Expected an XML body Present(7 bytes) but was missing"));
  }

    #[test]
    fn match_xml_handles_empty_strings() {
        let mut mismatches = vec![];
        let expected = request!("");
        let actual = request!("");
        match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(2));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$"), expected: Some(vec![]),
            actual: Some(vec![]), mismatch: s!("")}));
    }

    #[test]
    fn match_xml_handles_invalid_expected_xml() {
        let mut mismatches = vec![];
        let expected = request!(r#"<xml-is-bad"#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?> <blah/>"#);
        match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.body.value()),
            actual: Some(actual.body.value()), mismatch: s!("")}));
    }

    #[test]
    fn match_xml_handles_invalid_actual_xml() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?> <blah/>"#);
        let actual = request!(r#"{json: "is bad"}"#);
        match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.body.value()),
            actual: Some(actual.body.value()), mismatch: s!("")}));
    }

    fn mismatch_message(mismatch: &Mismatch) -> String {
        match mismatch {
            &Mismatch::BodyMismatch{ path: _, expected: _, actual: _, mismatch: ref m } => m.clone(),
            _ => s!("")
        }
    }

    #[test]
    fn match_xml_with_equal_bodies() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?> <blah/>"#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?> <blah/>"#);
        match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(be_empty());
    }

  #[test]
  fn match_xml_when_allow_unexpected_keys_is_true_and_comparing_an_empty_list_to_a_non_empty_one() {
    let mut mismatches = vec![];
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?> <foo></foo>"#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?> <foo><item/></foo>"#);
    match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
    expect!(mismatches.iter()).to(be_empty());
  }

  #[test]
  fn match_xml_when_allow_unexpected_keys_is_true_and_comparing_a_list_to_a_super_set() {
    let mut mismatches = vec![];
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?> <foo><item1/></foo>"#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?> <foo><item1/><item2/></foo>"#);
    match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
    expect!(mismatches.iter()).to(be_empty());
  }

    #[test]
    fn match_xml_when_bodies_differ_only_in_whitespace() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo>
            <bar></bar>
        </foo>
        "#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo><bar></bar></foo>
        "#);
        match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(be_empty());
    }

    #[test]
    fn match_xml_when_actual_has_different_root() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo/>
        "#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <bar></bar>
        "#);
        match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.foo"), expected: Some("foo".into()),
            actual: Some("bar".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected 'foo' to be equal to 'bar'")));
    }

    #[test]
    fn match_xml_with_equal_attributes() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <blah a="b" c="d"/>
        "#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <blah a="b" c="d"/>
        "#);
        match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(be_empty());
    }

    #[test]
    fn match_xml_with_nonequal_attributes() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <blah a="c" c="b"/>
        "#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <blah a="b"/>
        "#);
        match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(3));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.blah"),
            expected: Some("{\"a\": \"c\", \"c\": \"b\"}".into()),
            actual: Some("{\"a\": \"b\"}".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected at least 2 attribute(s) but received 1 attribute(s)")));
        let mismatch = mismatches[1].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.blah.@a"), expected: Some("c".into()),
            actual: Some("b".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected 'c' to be equal to 'b'")));
        let mismatch = mismatches[2].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.blah.@c"), expected: Some("c".into()),
            actual: None, mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected attribute \'c\'=\'b\' but was missing")));
    }

    #[test]
    fn match_xml_with_when_not_expecting_attributes() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <blah/>
        "#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <blah a="b" c="d"/>
        "#);
        match_xml(&expected, &actual, DiffConfig::NoUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.blah"), expected: Some("{}".into()),
            actual: Some("{\"a\": \"b\", \"c\": \"d\"}".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Did not expect any attributes but received {\"a\": \"b\", \"c\": \"d\"}")));
    }

    #[test]
    fn match_xml_with_comparing_a_tags_attributes_to_one_with_more_entries() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <blah a="b"/>
        "#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <blah a="b" c="d"/>
        "#);
        match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(be_empty());
    }

    #[test]
    fn match_xml_with_comparing_a_tags_attributes_to_one_with_less_entries() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo something="100"/>
        "#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo something="100" somethingElse="101"/>
        "#);
        match_xml(&expected, &actual, DiffConfig::NoUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.foo"), expected: Some("{\"something\": \"100\"}".into()),
            actual: Some("{\"something\": \"100\", \"somethingElse\": \"101\"}".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected 1 attribute(s) but received 2 attribute(s)")));
    }

    #[test]
    fn match_xml_when_a_tag_has_the_same_number_of_attributes_but_different_keys() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo something="100" somethingElse="100"/>
        "#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo something="100" somethingDifferent="100"/>
        "#);
        match_xml(&expected, &actual, DiffConfig::NoUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.foo.@somethingElse"), expected: Some("somethingElse".into()),
            actual: None, mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected attribute \'somethingElse\'=\'100\' but was missing")));
    }

    #[test]
    fn match_xml_when_a_tag_has_the_same_number_of_attributes_but_different_values() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo something="100" somethingElse="100"/>
        "#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo something="100" somethingElse="101"/>
        "#);
        match_xml(&expected.clone(), &actual.clone(), DiffConfig::NoUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.foo.@somethingElse"), expected: Some("100".into()),
            actual: Some("101".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected \'100\' to be equal to \'101\'")));

        mismatches.clear();
        match_xml(&expected, &actual, DiffConfig::NoUnexpectedKeys, &mut mismatches, &matchingrules!{
            "body" => {
                "$.foo.*" => [ MatchingRule::Type ]
            }
        });
        expect!(mismatches.iter()).to(be_empty());
    }

    #[test]
    fn match_xml_when_actual_is_non_empty_and_we_do_not_allow_extra_keys() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo/>
        "#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo><bar></bar></foo>
        "#);
        match_xml(&expected, &actual, DiffConfig::NoUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.foo"), expected: Some(vec![]),
            actual: Some("bar".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected no children but received [bar]")));
    }

    #[test]
    fn match_xml_when_actual_is_non_empty_and_we_allow_extra_keys() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo/>
        "#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo><bar></bar></foo>
        "#);
        match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(be_empty());
    }

    #[test]
    fn match_xml_when_actual_is_a_super_set() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo><item1/></foo>
        "#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo><item1/><item2/></foo>
        "#);
        match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(be_empty());
    }

    #[test]
    fn match_xml_when_actual_is_empty() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo><bar></bar></foo>
        "#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo/>
        "#);
        match_xml(&expected, &actual, DiffConfig::NoUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.foo"), expected: Some("bar".into()),
            actual: Some(vec![]), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected child <bar/> but was missing")));
    }

    #[test]
    fn match_xml_when_actual_is_different_size() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo><one/><two/><three/><four/></foo>
        "#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo><one/><two/><three/></foo>
        "#);
        match_xml(&expected, &actual, DiffConfig::NoUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.foo"),
            expected: Some("one, two, three, four".into()),
            actual: Some("one, two, three".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected child <four/> but was missing")));
    }

  #[test]
  fn match_xml_comparing_a_list_to_one_with_with_the_same_size_but_different_children() {
    let mut mismatches = vec![];
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo><one/><two/><three/><three/></foo>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo><one/><two/><three/><four/></foo>
    "#);
    match_xml(&expected, &actual, DiffConfig::NoUnexpectedKeys, &mut mismatches, &MatchingRules::default());
    expect!(mismatches.iter()).to(have_count(2));
    let mismatch = mismatches[0].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.foo"),
      expected: Some("one, two, three, three".into()),
      actual: Some("one, two, three, four".into()), mismatch: s!("")}));
    expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Unexpected child <four/>")));
    let mismatch = mismatches[1].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.foo.three"),
      expected: Some("three, three".into()),
      actual: Some("one, two, three, four".into()), mismatch: s!("")}));
    expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected child <three/> but was missing")));
  }

  #[test]
  fn match_xml_comparing_a_list_to_one_where_the_items_are_in_the_wrong_order() {
    let mut mismatches = vec![];
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo><one/><two/><three/></foo>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
    <foo><one/><three/><two/></foo>
    "#);
    match_xml(&expected, &actual, DiffConfig::NoUnexpectedKeys, &mut mismatches, &MatchingRules::default());
    expect!(mismatches.iter()).to(be_empty());
  }

    #[test]
    fn match_xml_with_the_same_text() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo>hello world</foo>
        "#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo>hello world</foo>
        "#);
        match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(be_empty());
    }

    #[test]
    fn match_xml_with_the_same_text_between_nodes() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo>hello<bar/>world</foo>
        "#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo>hello<bar/>world</foo>
        "#);
        match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(be_empty());
    }

    #[test]
    fn match_xml_with_the_different_text() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo>hello world</foo>
        "#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo>hello mars</foo>
        "#);
        match_xml(&expected.clone(), &actual.clone(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.foo.#text"),
            expected: Some("hello world".into()),
            actual: Some("hello mars".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected 'hello world' to be equal to 'hello mars'")));

        mismatches.clear();
        match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &matchingrules!{
            "body" => {
                "$.foo['#text']" => [ MatchingRule::Regex(r"[a-z\s]+".into()) ]
            }
        });
        expect!(mismatches.iter()).to(be_empty());
    }

    #[test]
    fn match_xml_with_the_different_text_between_nodes() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo>hello<bar/>world</foo>
        "#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo>hello<bar/>mars </foo>
        "#);
        match_xml(&expected.clone(), &actual.clone(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.foo.#text"),
            expected: Some("helloworld".into()),
            actual: Some("hellomars".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected 'helloworld' to be equal to 'hellomars'")));

        mismatches.clear();
        match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &matchingrules!{
            "body" => {
                "$.foo['#text']" => [ MatchingRule::Regex(s!("[a-z]+")) ]
            }
        });
        expect!(mismatches.iter()).to(be_empty());
    }

    #[test]
    fn match_xml_with_a_matcher() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo><one/></foo>
        "#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo><one/><one/><one/></foo>
        "#);
        match_xml(&expected, &actual, DiffConfig::NoUnexpectedKeys, &mut mismatches, &matchingrules!{
            "body" => {
                "$.foo" => [ MatchingRule::Type ]
            }
        });
        expect!(mismatches.iter()).to(be_empty());
    }

  #[test]
  fn match_xml_with_unexpected_elements() {
    let mut mismatches = vec![];
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <note> <to>John</to> <from>Jane</from> <subject>Reminder</subject>
        <address> <city>Manchester</city> </address> </note>
        "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <note> <to>John</to> <from>Jane</from> <subject>Reminder</subject>
         <address> <firstName>John</firstName> <lastName>Doe</lastName> <street>Prince Street</street>
         <number>34</number> <city>Manchester</city> </address> </note>
        "#);
    match_xml(&expected.clone(), &actual.clone(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
    expect!(mismatches.iter()).to(be_empty());
  }

  #[test]
  fn match_xml_with_cdata_elements() {
    let mut mismatches = vec![];
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <providerService version="1.0">
         <attribute1>
           <newattribute>
               <date month="11" year="2019"/>
             <name><![CDATA[Surname Name]]></name>
           </newattribute>
           <newattribute2>
             <countryCode>RO</countryCode>
             <hiddenData>ABCD***************010101</hiddenData>
           </newattribute2>
         </attribute1>
       </providerService>
        "#);
    match_xml(&expected.clone(), &expected.clone(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
    expect!(mismatches.iter()).to(be_empty());
  }

  #[test]
  fn match_xml_with_cdata_elements_matching_with_regex() {
    let mut mismatches = vec![];
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <providerService version="1.0">
         <attribute1>
           <newattribute>
               <date month="11" year="2019"/>
             <name><![CDATA[Surname Name]]></name>
           </newattribute>
           <newattribute2>
             <countryCode>RO</countryCode>
             <hiddenData>OWY0NzEyYTAyMmMzZjI2Y2RmYzZiMTcx</hiddenData>
           </newattribute2>
         </attribute1>
       </providerService>
        "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <providerService version="1.0">
         <attribute1>
           <newattribute>
               <date month="11" year="2019"/>
             <name><![CDATA[Surname Name]]></name>
           </newattribute>
           <newattribute2>
             <countryCode>RO</countryCode>
             <hiddenData><![CDATA[Mjc5MGJkNDVjZTI3OWNjYjJjMmYzZTVh]]></hiddenData>
           </newattribute2>
         </attribute1>
       </providerService>
        "#);
    let rules = matchingrules! {
      "body" => {
        "$.providerService.attribute1.newattribute2.hiddenData" => [ MatchingRule::Regex("[a-zA-Z0-9]*".into()) ]
      }
    };
    match_xml(&expected.clone(), &actual.clone(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &rules);
    expect!(mismatches.iter()).to(be_empty());
  }

  #[test_case(r#"<blah xmlns="urn:ns"/>"#, r#"<blah xmlns="urn:ns"/>"#)]
  #[test_case(r#"<b:blah xmlns:b="urn:ns"/>"#, r#"<blah xmlns="urn:ns"/>"#)]
  #[test_case(r#"<blah xmlns="urn:ns"/>"#, r#"<a:blah xmlns:a="urn:ns"/>"#)]
  #[test_case(r#"<b:blah xmlns:b="urn:ns"/>"#, r#"<a:blah xmlns:a="urn:ns"/>"#)]
  fn match_xml_with_different_namespace_declarations(expected: &str, actual: &str) {
    let mut mismatches = vec![];
    let expected = request!(expected);
    let actual = request!(actual);
    match_xml(&expected, &actual, DiffConfig::NoUnexpectedKeys, &mut mismatches, &MatchingRules::default());
    expect!(mismatches.iter()).to(be_empty());
  }

  #[test_case(r#"<foo xmlns="urn:ns"><item/></foo>"#, r#"<foo xmlns="urn:ns"><item/></foo>"#)]
  #[test_case(r#"<b:foo xmlns:b="urn:ns"><b:item/></b:foo>"#, r#"<foo xmlns="urn:ns"><item/></foo>"#)]
  #[test_case(r#"<foo xmlns="urn:ns"><item/></foo>"#, r#"<a:foo xmlns:a="urn:ns"><a:item/></a:foo>"#)]
  #[test_case(r#"<b:foo xmlns:b="urn:ns"><b:item/></b:foo>"#, r#"<a:foo xmlns:a="urn:ns"><a:item/></a:foo>"#)]
  #[test_case(r#"<b:foo xmlns:b="urn:ns"><b:item/></b:foo>"#, r#"<a:foo xmlns:a="urn:ns"><a2:item xmlns:a2="urn:ns"/></a:foo>"#)]
  fn match_xml_with_different_namespace_declarations_on_child_elements(expected: &str, actual: &str) {
    let mut mismatches = vec![];
    let expected = request!(expected);
    let actual = request!(actual);
    match_xml(&expected, &actual, DiffConfig::NoUnexpectedKeys, &mut mismatches, &MatchingRules::default());
    expect!(mismatches.iter()).to(be_empty());
  }

  #[test]
  fn matching_xml_bodies_returns_a_mismatch_when_different_namespaces_are_used() {
    let mut mismatches = vec![];
    let expected = request!("<blah xmlns=\"urn:other\"/>");
    let actual = request!(r#"<blah xmlns="urn:ns"/>"#);
    match_xml(&expected, &actual, DiffConfig::NoUnexpectedKeys, &mut mismatches, &MatchingRules::default());
    expect!(mismatches.iter()).to_not(be_empty());
    expect!(mismatches.first().unwrap()).to(be_equal_to(&Mismatch::BodyMismatch {
      path: "$.urn:other:blah".to_string(),
      expected: Some("urn:other:blah".as_bytes().to_vec()),
      actual: Some("urn:ns:blah".as_bytes().to_vec()),
      mismatch: "Expected 'urn:other:blah' to be equal to 'urn:ns:blah'".to_string()
    }));
  }

  #[test]
  fn matching_xml_bodies_returns_a_mismatch_when_expected_namespace_is_not_used() {
    let mut mismatches = vec![];
    let expected = request!("<blah xmlns=\"urn:other\"/>");
    let actual = request!("<blah/>");
    match_xml(&expected, &actual, DiffConfig::NoUnexpectedKeys, &mut mismatches, &MatchingRules::default());
    expect!(mismatches.iter()).to_not(be_empty());
    expect!(mismatches.first().unwrap()).to(be_equal_to(&Mismatch::BodyMismatch {
      path: "$.urn:other:blah".to_string(),
      expected: Some("urn:other:blah".as_bytes().to_vec()),
      actual: Some("blah".as_bytes().to_vec()),
      mismatch: "Expected 'urn:other:blah' to be equal to 'blah'".to_string()
    }));
  }

  #[test]
  fn matching_xml_bodies_returns_a_mismatch_when_allow_unexpected_keys_is_true_and_no_namespace_is_expected() {
    let mut mismatches = vec![];
    let expected = request!("<blah/>");
    let actual = request!("<blah xmlns=\"urn:ns\"/>");
    match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
    expect!(mismatches.iter()).to_not(be_empty());
    expect!(mismatches.first().unwrap()).to(be_equal_to(&Mismatch::BodyMismatch {
      path: "$.blah".to_string(),
      expected: Some("blah".as_bytes().to_vec()),
      actual: Some("urn:ns:blah".as_bytes().to_vec()),
      mismatch: "Expected 'blah' to be equal to 'urn:ns:blah'".to_string()
    }));
  }

  #[test]
  fn matching_xml_bodies_when_attribute_uses_different_prefix() {
    let mut mismatches = vec![];
    let expected = request!("<foo xmlns:a=\"urn:ns\" a:something=\"100\"/>");
    let actual = request!("<foo xmlns:b=\"urn:ns\" b:something=\"100\"/>");
    match_xml(&expected, &actual, DiffConfig::NoUnexpectedKeys, &mut mismatches, &MatchingRules::default());
    expect!(mismatches.iter()).to(be_empty());
  }

  #[test]
  fn matching_xml_bodies_returns_a_mismatch_when_attribute_uses_different_namespace() {
    let mut mismatches = vec![];
    let expected = request!("<foo xmlns:ns=\"urn:b\" ns:something=\"100\"/>");
    let actual = request!("<foo xmlns:ns=\"urn:a\" ns:something=\"100\"/>");
    match_xml(&expected, &actual, DiffConfig::NoUnexpectedKeys, &mut mismatches, &MatchingRules::default());
    expect!(mismatches.iter()).to_not(be_empty());
    expect!(mismatches.first().unwrap()).to(be_equal_to(&Mismatch::BodyMismatch {
      path: "$.foo.@urn:b:something".to_string(),
      expected: Some("urn:b:something".as_bytes().to_vec()),
      actual: None,
      mismatch: "Expected attribute 'urn:b:something'='100' but was missing".to_string()
    }));
  }

  #[test]
  fn matching_xml_bodies_with_namespaces_and_a_matcher_defined_delegate_to_matcher_for_attribute() {
    let mut mismatches = vec![];
    let expected = request!("<foo xmlns:b=\"urn:ns\" b:something=\"101\"/>");
    let actual = request!("<foo xmlns:a=\"urn:ns\" a:something=\"100\"/>");
    let matching_rules = matchingrules! {
      "body" => { "$.foo['@urn:ns:something']" => [ MatchingRule::Regex(s!("^[0-9]+$")) ] }
    };
    match_xml(&expected, &actual, DiffConfig::NoUnexpectedKeys, &mut mismatches, &matching_rules);
    expect!(mismatches.iter()).to(be_empty());
  }

  #[test]
  fn matching_xml_bodies_with_namespaces_and_a_matcher_defined_delegate_to_the_matcher() {
    let mut mismatches = vec![];
    let expected = request!("<ns:foo xmlns:ns=\"urn:ns\"><ns:something>101</ns:something></ns:foo>");
    let actual = request!("<ns:foo xmlns:ns=\"urn:ns\"><ns:something>100</ns:something></ns:foo>");
    let matching_rules = matchingrules! {
      "body" => { "$['urn:ns:foo']['urn:ns:something'].#text" => [ MatchingRule::Regex(s!("^[0-9]+$")) ] }
    };
    match_xml(&expected, &actual, DiffConfig::NoUnexpectedKeys, &mut mismatches, &matching_rules);
    expect!(mismatches.iter()).to(be_empty());
  }

  #[test]
  fn match_xml_when_an_element_has_different_types_of_children_but_we_allow_unexpected_keys() {
    let mut mismatches = vec![];
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <animals>
          <dog id="1" name="Canine"/>
          <cat id="2" name="Feline"/>
          <wolf id="3" name="Canine"/>
        </animals>
        "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <animals>
          <dog id="1" name="Canine"/>
          <dog id="1" name="Canine"/>
          <cat id="2" name="Feline"/>
          <cat id="2" name="Feline"/>
          <cat id="2" name="Feline"/>
          <wolf id="3" name="Canine"/>
        </animals>
        "#);
    match_xml(&expected.clone(), &actual.clone(), DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
    expect!(mismatches.iter()).to(be_empty());
  }

  #[test]
  fn match_xml_when_an_element_has_different_types_of_children_but_we_do_not_allow_unexpected_keys() {
    let mut mismatches = vec![];
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
      <animals>
          <dog id="1" name="Canine"/>
          <cat id="2" name="Feline"/>
          <wolf id="3" name="Canine"/>
      </animals>
    "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
      <animals>
          <dog id="1" name="Canine"/>
          <dog id="1" name="Canine"/>
          <cat id="2" name="Feline"/>
          <cat id="2" name="Feline"/>
          <cat id="2" name="Feline"/>
          <wolf id="3" name="Canine"/>
      </animals>
    "#);
    match_xml(&expected, &actual, DiffConfig::NoUnexpectedKeys, &mut mismatches, &MatchingRules::default());
    expect!(mismatches.iter()).to(have_count(3));
    let mismatch = mismatches[0].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.animals.cat"),
      expected: Some("cat".into()),
      actual: Some("dog, dog, cat, cat, cat, wolf".into()), mismatch: s!("")}));
    expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Unexpected child <cat/>")));
    let mismatch = mismatches[1].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.animals.cat"),
      expected: Some("cat".into()),
      actual: Some("dog, dog, cat, cat, cat, wolf".into()), mismatch: s!("")}));
    expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Unexpected child <cat/>")));
    let mismatch = mismatches[2].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.animals.dog"),
      expected: Some("dog".into()),
      actual: Some("dog, dog, cat, cat, cat, wolf".into()), mismatch: "Unexpected child <dog/>".into()}));
    expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Unexpected child <dog/>")));
  }

  #[test]
  fn match_xml_type_matcher_when_an_element_has_different_types_of_children() {
    let mut mismatches = vec![];
    let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <animals>
          <dog id="1" name="Canine"/>
          <cat id="2" name="Feline"/>
          <wolf id="3" name="Canine"/>
        </animals>
        "#);
    let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <animals>
          <dog id="1" name="Canine"/>
          <cat id="2" name="Feline"/>
          <dog id="1" name="Canine"/>
          <cat id="2" name="Feline"/>
          <cat id="2" name="Feline"/>
          <wolf id="3" name="Canine"/>
        </animals>
        "#);
    let matching_rules = matchingrules! {
      "body" => {
        "$.animals.*" => [ MatchingRule::Type ],
        "$.animals.*['@id']" => [ MatchingRule::Integer ]
      }
    };
    match_xml(&expected.clone(), &actual.clone(), DiffConfig::NoUnexpectedKeys, &mut mismatches, &matching_rules);
    expect!(mismatches.iter()).to(be_empty());
  }
}
