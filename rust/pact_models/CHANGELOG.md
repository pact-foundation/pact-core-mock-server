To generate the log, run `git log --pretty='* %h - %s (%an, %ad)' TAGNAME..HEAD .` replacing TAGNAME and HEAD as appropriate.

# 1.0.1 - Maintenance Release

* 43a8cae1 - chore: clean up deprecation warnings (Ronald Holshausen, Mon Nov 28 13:19:31 2022 +1100)
* 19cc4ca3 - chore: Update dependencies (Ronald Holshausen, Mon Nov 28 12:12:18 2022 +1100)
* e21d3454 - feat: add FFI function to parse JSON to a Pact model (Ronald Holshausen, Fri Nov 11 17:00:36 2022 +1100)
* 66e84e9d - bump version to 1.0.1 (Ronald Holshausen, Mon Nov 7 10:27:49 2022 +1100)

# 1.0.0 - Maintenance Release

* f91dc00d - fix: try loosen dependencies to fix dependency cycle issue (Ronald Holshausen, Mon Nov 7 10:15:19 2022 +1100)
* 87430af9 - chore: set onig to the crate version (Ronald Holshausen, Fri Nov 4 16:41:32 2022 +1100)
* 0bd3f51b - bump version to 0.4.7 (Ronald Holshausen, Fri Nov 4 15:50:52 2022 +1100)

# 0.4.6 - Maintenance Release

* 6ad00a5d - fix: Update onig to latest master to fix  Regex Matcher Fails On Valid Inputs #214 (Ronald Holshausen, Fri Nov 4 15:23:50 2022 +1100)
* ac4fe73f - chore: fix to release scripts (Ronald Holshausen, Wed Sep 7 10:51:01 2022 +1000)
* bcdc6443 - bump version to 0.4.6 (Ronald Holshausen, Fri Aug 26 11:39:48 2022 +1000)

# 0.4.5 - Bugfix Release

* b6bba540 - fix(FFI): FFI passes matching rules and generators for paths etc. with a path of $ (Ronald Holshausen, Fri Aug 26 11:22:19 2022 +1000)
* a37d621e - chore: update the doc comments on matchingrules! macro (Ronald Holshausen, Wed Aug 24 17:25:04 2022 +1000)
* 46b7c1a2 - bump version to 0.4.5 (Ronald Holshausen, Thu Aug 18 13:35:04 2022 +1000)

# 0.4.4 - Updated dependencies

* bd53ad0d - chore: Upgrade dependencies (uuid, tracing) (Ronald Holshausen, Thu Aug 18 13:32:28 2022 +1000)
* 3e5acc2c - bump version to 0.4.4 (Ronald Holshausen, Thu Aug 18 13:05:45 2022 +1000)

# 0.4.3 - Bugfix Release

* 74a36a1b - fix: Matching rule parser was not handling decimal values correctly (Ronald Holshausen, Wed Aug 17 13:19:58 2022 +1000)
* cf4b52eb - chore: fix failing time expression tests after a dependency update (Ronald Holshausen, Mon Aug 15 17:09:04 2022 +1000)
* 9e0dbc26 - chore: clean up some deprecation warnings in Pact models (Ronald Holshausen, Wed Aug 10 11:38:45 2022 +1000)
* 91cb99a9 - bump version to 0.4.3 (Ronald Holshausen, Mon Aug 8 12:59:10 2022 +1000)

# 0.4.2 - Maintenance Release

* 13dc3b52 - Merge branch 'master' into feat/verifier-multiple-transports (Ronald Holshausen, Wed Aug 3 11:55:05 2022 +1000)
* 3d73e3c2 - Removed dependency on time v0.1 (Daan Oosterveld, Wed Jul 6 15:56:29 2022 +0200)
* 2ca2fe49 - fix: add function to display binary data in a meaningful way (Ronald Holshausen, Mon Aug 1 17:39:34 2022 +1000)
* 1972a74a - feat: Detect Pactbroker responses from the URL content #199 (Ronald Holshausen, Mon Jun 6 14:48:06 2022 +1000)
* 4da79d75 - chore: lock the tracing crate version (Ronald Holshausen, Mon May 9 17:04:29 2022 +1000)
* 137e3503 - bump version to 0.4.2 (Ronald Holshausen, Mon May 9 14:18:35 2022 +1000)

# 0.4.1 - switch pact_models to use tracing crate

* 9d30a441 - chore: switch pact_models to use tracing crate (Ronald Holshausen, Mon May 9 13:24:37 2022 +1000)
* 0dd9a176 - bump version to 0.4.1 (Ronald Holshausen, Fri Apr 22 12:07:23 2022 +1000)

# 0.4.0 - Updated V4 model interfaces

