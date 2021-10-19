To generate the log, run `git log --pretty='* %h - %s (%an, %ad)' TAGNAME..HEAD .` replacing TAGNAME and HEAD as appropriate.

# 0.2.0-beta.6 - Bugfix Release

* 48a6be5f - fix: EachValue was outputting the wrong JSON (Ronald Holshausen, Tue Oct 19 16:35:17 2021 +1100)
* 021a65e6 - bump version to 0.2.0-beta.6 (Ronald Holshausen, Mon Oct 18 13:00:22 2021 +1100)

# 0.2.0-beta.5 - matching synchronous request/response messages

* 2b4b7cc3 - feat(plugins): Support matching synchronous request/response messages (Ronald Holshausen, Fri Oct 15 16:01:50 2021 +1100)
* de3e8296 - bump version to 0.2.0-beta.5 (Ronald Holshausen, Tue Oct 12 15:09:59 2021 +1100)

# 0.2.0-beta.4 - Enhancements for plugins

* a52db737 - feat: record the version of the lib that created the pact in the metadata (Ronald Holshausen, Tue Oct 12 14:55:42 2021 +1100)
* 35ff0993 - feat: record the version of the lib that created the pact in the metadata (Ronald Holshausen, Tue Oct 12 14:52:43 2021 +1100)
* 5f4a578e - refactor: add method to set content type of a body (Ronald Holshausen, Tue Oct 12 14:40:46 2021 +1100)
* d1016565 - refactor: renamed SynchronousMessages -> SynchronousMessage (Ronald Holshausen, Tue Oct 12 14:37:30 2021 +1100)
* 407cc2e5 - fix: notEmpty matching rule defintion should be applied to any primitive value (Ronald Holshausen, Thu Oct 7 14:08:02 2021 +1100)
* 48d662a8 - chore: add docs about the matching rule definition language (Ronald Holshausen, Thu Oct 7 13:29:16 2021 +1100)
* 780a0c97 - bump version to 0.2.0-beta.4 (Ronald Holshausen, Tue Oct 5 15:12:12 2021 +1100)

# 0.2.0-beta.3 - Fixes from master + Updated matching rule definitions

* d608a028 - chore: re-enable matching definition tests (Ronald Holshausen, Tue Oct 5 15:06:11 2021 +1100)
* 57ba661a - chore: fix tests after removing Deserialize, Serialize from Message (Ronald Holshausen, Tue Oct 5 14:55:58 2021 +1100)
* 9fd9e652 - feat: do no write empty comments + added consumer version to metadata (Ronald Holshausen, Thu Sep 30 17:40:56 2021 +1000)
* 5525b039 - feat(plugins): cleaned up the verfier output (Ronald Holshausen, Thu Sep 30 16:19:15 2021 +1000)
* 6d23796f - feat(plugins): support each key and each value matchers (Ronald Holshausen, Wed Sep 29 11:10:46 2021 +1000)
* 6f20282d - Merge branch 'master' into feat/plugins (Ronald Holshausen, Tue Sep 28 14:51:34 2021 +1000)
* 1b994c8d - bump version to 0.1.5 (Ronald Holshausen, Tue Sep 28 13:25:36 2021 +1000)
* 7d46a966 - update changelog for release 0.1.4 (Ronald Holshausen, Tue Sep 28 13:23:07 2021 +1000)
* df715cd5 - feat: support native TLS. Fixes #144 (Matt Fellows, Mon Sep 20 13:00:33 2021 +1000)
* 97ebf555 - feat(plugins): Updated matching rule definitions to include notEmpty and contentType (Ronald Holshausen, Wed Sep 15 12:30:01 2021 +1000)
* 21a7ede5 - feat(plugins): support matching protobuf embedded messages (Ronald Holshausen, Wed Sep 15 12:14:54 2021 +1000)
* 9bf9dc30 - feat(plugins): Add consumer message test builder + persist plugin data (Ronald Holshausen, Tue Sep 14 15:14:17 2021 +1000)
* b71dcabf - refactor(plugins): rename ContentTypeOverride -> ContentTypeHint (Ronald Holshausen, Tue Sep 14 15:08:52 2021 +1000)
* 03ebe632 - Merge branch 'master' into feat/plugins (Ronald Holshausen, Mon Sep 13 12:01:13 2021 +1000)
* 62db0e44 - chore: fix clippy warnings (Ronald Holshausen, Fri Sep 10 17:41:43 2021 +1000)
* 7cfe2883 - chore: fix clippy violation (Ronald Holshausen, Fri Sep 10 15:34:03 2021 +1000)
* 24d4b39a - bump version to 0.2.0-beta.3 (Ronald Holshausen, Fri Sep 10 14:16:03 2021 +1000)

