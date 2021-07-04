use expectest::prelude::*;

use crate::bodies::OptionalBody;
use crate::content_types::JSON;

// #[test]
// fn apply_generator_to_empty_body_test() {
//   let generators = Generators::default();
//   expect!(generators.apply_body_generators(&GeneratorTestMode::Provider, &OptionalBody::Empty, Some(TEXT.clone()), &hashmap!{})).to(be_equal_to(OptionalBody::Empty));
//   expect!(generators.apply_body_generators(&GeneratorTestMode::Provider, &OptionalBody::Null, Some(TEXT.clone()), &hashmap!{})).to(be_equal_to(OptionalBody::Null));
//   expect!(generators.apply_body_generators(&GeneratorTestMode::Provider, &OptionalBody::Missing, Some(TEXT.clone()), &hashmap!{})).to(be_equal_to(OptionalBody::Missing));
// }
//
// #[test]
// fn do_not_apply_generators_if_there_are_no_body_generators() {
//   let generators = Generators::default();
//   let body = OptionalBody::Present("{\"a\": 100, \"b\": \"B\"}".into(), None);
//   expect!(generators.apply_body_generators(&GeneratorTestMode::Provider, &body, Some(JSON.clone()), &hashmap!{})).to(be_equal_to(body));
// }
//
// #[test]
// fn apply_generator_to_text_body_test() {
//   let generators = Generators::default();
//   let body = OptionalBody::Present("some text".into(), None);
//   expect!(generators.apply_body_generators(&GeneratorTestMode::Provider, &body, Some(TEXT.clone()), &hashmap!{})).to(be_equal_to(body));
// }
//
// #[test]
// fn does_not_change_body_if_there_are_no_generators() {
//   let body = OptionalBody::Present("{\"a\": 100, \"b\": \"B\"}".into(), None);
//   let generators = Generators::default();
//   let processed = generators.apply_body_generators(&GeneratorTestMode::Provider, &body, Some(JSON.clone()),
//                                                    &hashmap!{});
//   expect!(processed).to(be_equal_to(body));
// }