* 6de6c229 - feat: Add functions to calc unique key to V4 interaction trait (Ronald Holshausen, Fri Apr 22 12:00:57 2022 +1000)
* 7e3e2e18 - feat: add method to V4Pact to find an interaction by ID (Ronald Holshausen, Thu Apr 21 12:25:09 2022 +1000)
* 49640c5f - chore: minor update to release scripts (Ronald Holshausen, Wed Apr 13 15:32:46 2022 +1000)
* 97b49229 - bump version to 0.3.4 (Ronald Holshausen, Wed Apr 13 13:47:10 2022 +1000)

# 0.3.3 - Bugfix Release

* 73ae0ef0 - fix: Upgrade reqwest to 0.11.10 to resolve #156 (Ronald Holshausen, Wed Apr 13 13:31:55 2022 +1000)
* 42b1a461 - Merge branch 'master' into feat/plugin-mock-server (Ronald Holshausen, Mon Mar 21 16:01:33 2022 +1100)
* 345b0011 - feat: support mock servers provided from plugins (Ronald Holshausen, Mon Mar 21 15:59:46 2022 +1100)
* daa2c101 - feat: add mutable iteraction over Pact interactions (Ronald Holshausen, Fri Mar 18 16:55:34 2022 +1100)
* 0bc98834 - bump version to 0.3.3 (Ronald Holshausen, Fri Mar 18 16:08:30 2022 +1100)
* 0ca9f62b - update changelog for release 0.3.2 (Ronald Holshausen, Fri Mar 18 16:06:30 2022 +1100)
* 01ac989b - fix: was missing setter to set the transport with V4 interactions (Ronald Holshausen, Fri Mar 18 16:04:00 2022 +1100)
* a075f679 - bump version to 0.3.2 (Ronald Holshausen, Fri Mar 18 14:39:20 2022 +1100)
* e82a67fb - update changelog for release 0.3.1 (Ronald Holshausen, Fri Mar 18 14:37:03 2022 +1100)
* 7fd87eb9 - feat: store the transport with V4 interactions to support mockservers from plugins (Ronald Holshausen, Fri Mar 18 14:30:20 2022 +1100)
* 27e41386 - bump version to 0.3.1 (Ronald Holshausen, Fri Mar 4 11:32:56 2022 +1100)

# 0.3.2 - Maintenance Release

* 01ac989b - fix: was missing setter to set the transport with V4 interactions (Ronald Holshausen, Fri Mar 18 16:04:00 2022 +1100)
* a075f679 - bump version to 0.3.2 (Ronald Holshausen, Fri Mar 18 14:39:20 2022 +1100)

# 0.3.1 - Plugin Support

* 7fd87eb9 - feat: store the transport with V4 interactions to support mockservers from plugins (Ronald Holshausen, Fri Mar 18 14:30:20 2022 +1100)
* 27e41386 - bump version to 0.3.1 (Ronald Holshausen, Fri Mar 4 11:32:56 2022 +1100)

# 0.3.0 - Ported the date-time expressions from Pact-JVM

* 0aa55cfe - feat: wired the date-time expression parsers into the generators (Ronald Holshausen, Thu Mar 3 18:01:46 2022 +1100)
* 98b887f0 - feat: Implemented date-time expression parser (from Pact-JVM) (Ronald Holshausen, Thu Mar 3 16:34:42 2022 +1100)
* 318037a7 - feat: Implemented time part in date-time expressions (Ronald Holshausen, Thu Mar 3 16:05:01 2022 +1100)
* 66442251 - feat: Implement the base part of time expressions (Ronald Holshausen, Thu Mar 3 13:09:03 2022 +1100)
* b8ea7240 - feat: Implemented date expression parser (from Pact-JVM) (Ronald Holshausen, Tue Mar 1 14:46:57 2022 +1100)
* 2927e979 - feat: ported the date manipulation functions from Pact-JVM #180 (Ronald Holshausen, Thu Feb 17 16:14:52 2022 +1100)
* 12a7b78c - chore: bump minor version (Ronald Holshausen, Wed Feb 16 15:06:35 2022 +1100)
* c7d39ca6 - bump version to 0.2.8 (Ronald Holshausen, Mon Jan 17 10:47:48 2022 +1100)

# 0.2.7 - Bugfix Release

* c2089645 - fix: log crate version must be fixed across all crates (including plugin driver) (Ronald Holshausen, Fri Jan 14 16:10:50 2022 +1100)
* ff49f33a - chore: update docs on matching rule definitions (Ronald Holshausen, Wed Jan 5 15:07:26 2022 +1100)
* 7b23378f - feat: some matching rules should not cascade (Ronald Holshausen, Thu Dec 30 13:32:04 2021 +1100)
* 83c36db7 - bump version to 0.2.7 (Ronald Holshausen, Wed Dec 29 13:33:06 2021 +1100)

# 0.2.6 - Bugfix Release