# 0.1.4 - WASM support + native TLS certs

* df715cd5 - feat: support native TLS. Fixes #144 (Matt Fellows, Mon Sep 20 13:00:33 2021 +1000)
* 62db0e44 - chore: fix clippy warnings (Ronald Holshausen, Fri Sep 10 17:41:43 2021 +1000)
* 067ded8f - feat: expose Pact models via WASM (Ronald Holshausen, Sun Sep 5 11:55:26 2021 +1000)
* 80509c01 - chore: add crate to support WASM (Ronald Holshausen, Sat Sep 4 17:32:12 2021 +1000)
* 7c12b03b - bump version to 0.1.4 (Ronald Holshausen, Sat Sep 4 15:30:35 2021 +1000)
* 8fe00acd - chore: correct release script (Ronald Holshausen, Sat Sep 4 15:26:17 2021 +1000)

# 0.2.0-beta.2 - Support for getting interaction markup from plugins

* 997c063f - chore: update release script (Ronald Holshausen, Fri Sep 10 14:08:52 2021 +1000)
* dc41e498 - chore: correct links in rust docs (Ronald Holshausen, Fri Sep 10 14:07:01 2021 +1000)
* 37978075 - feat(plugins): support getting interaction markup from plugins (Ronald Holshausen, Thu Sep 9 12:09:06 2021 +1000)
* f44ecc54 - feat(plugins): make the interaction markup type explicit (Ronald Holshausen, Thu Sep 9 11:47:32 2021 +1000)
* a89eca23 - feat(plugins): support storing interaction markup for interactions in Pact files (Ronald Holshausen, Wed Sep 8 16:39:43 2021 +1000)
* ceb1c35f - Merge branch 'master' into feat/plugins (Ronald Holshausen, Tue Sep 7 10:07:45 2021 +1000)
* 067ded8f - feat: expose Pact models via WASM (Ronald Holshausen, Sun Sep 5 11:55:26 2021 +1000)
* 80509c01 - chore: add crate to support WASM (Ronald Holshausen, Sat Sep 4 17:32:12 2021 +1000)
* 7c12b03b - bump version to 0.1.4 (Ronald Holshausen, Sat Sep 4 15:30:35 2021 +1000)
* 8fe00acd - chore: correct release script (Ronald Holshausen, Sat Sep 4 15:26:17 2021 +1000)
* 53d5d75a - update changelog for release 0.1.3 (Ronald Holshausen, Sat Sep 4 15:25:43 2021 +1000)
* 1dfe83fa - bump version to 0.2.0-beta.2 (Ronald Holshausen, Fri Sep 3 17:19:27 2021 +1000)
* 689a35f4 - chore: fix release script (Ronald Holshausen, Fri Sep 3 17:16:45 2021 +1000)
* 46135a16 - chore: add verifier FFI functions for directory, URL and Pact broker sources (Ronald Holshausen, Tue Aug 24 10:14:46 2021 +1000)

# 0.1.3 - Bugfix Release

* 46135a16 - chore: add verifier FFI functions for directory, URL and Pact broker sources (Ronald Holshausen, Tue Aug 24 10:14:46 2021 +1000)
* c2a9c5cc - bump version to 0.1.3 (Ronald Holshausen, Sun Aug 22 15:15:16 2021 +1000)

# 0.2.0-beta.1 - Support for plugins

