use super::Mismatch;
use super::DiffConfig;
use sxd_document::dom::*;
use std::collections::btree_map::BTreeMap;
use itertools::Itertools;
use crate::models::matchingrules::*;
use crate::matchers::*;
use onig::Regex;
use crate::models::xml_utils::parse_bytes;
use crate::models::HttpPart;

pub fn match_xml(expected: &dyn HttpPart, actual: &dyn HttpPart, config: DiffConfig,
    mismatches: &mut Vec<super::Mismatch>, matchers: &MatchingRules) {
    let expected_result = parse_bytes(&expected.body().value());
    let actual_result = parse_bytes(&actual.body().value());

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
        compare_element(&vec![s!("$")], &expected_root_node.unwrap().element().unwrap(),
            &actual_root_node.unwrap().element().unwrap(), config, mismatches, matchers);
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
                  Err(format!("Expected '{}' to match '{}'", actual.name().local_part(), regex))
                }
              },
              Err(err) => Err(format!("'{}' is not a valid regular expression - {}", regex, err))
            }
          },
          MatchingRule::Type => if self.name() == actual.name() {
             Ok(())
          } else {
             Err(format!("Expected '{}' to be the same type as '{}'", self.name().local_part(),
                  actual.name().local_part()))
          },
          MatchingRule::MinType(min) => if actual.children().len() < min {
             Err(format!("Expected '{}' to have at least {} children", actual.name().local_part(), min))
          } else {
             Ok(())
          },
          MatchingRule::MaxType(max) => if actual.children().len() > max {
             Err(format!("Expected '{}' to have at most {} children", actual.name().local_part(), max))
          } else {
             Ok(())
          },
          MatchingRule::MinMaxType(min, max) => if actual.children().len() < min {
            Err(format!("Expected '{}' to have at least {} children", actual.name().local_part(), min))
          } else if actual.children().len() > max {
            Err(format!("Expected '{}' to have at most {} children", actual.name().local_part(), max))
          } else {
            Ok(())
          },
          MatchingRule::Equality => {
             if self.name() == actual.name() {
                 Ok(())
             } else {
                  Err(format!("Expected '{}' to be equal to '{}'", self.name().local_part(),
                      actual.name().local_part()))
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
      log::debug!("calling match_values");
      match_values("body", path, matchers.clone(), expected, actual)
    } else {
      expected.matches(actual, &MatchingRule::Equality).map_err(|err| vec![err])
    };
    log::debug!("Comparing '{:?}' to '{:?}' at path '{}' -> {:?}", expected, actual, path_to_string(path), matcher_result);
    match matcher_result {
        Err(messages) => {
          for message in messages {
            mismatches.push(Mismatch::BodyMismatch {
              path: path_to_string(path),
              expected: Some(expected.name().local_part().into()),
              actual: Some(actual.name().local_part().into()),
              mismatch: message.clone()
            })
          }
        },
        Ok(_) => {
            let mut new_path = path.to_vec();
            new_path.push(s!(actual.name().local_part()));
            compare_attributes(&new_path, expected, actual, config.clone(), mismatches, matchers);
            compare_children(&new_path, expected, actual, config.clone(), mismatches, matchers);
            compare_text(&new_path, expected, actual, mismatches, matchers);
        }
    }
}

fn compare_attributes(path: &Vec<String>, expected: &Element, actual: &Element, config: DiffConfig,
    mismatches: &mut Vec<super::Mismatch>, matchers: &MatchingRules) {
    let expected_attributes: BTreeMap<String, String> = expected.attributes()
        .iter().map(|attr| (s!(attr.name().local_part()), s!(attr.value()))).collect();
    let actual_attributes: BTreeMap<String, String> = actual.attributes()
        .iter().map(|attr| (s!(attr.name().local_part()), s!(attr.value()))).collect();
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
            if actual_attributes.contains_key(key) {
                let mut p = path.to_vec();
                p.push(s!("@") + key);
                compare_value(&p, value, &actual_attributes[key], mismatches, matchers);
            } else {
                mismatches.push(Mismatch::BodyMismatch { path: path_to_string(path),
                    expected: Some(format!("{:?}", expected_attributes).into()),
                    actual: Some(format!("{:?}", actual_attributes).into()),
                    mismatch: format!("Expected attribute '{}'='{}' but was missing", key, value)});
            }
        }
    }
}

fn children<'a>(element: &Element<'a>) -> Vec<ChildOfElement<'a>> {
    element.children().iter().cloned().filter(|child| child.element().is_some()).collect()
}

fn desc_children<'a>(children: &Vec<ChildOfElement<'a>>) -> String {
    children.iter().map(|child| child.element().unwrap().name().local_part()).join(", ")
}