* 7b2e8538 - fix: DocPath join needs to detect numeric values (Ronald Holshausen, Wed Dec 29 13:23:47 2021 +1100)
* e5fd165d - refactor: Move is_values_matcher logic to MatchingRule (Ronald Holshausen, Wed Dec 29 10:05:03 2021 +1100)
* 41a52319 - fix: values_matcher_defined should include EachValue matcher (Ronald Holshausen, Wed Dec 29 09:48:02 2021 +1100)
* b7f967e0 - fix: `match` arms have incompatible types (Ronald Holshausen, Thu Dec 23 17:48:20 2021 +1100)
* 39338c46 - fix: Some matching rules do not have associated configuration (Ronald Holshausen, Thu Dec 23 14:02:50 2021 +1100)
* deb30e92 - bump version to 0.2.6 (Ronald Holshausen, Thu Dec 23 11:57:34 2021 +1100)

# 0.2.5 - Bugfix Release

* e1e0b43e - fix: matching definition parser was incorrectly merging multiple definitions (Ronald Holshausen, Thu Dec 23 11:48:09 2021 +1100)
* 85bffe40 - bump version to 0.2.5 (Ronald Holshausen, Thu Dec 23 09:23:57 2021 +1100)

# 0.2.4 - Maintenance Release

* b5fd82e5 - feat: add method to DocPath to return the parent path (Ronald Holshausen, Wed Dec 22 18:06:43 2021 +1100)
* a3f74711 - fix: Docpath join was escaping * (Ronald Holshausen, Wed Dec 22 17:23:12 2021 +1100)
* cc0775e2 - bump version to 0.2.4 (Ronald Holshausen, Tue Dec 21 13:16:32 2021 +1100)

# 0.2.3 - Maintenance Release

* 481762f0 - feat: add function to detect if a string is a matching definition (Ronald Holshausen, Tue Dec 21 13:06:31 2021 +1100)
* 98e364b2 - chore: add docpath method to return a vector of strings (Ronald Holshausen, Thu Dec 16 17:16:36 2021 +1100)
* c707a8c0 - feat: add a method to join a value onto a doc path (Ronald Holshausen, Thu Dec 16 16:03:27 2021 +1100)
* 77892ab1 - bump version to 0.2.3 (Ronald Holshausen, Mon Nov 29 11:57:36 2021 +1100)

# 0.2.2 - Fixes to the matching rule parser

* 58039496 - chore: fix imports on expression parser (Ronald Holshausen, Mon Nov 29 11:49:49 2021 +1100)
* cb0f7df8 - Revert "update changelog for release 0.2.2" (Ronald Holshausen, Mon Nov 29 11:42:37 2021 +1100)
* 33a37c9d - update changelog for release 0.2.2 (Ronald Holshausen, Mon Nov 29 11:39:56 2021 +1100)
* 3207cb49 - feat: implement each key and aech value matching rule definitions (Ronald Holshausen, Wed Nov 24 14:28:42 2021 +1100)
* 2db6a46f - refactor: test_env_log has been replaced with test_log (Ronald Holshausen, Tue Nov 23 16:15:02 2021 +1100)
* d3234684 - feat: update matcher defintions to support references (Ronald Holshausen, Tue Nov 23 16:13:49 2021 +1100)
* 682df9e4 - feat: update matcher defintions to include the semver matcher (Ronald Holshausen, Tue Nov 23 14:24:02 2021 +1100)
* 20a275fb - feat: Improve the error message format for matching rule definitions (Ronald Holshausen, Mon Nov 22 15:21:57 2021 +1100)
* a859d0e1 - fix: make sure metadata entries are correctly encoded when downgrading a pact (Ronald Holshausen, Wed Nov 17 16:54:15 2021 +1100)
* d32ae2b1 - bump version to 0.2.2 (Ronald Holshausen, Tue Nov 16 10:38:04 2021 +1100)

# 0.2.1 - Update V4 models to support FFI + plugins

* 15b8f08f - feat: add functions to return mutable references to the V4 model trait (Ronald Holshausen, Tue Nov 16 10:03:03 2021 +1100)
* 7c150c8b - feat(plugins): Support message tests via FFI that use plugins (Ronald Holshausen, Wed Nov 10 17:03:49 2021 +1100)
* fa83806c - feat: add mutable methods to Pact model traits (Ronald Holshausen, Tue Nov 9 16:04:16 2021 +1100)
* 2027537d - refactor: update FFI to use V4 models internally (Ronald Holshausen, Mon Nov 8 16:44:39 2021 +1100)
* b42b7ad9 - chore: fix clippy warnings and errors (Ronald Holshausen, Wed Nov 3 15:45:29 2021 +1100)
* 296b4370 - chore: update project to Rust 2021 edition (Ronald Holshausen, Fri Oct 22 10:44:48 2021 +1100)
* dac05481 - bump version to 0.2.1 (Ronald Holshausen, Thu Oct 21 17:52:39 2021 +1100)

# 0.2.0 - Pact V4 + Plugins release

* e72d602a - chore: bump pact models to non-beta version (Ronald Holshausen, Thu Oct 21 17:47:17 2021 +1100)
* 3fb9258a - bump version to 0.2.0-beta.7 (Ronald Holshausen, Tue Oct 19 16:41:08 2021 +1100)

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
