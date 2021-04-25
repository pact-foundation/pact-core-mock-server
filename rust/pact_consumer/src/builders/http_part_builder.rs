use std::collections::HashMap;

#[cfg(test)]
#[allow(unused_imports)]
use env_logger;
#[cfg(test)]
use regex::Regex;

use pact_matching::models::matchingrules::MatchingRules;
use pact_models::bodies::OptionalBody;

use crate::prelude::*;

/// Various methods shared between `RequestBuilder` and `ResponseBuilder`.
pub trait HttpPartBuilder {
    /// (Implementation detail.) This function fetches the mutable state that's
    /// needed to update this builder's `headers`. You should not need to use
    /// this under normal circumstances.
    ///
    /// This function has two return values because its job is to split a single
    /// `&mut` into two `&mut` pointing to sub-objects, which has to be done
    /// carefully in Rust.
    #[doc(hidden)]
    fn headers_and_matching_rules_mut(&mut self) -> (&mut HashMap<String, Vec<String>>, &mut MatchingRules);

    /// (Implementation detail.) This function fetches the mutable state that's
    /// needed to update this builder's `body`. You should not need to use this
    /// under normal circumstances.
    ///
    /// This function has two return values because its job is to split a single
    /// `&mut` into two `&mut` pointing to sub-objects, which has to be done
    /// carefully in Rust.
    #[doc(hidden)]
    fn body_and_matching_rules_mut(&mut self) -> (&mut OptionalBody, &mut MatchingRules);

    /// Specify a header pattern.
    ///
    /// ```
    /// use pact_consumer::prelude::*;
    /// use pact_consumer::*;
    /// use pact_consumer::builders::RequestBuilder;
    /// use regex::Regex;
    ///
    /// RequestBuilder::default()
    ///     .header("X-Simple", "value")
    ///     .header("X-Digits", term!("^[0-9]+$", "123"));
    /// ```
    #[allow(clippy::option_map_unit_fn)]
    fn header<N, V>(&mut self, name: N, value: V) -> &mut Self
    where
        N: Into<String>,
        V: Into<StringPattern>,
    {
      let name = name.into();
      let value = value.into();
      {
        let (headers, rules) = self.headers_and_matching_rules_mut();
        if headers.contains_key(&name) {
          headers.get_mut(&name).map(|val| val.push(value.to_example()));
        } else {
          headers.insert(name.clone(), vec![value.to_example()]);
        }
        value.extract_matching_rules(&name, rules.add_category("header"))
      }
      self
    }

    /// Set the `Content-Type` header.
    fn content_type<CT>(&mut self, content_type: CT) -> &mut Self
    where
        CT: Into<StringPattern>,
    {
        self.header("Content-Type", content_type)
    }

    /// Set the `Content-Type` header to `text/html`.
    fn html(&mut self) -> &mut Self {
        self.content_type("text/html")
    }

    /// Set the `Content-Type` header to `application/json; charset=utf-8`,
    /// with enough flexibility to cover common variations.
    fn json_utf8(&mut self) -> &mut Self {
        self.content_type(term!(
            "^application/json; charset=(utf|UTF)-8$",
            "application/json; charset=utf-8"
        ))
    }

    /// Specify a body literal. This does not allow using patterns.
    ///
    /// ```
    /// use pact_consumer::prelude::*;
    /// use pact_consumer::builders::RequestBuilder;
    ///
    /// RequestBuilder::default().body("Hello");
    /// ```
    ///
    /// TODO: We may want to change this to `B: Into<Vec<u8>>` depending on what
    /// happens with https://github.com/pact-foundation/pact-reference/issues/19
    fn body<B: Into<String>>(&mut self, body: B) -> &mut Self {
        let body = body.into();
        {
            let (body_ref, _) = self.body_and_matching_rules_mut();
            *body_ref = OptionalBody::Present(body.into(), None);
        }
        self
    }

  /// Specify a body literal with content type. This does not allow using patterns.
  ///
  /// ```
  /// use pact_consumer::prelude::*;
  /// use pact_consumer::builders::RequestBuilder;
  ///
  /// RequestBuilder::default().body2("Hello", "plain/text");
  /// ```
  ///
  /// TODO: We may want to change this to `B: Into<Vec<u8>>` depending on what
  /// happens with https://github.com/pact-foundation/pact-reference/issues/19
  fn body2<B: Into<String>>(&mut self, body: B, content_type: B) -> &mut Self {
    let body = body.into();
    {
      let (body_ref, _) = self.body_and_matching_rules_mut();
      *body_ref = OptionalBody::Present(body.into(), content_type.into().parse().ok());
    }
    self
  }

    /// Specify the body as `JsonPattern`, possibly including special matching
    /// rules.
    ///
    /// ```
    /// use pact_consumer::prelude::*;
    /// use pact_consumer::*;
    /// use pact_consumer::builders::RequestBuilder;
    ///
    /// RequestBuilder::default().json_body(json_pattern!({
    ///     "message": like!("Hello"),
    /// }));
    /// ```
    fn json_body<B: Into<JsonPattern>>(&mut self, body: B) -> &mut Self {
        let body = body.into();
        {
            let (body_ref, rules) = self.body_and_matching_rules_mut();
            *body_ref = OptionalBody::Present(body.to_example().to_string().into(), Some("application/json".into()));
            body.extract_matching_rules("$", rules.add_category("body"));
        }
        self
    }
}

#[test]
fn header_pattern() {
    let application_regex = Regex::new("application/.*").unwrap();
    let pattern = PactBuilder::new("C", "P")
        .interaction("I", |i| {
            i.request.header(
                "Content-Type",
                Term::new(application_regex, "application/json"),
            );
        })
        .build();
    let good = PactBuilder::new("C", "P")
        .interaction("I", |i| {
            i.request.header("Content-Type", "application/xml");
        })
        .build();
    let bad = PactBuilder::new("C", "P")
        .interaction("I", |i| { i.request.header("Content-Type", "text/html"); })
        .build();
    assert_requests_match!(good, pattern);
    assert_requests_do_not_match!(bad, pattern);
}

#[test]
fn body_literal() {
    let pattern = PactBuilder::new("C", "P")
        .interaction("I", |i| { i.request.body("Hello"); })
        .build();
    let good = PactBuilder::new("C", "P")
        .interaction("I", |i| { i.request.body("Hello"); })
        .build();
    let bad = PactBuilder::new("C", "P")
        .interaction("I", |i| { i.request.body("Bye"); })
        .build();
    assert_requests_match!(good, pattern);
    assert_requests_do_not_match!(bad, pattern);
}

#[test]
fn json_body_pattern() {
    let pattern = PactBuilder::new("C", "P")
        .interaction("I", |i| {
            i.request.json_body(json_pattern!({
                "message": Like::new(json_pattern!("Hello")),
            }));
        })
        .build();
    let good = PactBuilder::new("C", "P")
        .interaction("I", |i| {
            i.request.json_body(json_pattern!({ "message": "Goodbye" }));
        })
        .build();
    let bad = PactBuilder::new("C", "P")
        .interaction("I", |i| {
            i.request.json_body(json_pattern!({ "message": false }));
        })
        .build();
    assert_requests_match!(good, pattern);
    assert_requests_do_not_match!(bad, pattern);
}
