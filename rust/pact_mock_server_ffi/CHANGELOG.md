To generate the log, run `git log --pretty='* %h - %s (%an, %ad)' TAGNAME..HEAD .` replacing TAGNAME and HEAD as appropriate.

# 0.0.17 - V4 features + bugfixes/enhancements

* eb19188 - feat: allow FFI mockserver to log to a memory buffer (Ronald Holshausen, Sun May 30 15:31:01 2021 +1000)
* 7022625 - refactor: move provider state models to the pact models crate (Ronald Holshausen, Sat May 29 17:18:48 2021 +1000)
* 64adcdc - feat: add a boolean return value for all FFI interaction functions #108 (Ronald Holshausen, Sat May 29 16:16:36 2021 +1000)
* 59e23f4 - fix: message pact needed matchingrules + generators (Matt Fellows, Sat May 29 15:16:32 2021 +1000)
* 73a53b8 - feat(V4): add an HTTP status code matcher (Ronald Holshausen, Fri May 28 18:40:11 2021 +1000)
* ad0a72e - fix: message_reify was returning a pointer to a Rust string (Ronald Holshausen, Fri May 28 11:11:35 2021 +1000)
* eb4b328 - fix: ok so maybe let's see if this works on linux 🤷 (Matt Fellows, Thu May 27 23:41:09 2021 +1000)
* 52768a3 - fix: http_consumer_feature_test on linux (Matt Fellows, Thu May 27 23:09:45 2021 +1000)
* cf679bd - fix: message pact feature test (Matt Fellows, Thu May 27 21:33:24 2021 +1000)
* 0fcb371 - chore: ignore failing message interaction tests for now (Ronald Holshausen, Thu May 27 14:32:03 2021 +1000)
* 0c9391d - Merge pull request #101 from pact-foundation/feat/ffi-consumer-message-pact (Matt Fellows, Thu May 27 12:57:58 2021 +1000)
* 810106d - Merge pull request #100 from pact-foundation/feat/more-consumer-ffi-matchers (Ronald Holshausen, Thu May 27 11:17:53 2021 +1000)
* 0dc59d0 - feat: allow callers to add metadata into resulting pact file (Matt Fellows, Sun May 23 14:40:23 2021 +1000)
* 7d50453 - fix: update flakey ffi feature test (Matt Fellows, Sun May 23 13:56:45 2021 +1000)
* 5024e17 - feat: allow messages to have binary payloads (Matt Fellows, Sat May 22 21:50:57 2021 +1000)
* b329c39 - wip: support binary data in message_with_contents (Matt Fellows, Fri May 21 21:33:01 2021 +1000)
* 34c8701 - feat: return full message from the reify function (Matt Fellows, Fri May 21 11:06:10 2021 +1000)
* 9020ee6 - wip: use ContentType struct to detect message types (Matt Fellows, Thu May 20 22:13:23 2021 +1000)
* 10e54b8 - feat: add metadata to consumer message ffi interface (Matt Fellows, Wed May 19 11:43:36 2021 +1000)
* 413e9a5 - feat: initial consumer FFI based message pact (Matt Fellows, Tue May 18 23:37:49 2021 +1000)
* 9338268 - chore: add feature test for FFI matchers (Matt Fellows, Sun May 16 17:23:25 2021 +1000)
* 784a023 - feat: support path, query and header matching in FFI (Matt Fellows, Sun May 16 14:17:59 2021 +1000)
* 4224088 - chore: add shasums to all release artifacts (Matt Fellows, Wed May 5 15:18:31 2021 +1000)
* b3d52b3 - chore: correct the dyn lib name for OSX (Ronald Holshausen, Mon May 3 15:11:39 2021 +1000)
* 82711d6 - chore: use a feature to enable FFI representation in the core crates (Ronald Holshausen, Mon May 3 12:14:02 2021 +1000)
* 6af4d3f - feat: allow ffi bindings to set spec version (Matt Fellows, Sun May 2 22:41:41 2021 +1000)
* 735c9e7 - chore: bump pact_matching to 0.9 (Ronald Holshausen, Sun Apr 25 13:50:18 2021 +1000)
* fb373b4 - chore: bump version to 0.0.2 (Ronald Holshausen, Sun Apr 25 13:40:52 2021 +1000)
* d010630 - chore: cleanup deprecation and compiler warnings (Ronald Holshausen, Sun Apr 25 12:23:30 2021 +1000)
* 3dd610a - refactor: move structs and code dealing with bodies to a seperate package (Ronald Holshausen, Sun Apr 25 11:20:47 2021 +1000)
* 80b7148 - feat(V4): Updated consumer DSL to set comments + mock server initial support for V4 pacts (Ronald Holshausen, Fri Apr 23 17:58:10 2021 +1000)
* 4bcd94f - refactor: moved OptionalBody and content types to pact models crate (Ronald Holshausen, Thu Apr 22 14:01:56 2021 +1000)
* 80812d0 - refactor: move Consumer and Provider structs to models crate (Ronald Holshausen, Thu Apr 22 13:11:03 2021 +1000)
* 63fcf49 - feat: enable consumer code to use the new Value matcher (Matt Fellows, Wed Apr 7 14:01:44 2021 +1000)
* 4f48223 - chore: add some tests for process_object (Ronald Holshausen, Fri Mar 26 15:00:32 2021 +1100)
* 5fba704 - chore: update the conan packages for inclusion in conan index (Ronald Holshausen, Sun Mar 14 15:22:48 2021 +1100)
* 38c8834 - bump version to 0.0.17 (Ronald Holshausen, Sun Mar 14 14:50:41 2021 +1100)