* b77498c8 - chore: fix tests after updating plugin API (Ronald Holshausen, Fri Sep 3 16:48:18 2021 +1000)
* c0bdd359 - fix: PluginData configuration is optional (Ronald Holshausen, Thu Sep 2 15:37:01 2021 +1000)
* e8ae81b3 - refactor: matching req/res with plugins requires data from the pact and interaction (Ronald Holshausen, Thu Sep 2 11:57:50 2021 +1000)
* 474b803e - feat(V4): added nontEmpty and semver matchers (Ronald Holshausen, Tue Aug 31 11:58:18 2021 +1000)
* b9aa7ecb - feat(Plugins): allow plugins to override text/binary format of the interaction content (Ronald Holshausen, Mon Aug 30 10:48:04 2021 +1000)
* 7c8fae8b - chore: add additional tests for matching definition parser (Ronald Holshausen, Thu Aug 26 13:49:28 2021 +1000)
* 1a3c1959 - feat(plugins): moved the matching rule definition parser into the models crate (Ronald Holshausen, Wed Aug 25 17:31:17 2021 +1000)
* b40dab60 - feat(plugins): moved the matching rule definition parser into the models crate (Ronald Holshausen, Wed Aug 25 17:27:02 2021 +1000)
* c53dbd79 - bump version to 0.2.0-beta.1 (Ronald Holshausen, Mon Aug 23 13:04:02 2021 +1000)

# 0.2.0-beta.0 - Beta version supporting Pact plugins

* 72b9baaa - chore: update release script for beta versions (Ronald Holshausen, Mon Aug 23 12:57:49 2021 +1000)
* 0c5cede2 - chore: bump models crate to 0.2 (Ronald Holshausen, Mon Aug 23 12:56:14 2021 +1000)
* 75e13fd8 - Merge branch 'master' into feat/plugins (Ronald Holshausen, Mon Aug 23 10:33:45 2021 +1000)
* e3a2660f - chore: fix tests after updating test builders to be async (Ronald Holshausen, Fri Aug 20 12:41:10 2021 +1000)
* 779f099c - feat(plugins): Got generators from plugin working (Ronald Holshausen, Thu Aug 19 17:20:47 2021 +1000)
* 8c61ae96 - feat(plugins): Support plugins with the consumer DSL interaction/response (Ronald Holshausen, Thu Aug 19 15:24:04 2021 +1000)
* b75fea5d - Merge branch 'master' into feat/plugins (Ronald Holshausen, Wed Aug 18 12:27:41 2021 +1000)
* 5a235414 - feat(plugins): order the matching results as plugins mau return them in any order (Ronald Holshausen, Fri Aug 13 17:18:46 2021 +1000)
* 2662241e - feat(plugins): Call out to plugins when comparing content owned by the plugin during verification (Ronald Holshausen, Fri Aug 13 14:29:30 2021 +1000)
* bdfc6f02 - feat(plugins): Load required plugins when verifying a V4 pact (Ronald Holshausen, Wed Aug 11 14:23:54 2021 +1000)

# 0.1.2 - upgrade nom to 7.0

* 21be6bce - chore: upgrade nom to 7.0 #128 (Ronald Holshausen, Sun Aug 22 11:56:33 2021 +1000)
* c274ca1a - fix: use the pacts for verification endpoint if the conusmer selectors are specified #133 (Ronald Holshausen, Sun Aug 22 11:51:22 2021 +1000)

# 0.1.1 - Bugfix Release

* 8bcd1c7e - fix: min/max type matchers must not apply the limits when cascading (Ronald Holshausen, Sun Aug 8 15:50:40 2021 +1000)
* cb1beb99 - feat(plugins): make NoopVariantMatcher public (Ronald Holshausen, Sat Aug 7 14:18:29 2021 +1000)
* 33b308d8 - feat(plugins): fix after merging PR (Ronald Holshausen, Thu Aug 5 12:43:58 2021 +1000)
* 4ca3e02b - Merge pull request #129 from mitre/docpath (Ronald Holshausen, Thu Aug 5 12:16:56 2021 +1000)
* 41e66b30 - feat(plugins): updated matching rules + generators to support working with plugins (Ronald Holshausen, Thu Aug 5 11:58:56 2021 +1000)
* 6124ed0b - refactor: Introduce DocPath struct for path expressions (Caleb Stepanian, Thu Jul 29 12:27:32 2021 -0400)

# 0.1.0 - Final Version