fn compare_children(path: &Vec<String>, expected: &Element, actual: &Element, config: DiffConfig,
    mismatches: &mut Vec<super::Mismatch>, matchers: &MatchingRules) {
    let mut expected_children = children(expected);
    let actual_children = children(actual);
    if matchers.matcher_is_defined("body", &path) {
        if !expected_children.is_empty() {
            let expected_example = expected_children[0].clone();
            expected_children.resize(actual_children.len(), expected_example);
        }
    } else {
        if expected_children.is_empty() && !actual_children.is_empty() && config == DiffConfig::NoUnexpectedKeys {
          mismatches.push(Mismatch::BodyMismatch { path: path_to_string(path),
              expected: Some(desc_children(&expected_children).into()),
              actual: Some(desc_children(&actual_children).into()),
              mismatch: format!("Expected an empty List but received [{}]", desc_children(&actual_children))});
        } else if expected_children.len() != actual_children.len() {
            if config == DiffConfig::AllowUnexpectedKeys && expected_children.len() > actual_children.len() {
                mismatches.push(Mismatch::BodyMismatch { path: path_to_string(path),
                    expected: Some(desc_children(&expected_children).into()),
                    actual: Some(desc_children(&actual_children).into()),
                    mismatch: format!("Expected a List with at least {} element(s) but received {} element(s)",
                        expected_children.len(), actual_children.len())});

            } else if config == DiffConfig::NoUnexpectedKeys {
                mismatches.push(Mismatch::BodyMismatch { path: path_to_string(path),
                    expected: Some(desc_children(&expected_children).into()),
                    actual: Some(desc_children(&actual_children).into()),
                    mismatch: format!("Expected a List with {} element(s) but received {} element(s)",
                        expected_children.len(), actual_children.len())});
            }
        }
    }

    for ((i, exp), act) in expected_children.iter().enumerate().zip(actual_children.iter()) {
        let expected = exp.element().unwrap();
        let mut p = path.to_vec();
        p.push(format!("{}", i));
        compare_element(&p, &expected, &act.element().unwrap(),
            config.clone(), mismatches, matchers);
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

    macro_rules! request {
      ($e:expr) => (Request { body: OptionalBody::Present($e.as_bytes().to_vec()), .. Request::default() })
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
        <bar/>
        "#);
        match_xml(&expected, &actual, DiffConfig::AllowUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$"), expected: Some("foo".into()),
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
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.blah"), expected: Some("{\"a\": \"c\", \"c\": \"b\"}".into()),
            actual: Some("{\"a\": \"b\"}".into()), mismatch: s!("")}));
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
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.foo"), expected: Some("{\"something\": \"100\", \"somethingElse\": \"100\"}".into()),
            actual: Some("{\"something\": \"100\", \"somethingDifferent\": \"100\"}".into()), mismatch: s!("")}));
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
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected an empty List but received [bar]")));
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
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected a List with 1 element(s) but received 0 element(s)")));
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
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected a List with 4 element(s) but received 3 element(s)")));
    }

    #[test]
    fn match_xml_when_actual_is_same_size_but_different_children() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo><one/><two/><three/></foo>
        "#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo><one/><two/><four/></foo>
        "#);
        match_xml(&expected.clone(), &actual.clone(), DiffConfig::NoUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.foo.2"),
            expected: Some("three".into()),
            actual: Some("four".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected 'three' to be equal to 'four'")));

        mismatches.clear();
        match_xml(&expected.clone(), &actual.clone(), DiffConfig::NoUnexpectedKeys, &mut mismatches, &matchingrules!{
            "body" => {
                "$.foo" => [ MatchingRule::Type ]
            }
        });
        expect!(mismatches.iter()).to(have_count(1));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.foo.2"),
            expected: Some("three".into()),
            actual: Some("four".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected 'three' to be the same type as 'four'")));
    }

    #[test]
    fn match_xml_when_actual_is_same_size_but_wrong_order() {
        let mut mismatches = vec![];
        let expected = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo><one/><two/></foo>
        "#);
        let actual = request!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <foo><two/><one/></foo>
        "#);
        match_xml(&expected.clone(), &actual.clone(), DiffConfig::NoUnexpectedKeys, &mut mismatches, &MatchingRules::default());
        expect!(mismatches.iter()).to(have_count(2));
        let mismatch = mismatches[0].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.foo.0"),
            expected: Some("one".into()),
            actual: Some("two".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected 'one' to be equal to 'two'")));
        let mismatch = mismatches[1].clone();
        expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: s!("$.foo.1"),
            expected: Some("two".into()),
            actual: Some("one".into()), mismatch: s!("")}));
        expect!(mismatch_message(&mismatch)).to(be_equal_to(s!("Expected 'two' to be equal to 'one'")));
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

}