# 0.0.16 - Bugfix Release

* f4881db - feat: set non-hard coded install name on Mac dylib (Matt Fellows, Wed Feb 24 14:29:52 2021 +1100)
* abcf6a7 - bump version to 0.0.16 (Ronald Holshausen, Mon Feb 8 15:55:11 2021 +1100)

# 0.0.15 - use a file system lock when merging pact files

* 9976e80 - feat: added read locks and a mutex guard to reading and writing pacts (Ronald Holshausen, Mon Feb 8 11:58:52 2021 +1100)
* 49a3cf2 - refactor: use bytes crate instead of vector of bytes for body content (Ronald Holshausen, Sun Feb 7 14:43:40 2021 +1100)
* e43fdb8 - chore: upgrade maplit, itertools (Audun Halland, Mon Jan 11 05:30:10 2021 +0100)
* cbd61b3 - bump version to 0.0.15 (Ronald Holshausen, Mon Jan 11 10:16:18 2021 +1100)

# 0.0.14 - Updated dependencies

* 5e5c320 - chore: upgrade rand, rand_regex (Audun Halland, Sat Jan 9 09:33:13 2021 +0100)
* 3a28a6c - chore: upgrade regex, chrono-tz (Audun Halland, Sat Jan 9 11:12:49 2021 +0100)
* 1ac3548 - chore: upgrade env_logger to 0.8 (Audun Halland, Sat Jan 9 09:50:27 2021 +0100)
* 5b60ec0 - chore: get rid of nom dupes by using tree_magic_mini (Audun Halland, Sat Jan 9 08:06:22 2021 +0100)
* 3a6945e - chore: Upgrade reqwest to 0.11 and hence tokio to 1.0 (Ronald Holshausen, Wed Jan 6 15:34:47 2021 +1100)
* 39188df - bump version to 0.0.14 (Ronald Holshausen, Tue Jan 5 13:50:07 2021 +1100)

# 0.0.13 - Cleanup outdated crates

* 4636982 - chore: update other crates to use Tokio 1.0 (Ronald Holshausen, Mon Jan 4 17:26:59 2021 +1100)
* e1f6ddb - test: fix windows test by removing newline before EOF (Audun Halland, Sun Jan 3 04:47:47 2021 +0100)
* ade0237 - deps: remove ffi/hyper 0.10, update lock file (Audun Halland, Sun Jan 3 03:48:21 2021 +0100)
* 3b73b71 - deps(pact_mock_server_ffi): remove formdata, add multipart (Audun Halland, Sun Jan 3 03:40:32 2021 +0100)
* cc104f7 - test(ffi): multipart file (Audun Halland, Sun Jan 3 01:57:38 2021 +0100)
* c0f039f - bump version to 0.0.13 (Ronald Holshausen, Thu Dec 31 14:04:01 2020 +1100)

# 0.0.12 - Supports generators associated with array contains matcher variants

* fe06178 - chore: update pact_matching and pact_mock_server crates to latest (Ronald Holshausen, Thu Dec 31 13:47:43 2020 +1100)
* 329c1d7 - Merge branch 'feat/add-version-to-ffi' (Ronald Holshausen, Wed Dec 30 15:01:19 2020 +1100)
* 51eef86 - fix: return version of the mock server via FFI without heap allocation #80 (Ronald Holshausen, Wed Dec 30 14:55:08 2020 +1100)
* 5e56ecb - refactor: support generators associated with array contains matcher variants (Ronald Holshausen, Tue Dec 29 11:46:56 2020 +1100)
* 638059d - feat: return version of the mock server via FFI (Matt Fellows, Mon Nov 30 18:06:55 2020 +1100)
* 7d4cd5b - chore: update publish.md (Ronald Holshausen, Fri Oct 30 13:28:19 2020 +1100)
* be35049 - bump version to 0.0.12 (Ronald Holshausen, Fri Oct 30 12:28:20 2020 +1100)