* 533c9e1f - chore: bump minor version of the Pact models crate (Ronald Holshausen, Fri Jul 23 13:15:32 2021 +1000)
* b37a4d02 - chore: add a prelude module to the models crate (Ronald Holshausen, Fri Jul 23 13:10:48 2021 +1000)
* 20f01695 - refactor: Make many JSON parsing functions fallible (Caleb Stepanian, Wed Jul 21 18:04:45 2021 -0400)
* 458fdd15 - refactor: Move path expression functions into path_exp module (Caleb Stepanian, Mon Jul 19 14:22:02 2021 -0400)
* 3dccf866 - refacfor: moved the pact structs to the models crate (Ronald Holshausen, Sun Jul 18 16:58:14 2021 +1000)
* e8046d84 - refactor: moved interaction structs to the models crate (Ronald Holshausen, Sun Jul 18 14:36:03 2021 +1000)
* 31873ee3 - feat: added validation of provider state JSON (Ronald Holshausen, Wed Jul 14 15:44:20 2021 +1000)

# 0.0.5 - Moved structs to models crate + bugfixes and enhancements

* e2151800 - feat: support generating UUIDs with different formats #121 (Ronald Holshausen, Sun Jul 11 12:36:23 2021 +1000)
* e2e10241 - refactor: moved Request and Response structs to the models crate (Ronald Holshausen, Wed Jul 7 18:09:36 2021 +1000)
* 2c3c6ac0 - refactor: moved the header, body and query functions to the model crate (Ronald Holshausen, Wed Jul 7 16:37:28 2021 +1000)
* 9e8b01d7 - refactor: move HttpPart struct to models crate (Ronald Holshausen, Wed Jul 7 15:59:34 2021 +1000)
* 10e8ef87 - refactor: moved http_utils to the models crate (Ronald Holshausen, Wed Jul 7 14:34:20 2021 +1000)
* 01ff9877 - refactor: moved matching rules and generators to models crate (Ronald Holshausen, Sun Jul 4 17:17:30 2021 +1000)
* 357b2390 - refactor: move path expressions to models crate (Ronald Holshausen, Sun Jul 4 15:31:36 2021 +1000)
* 80e3c4e7 - fix: retain the data type for simple expressions #116 (Ronald Holshausen, Sun Jul 4 13:02:43 2021 +1000)
* b1a4c8cb - fix: failing tests #116 (Ronald Holshausen, Sun Jul 4 11:28:20 2021 +1000)
* e21db699 - fix: Keep the original value when injecting from a provider state value so data type is retained #116 (Ronald Holshausen, Sat Jul 3 18:01:34 2021 +1000)
* 8b075d38 - fix: FFI function was exposing a struct from the models crate (Ronald Holshausen, Sun Jun 27 11:30:55 2021 +1000)
* c3c22ea8 - Revert "refactor: moved matching rules and generators to models crate (part 1)" (Ronald Holshausen, Wed Jun 23 14:37:46 2021 +1000)
* d3406650 - refactor: moved matching rules and generators to models crate (part 1) (Ronald Holshausen, Wed Jun 23 12:58:30 2021 +1000)
* 5babb21b - refactor: convert xml_utils to use anyhow::Result (Ronald Holshausen, Tue Jun 22 16:35:56 2021 +1000)
* 9b7ad27d - refactor: moved xml_utils to models crate (Ronald Holshausen, Tue Jun 22 16:30:06 2021 +1000)
* 193da2b9 - chore: fix compiler warnings (Ronald Holshausen, Tue Jun 22 16:06:16 2021 +1000)
* 4db98181 - refactor: move file_utils to the models crate (Ronald Holshausen, Tue Jun 22 16:06:02 2021 +1000)

# 0.0.4 - Refactor + Bugfixes