# 0.0.11 - Bugfix Release

* b4c4de8 - chore: upgrade to latest Onig crate (Ronald Holshausen, Wed Oct 28 09:59:36 2020 +1100)
* 2be4f8c - bump version to 0.0.11 (Ronald Holshausen, Fri Oct 16 11:39:33 2020 +1100)

# 0.0.10 - arrayContains matcher + text/xml content type

* f090323 - feat: updated integration JSON to handle array contains matcher (Ronald Holshausen, Thu Oct 15 15:31:47 2020 +1100)
* f334a4f - refactor: introduce a MatchingContext into all matching functions + delgate to matchers for collections (Ronald Holshausen, Mon Oct 12 14:06:00 2020 +1100)
* 7fbc731 - chore: bump minor version of matching lib (Ronald Holshausen, Fri Oct 9 10:42:33 2020 +1100)
* facc898 - refactor: moved the shutdown code to a method in mock server crate (Ronald Holshausen, Sun Oct 4 11:56:10 2020 +1100)
* 6f530a1 - bump version to 0.0.10 (Ronald Holshausen, Mon Sep 28 12:38:44 2020 +1000)

# 0.0.9 - CORS pre-flight support

* 1019ed6 - chore: update to the latest mock server and matching crates (Ronald Holshausen, Mon Sep 28 12:30:09 2020 +1000)
* 625730b - update changelog for release 0.0.9 (Ronald Holshausen, Mon Sep 28 12:25:39 2020 +1000)
* 29ba743 - feat: add a mock server config struct (Ronald Holshausen, Thu Sep 24 10:30:59 2020 +1000)
* b64c4eb - bump version to 0.0.9 (Ronald Holshausen, Mon Sep 14 17:06:34 2020 +1000)

# 0.0.9 - CORS pre-flight support

* 29ba743 - feat: add a mock server config struct (Ronald Holshausen, Thu Sep 24 10:30:59 2020 +1000)
* b64c4eb - bump version to 0.0.9 (Ronald Holshausen, Mon Sep 14 17:06:34 2020 +1000)

# 0.0.8 - Supports TLS CA certs + better content type handling

* 6cba6ad - feat: implemented basic message verification with the verifier cli (Ronald Holshausen, Mon Sep 14 13:48:27 2020 +1000)
* 2d44ffd - chore: bump minor version of the matching crate (Ronald Holshausen, Mon Sep 14 12:06:37 2020 +1000)
* 814c416 - refactor: added a trait for interactions, renamed Interaction to RequestResponseInteraction (Ronald Holshausen, Sun Sep 13 17:09:41 2020 +1000)
* a05bcbb - refactor: renamed Pact to RequestResponsePact (Ronald Holshausen, Sun Sep 13 12:45:34 2020 +1000)
* 1185e4c - chore: update to latest matching and mock server crates (Ronald Holshausen, Sun Aug 23 14:52:47 2020 +1000)
* 2f16b34 - feat: add function to retrieve the TLS CA cert (Matt Fellows, Sun Aug 9 00:18:22 2020 +1000)
* b242eb1 - refactor: changed the remaining uses of the old content type methods (Ronald Holshausen, Sun Jun 28 17:11:51 2020 +1000)
* 5316030 - feat: added a struct for handling content types (Ronald Holshausen, Sun Jun 28 13:31:22 2020 +1000)
* ed207a7 - chore: updated readmes for docs site (Ronald Holshausen, Sun Jun 28 10:04:09 2020 +1000)
* 359a944 - chore: update versions in readmes (Ronald Holshausen, Sat Jun 27 13:21:24 2020 +1000)
* 2ab1a9d - refactor: extract generate_regex_value to a seperate function (Ronald Holshausen, Sat Jun 27 13:17:54 2020 +1000)
* a6f926e - chore: document conan publish steps (Ronald Holshausen, Wed Jun 24 12:11:10 2020 +1000)
* 01d6ed2 - bump version to 0.0.8 (Ronald Holshausen, Wed Jun 24 11:25:43 2020 +1000)

# 0.0.7 - Updated XML Matching and Date/Time parsing

* f9b0cae - chore: update to latest matching crate (Ronald Holshausen, Wed Jun 24 11:16:38 2020 +1000)
* a15edea - chore: try set the content type on the body if known (Ronald Holshausen, Tue Jun 23 16:53:32 2020 +1000)
* 45fc1a0 - fix: cleanup warnings and fixed test (Ronald Holshausen, Fri Jun 12 10:51:44 2020 +1000)
* 875d7a1 - refactor: changed date/time parsing to support Java DateTimeFormatter format (Ronald Holshausen, Tue Jun 9 17:56:30 2020 +1000)
* 4996c0f - feat: make body processing functions public so other language impl can use them (Ronald Holshausen, Thu Jun 4 16:02:55 2020 +1000)
* f71c57a - bump version to 0.0.7 (Ronald Holshausen, Wed May 27 10:47:48 2020 +1000)

# 0.0.6 - Refactor

* 643ac2f - refactor: moved the multipart handling code into re-usable functions (Ronald Holshausen, Sun May 24 13:48:21 2020 +1000)
* 1104d65 - bump version to 0.0.6 (Ronald Holshausen, Sun May 24 12:05:35 2020 +1000)

# 0.0.5 - matching multipart form posts

* dac517b - feat: implemented FFI support for matching multipart form posts (Ronald Holshausen, Sun May 24 11:17:58 2020 +1000)
* bea787c - chore: bump matching crate version to 0.6.0 (Ronald Holshausen, Sat May 23 17:56:04 2020 +1000)
* 78854a8 - bump version to 0.0.5 (Ronald Holshausen, Fri May 15 16:38:58 2020 +1000)

# 0.0.4 - Bugfix Release

* 2d11c17 - chore: set version of patch matching crate to 0.5.14 (Ronald Holshausen, Fri May 15 16:33:21 2020 +1000)
* c93e364 - fix: correct the backing array list for query parameters from FFI call (Ronald Holshausen, Wed May 13 11:52:55 2020 +1000)
* 9c84713 - fix: correct the backing arary list for headers from FFI call (Ronald Holshausen, Tue May 12 16:45:38 2020 +1000)
* d5c4f96 - bump version to 0.0.4 (Ronald Holshausen, Tue May 12 12:56:30 2020 +1000)
* 8f01bc6 - bump version to 0.0.4 (Ronald Holshausen, Tue May 12 12:54:56 2020 +1000)

# 0.0.3 - matching of binary payloads + fixes handling provider state parameters

* 3a12b6f - fix: incorrectly handling provider state parameters from FFI call (Ronald Holshausen, Fri May 8 16:31:45 2020 +1000)
* 708db47 - feat: implement matching of binary payloads (application/octet-stream) (Ronald Holshausen, Fri May 8 15:52:03 2020 +1000)
* 136c61b - feat: update FFI to support provider states with parameters (Ronald Holshausen, Wed May 6 15:50:10 2020 +1000)
* 754a483 - chore: updated itertools to latest (Ronald Holshausen, Wed May 6 15:49:27 2020 +1000)
* dffcd42 - chore: fix release script (Ronald Holshausen, Tue May 5 17:20:18 2020 +1000)
* 4601c49 - chore: fix release script (Ronald Holshausen, Tue May 5 17:17:15 2020 +1000)
* 95899fe - bump version to 0.0.3 (Ronald Holshausen, Tue May 5 17:14:49 2020 +1000)
* f64b668 - chore: fix release script (Ronald Holshausen, Tue May 5 17:10:32 2020 +1000)
* cc5c79c - chore: fix release script (Ronald Holshausen, Tue May 5 17:09:15 2020 +1000)

# 0.0.2 - Bugfix Release