* 225fd3d8 - chore: upgrade nom to 6.2.0 to resolve lexical-core compiler error (Ronald Holshausen, Tue Jun 22 15:10:31 2021 +1000)
* c392caae - Revert "update changelog for release 0.0.4" (Ronald Holshausen, Tue Jun 22 15:08:36 2021 +1000)
* 6866204f - update changelog for release 0.0.4 (Ronald Holshausen, Tue Jun 22 15:07:17 2021 +1000)
* ed3ae59a - chore: fix for failing parse_era test on CI (Ronald Holshausen, Tue Jun 22 14:55:46 2021 +1000)
* bbc638be - feat(pact file verification): verify consumer and provider sections (Ronald Holshausen, Fri Jun 18 16:52:15 2021 +1000)
* a7c071bc - feat(pact-file-validation): implemented validation of the metadata section (Ronald Holshausen, Wed Jun 16 09:17:28 2021 +1000)
* 00b65dcf - chore: rename pact_file_verifier -> pact_cli (Ronald Holshausen, Mon Jun 14 14:08:24 2021 +1000)
* 6198538d - refactor: move time_utils to pact_models crate (Ronald Holshausen, Fri Jun 11 12:58:26 2021 +1000)
* 5c670814 - refactor: move expression_parser to pact_models crate (Ronald Holshausen, Fri Jun 11 10:51:51 2021 +1000)
* 457aa5fc - fix(V4): Status code matcher was not converted to JSON correctly (Ronald Holshausen, Sun Jun 6 12:53:37 2021 +1000)
* b0ac7141 - feat: support graphql as a JSON content type (Ronald Holshausen, Sat Jun 5 15:14:06 2021 +1000)
* a44cbbee - fix: verifier was returning a mismatch when the expected body is empty #113 (Ronald Holshausen, Sat Jun 5 15:07:22 2021 +1000)
* 4e328d93 - feat: implement verification for RequestResponsePact, Consumer, Provider (Ronald Holshausen, Thu Jun 3 16:59:23 2021 +1000)
* 2f678213 - feat: initial prototype of a pact file verifier (Ronald Holshausen, Thu Jun 3 14:56:16 2021 +1000)

# 0.0.3 - Moved provider state models

* a7b81af - chore: fix clippy violation (Ronald Holshausen, Sat May 29 17:29:06 2021 +1000)
* 7022625 - refactor: move provider state models to the pact models crate (Ronald Holshausen, Sat May 29 17:18:48 2021 +1000)
* ef37cb9 - refactor(V4): extract common message parts into a seperate struct (Ronald Holshausen, Sat May 29 16:38:38 2021 +1000)
* ebb11df - feat(V4): fixed test _ refactored types for match functions (Ronald Holshausen, Sat May 29 14:56:31 2021 +1000)
* 73a53b8 - feat(V4): add an HTTP status code matcher (Ronald Holshausen, Fri May 28 18:40:11 2021 +1000)
* 8e8075b - refactor: move some more structs to the models crate (Ronald Holshausen, Thu May 27 14:34:03 2021 +1000)

# 0.0.2 - FFI support

* 82711d6 - chore: use a feature to enable FFI representation in the core crates (Ronald Holshausen, Mon May 3 12:14:02 2021 +1000)
* 6af4d3f - feat: allow ffi bindings to set spec version (Matt Fellows, Sun May 2 22:41:41 2021 +1000)

# 0.0.1 - Refactor: moved content type and body code from pact_matching

* 5ea36db - refactor: move content handling functions to pact_models crate (Ronald Holshausen, Sun Apr 25 13:12:22 2021 +1000)
* d010630 - chore: cleanup deprecation and compiler warnings (Ronald Holshausen, Sun Apr 25 12:23:30 2021 +1000)
* 3dd610a - refactor: move structs and code dealing with bodies to a seperate package (Ronald Holshausen, Sun Apr 25 11:20:47 2021 +1000)
* a725ab1 - feat(V4): added synchronous request/response message formats (Ronald Holshausen, Sat Apr 24 16:05:12 2021 +1000)
* 4bcd94f - refactor: moved OptionalBody and content types to pact models crate (Ronald Holshausen, Thu Apr 22 14:01:56 2021 +1000)
* 80812d0 - refactor: move Consumer and Provider structs to models crate (Ronald Holshausen, Thu Apr 22 13:11:03 2021 +1000)
* 220fb5e - refactor: move the PactSpecification enum to the pact_models crate (Ronald Holshausen, Thu Apr 22 11:18:26 2021 +1000)
* 83d3d60 - chore: bump version to 0.0.1 (Ronald Holshausen, Thu Apr 22 10:52:04 2021 +1000)
* 9962e0e - chore: add required metadata fields to Cargo manifest (Ronald Holshausen, Thu Apr 22 10:45:14 2021 +1000)
* 34e7dcd - chore: add a pact models crate (Ronald Holshausen, Thu Apr 22 10:04:40 2021 +1000)

# 0.0.0 - First Release