* 3260ce3 - chore: fix release script (Ronald Holshausen, Tue May 5 17:03:33 2020 +1000)
* b41fc58 - chore: correct the version in the conan recipe (Ronald Holshausen, Tue May 5 16:59:21 2020 +1000)
* a686afd - chore: use a hashmap instead of a vec to store the Pact models (Ronald Holshausen, Tue May 5 12:54:18 2020 +1000)
* 75c965e - fix: correct issue with headers/query with multiple values (Ronald Holshausen, Tue May 5 12:53:28 2020 +1000)
* 2eba288 - fix: update conan test packages to use updated API (Ronald Holshausen, Tue May 5 12:52:28 2020 +1000)
* 2679653 - fix: for failing integration test (Ronald Holshausen, Fri May 1 16:16:30 2020 +1000)
* 1cc568c - chore: generate headers for both C and C++ (Ronald Holshausen, Fri May 1 15:42:53 2020 +1000)
* 9b1c192 - fix: use a single result enum #66 (Ronald Holshausen, Fri May 1 15:42:27 2020 +1000)
* 90b93f1 - chore: create a conan package for the dll/so (Ronald Holshausen, Fri May 1 14:45:49 2020 +1000)
* da885a3 - feat: add support for TLS with the mock server #65 (Ronald Holshausen, Thu Apr 30 16:41:30 2020 +1000)
* a45d0c3 - fix: FFI mismatch json should have the actual values as UTF-8 string not bytes #64 (Ronald Holshausen, Thu Apr 30 11:16:25 2020 +1000)
* 2003d7b - chore: roll back onig crate to 4.3.3 #64 (Ronald Holshausen, Thu Apr 30 09:50:48 2020 +1000)
* 411f697 - chore: correct some clippy warnings (Ronald Holshausen, Wed Apr 29 16:49:36 2020 +1000)
* 47cc589 - chore: added clippy and fixed resulting lint errors (Ronald Holshausen, Wed Apr 29 15:32:55 2020 +1000)
* b30fd2d - feat: add support functions for regular expressions for foreign DSLs (Ronald Holshausen, Tue Apr 28 17:33:48 2020 +1000)
* a36b490 - chore: update documentation (Ronald Holshausen, Tue Apr 28 11:25:09 2020 +1000)
* 8a7e4d1 - chore: bump version in conan package (Ronald Holshausen, Tue Apr 28 10:04:55 2020 +1000)
* 4287f0e - fix: correct the windows lib name in conan package (Ronald Holshausen, Mon Apr 27 14:18:01 2020 +1000)
* c1015d5 - fix: correct the windows lib name in conan package (Ronald Holshausen, Mon Apr 27 14:02:44 2020 +1000)
* a0d701e - fix: Macos on conan package (Ronald Holshausen, Fri Apr 24 15:23:53 2020 +1000)
* bb1e35e - fix: Windows URL on conan package (Ronald Holshausen, Fri Apr 24 15:00:24 2020 +1000)
* a13c0fc - fix: Add OSX to the conan package (Ronald Holshausen, Fri Apr 24 14:46:49 2020 +1000)

# 0.0.1 - Changes to support C++ DSL

* 1655472 - chore: remove the version for the release artifact file name (Ronald Holshausen, Fri Apr 24 11:06:24 2020 +1000)
* f84e672 - chore: update mock server library to latest (Ronald Holshausen, Fri Apr 24 11:00:34 2020 +1000)
* 43de9c3 - chore: update matching library to latest (Ronald Holshausen, Fri Apr 24 10:20:55 2020 +1000)
* 5f8d0a0 - feat: handle bodies with embedded matchers and generators (Ronald Holshausen, Thu Apr 23 12:25:05 2020 +1000)
* 0613180 - feat: add FFI function to create mock server from Pact handle (Ronald Holshausen, Wed Apr 22 17:01:34 2020 +1000)
* fc17d30 - feat: implemented FFI methods for query parameters, headers and bodies (Ronald Holshausen, Wed Apr 22 13:03:52 2020 +1000)
* 9ec8817 - feat: added interaction handle (Ronald Holshausen, Fri Apr 17 18:28:11 2020 +1000)
* d357ad2 - feat: create conan package for linking with C++ projects (Ronald Holshausen, Thu Apr 16 18:00:24 2020 +1000)
* 7e89ca9 - chore: update matching crate to latest (Ronald Holshausen, Thu Apr 16 14:06:02 2020 +1000)
* d457221 - chore: update dependant crates to use mock server lib 0.7.0 (Ronald Holshausen, Sun Jan 19 11:31:21 2020 +1100)
* aa80ff1 - pact_mock_server_ffi: Remove extern crate from lib.rs (Audun Halland, Sun Nov 17 23:11:46 2019 +0100)
* 713cd6a - Explicit edition 2018 in Cargo.toml files (Audun Halland, Sat Nov 16 23:55:37 2019 +0100)
* f86cb19 - Merge branch 'master' into feature/create-mock-server-several-times (Ronald Holshausen, Sun Oct 13 14:13:53 2019 +1100)
* d4298f1 - Allow to recreate mock-server (Jérémy Demeule, Sun Oct 6 20:47:15 2019 +0200)
* 7425e6f - Fix pack_mock_server C api and usage (Jérémy Demeule, Sun Oct 6 20:11:57 2019 +0200)
* 2488ab9 - Merge branch 'master' of https://github.com/pact-foundation/pact-reference (milleniumbug, Wed Sep 18 11:32:03 2019 +0200)
* 37b1083 - chore: update release scripts to cross compile (Ronald Holshausen, Sat Sep 7 14:25:16 2019 +1000)
* 80404ab - bump version to 0.0.1 (Ronald Holshausen, Sat Sep 7 12:43:08 2019 +1000)

# 0.0.0 - First Release


