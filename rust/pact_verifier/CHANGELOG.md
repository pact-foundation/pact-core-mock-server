To generate the log, run `git log --pretty='* %h - %s (%an, %ad)' TAGNAME..HEAD .` replacing TAGNAME and HEAD as appropriate.

# 0.10.6 - V 4 featues + bugfixes

* 905118e - Merge pull request #109 from tonynguyenit18/fix/unmatched-expected-and-response-headers-with-multiple-value (Ronald Holshausen, Sun May 30 10:19:51 2021 +1000)
* eef6b08 - fix: correct headers attribute with multiple values might not be matched (Tony Nguyen, Sat May 29 20:55:35 2021 +0700)
* 44e7eb4 - chore: cleanup deprecation warnings (Ronald Holshausen, Sat May 29 17:55:04 2021 +1000)
* a7b81af - chore: fix clippy violation (Ronald Holshausen, Sat May 29 17:29:06 2021 +1000)
* 7022625 - refactor: move provider state models to the pact models crate (Ronald Holshausen, Sat May 29 17:18:48 2021 +1000)
* 73a53b8 - feat(V4): add an HTTP status code matcher (Ronald Holshausen, Fri May 28 18:40:11 2021 +1000)
* 62a653c - chore: remove unused imports (Matt Fellows, Thu May 27 23:40:27 2021 +1000)
* af6721a - feat: rename callback_timeout to request_timeout, and support timeouts for all http requests during verification (Matt Fellows, Thu May 27 09:04:05 2021 +1000)
* 4224088 - chore: add shasums to all release artifacts (Matt Fellows, Wed May 5 15:18:31 2021 +1000)
* b84420d - chore: add a verification test for matching values (Ronald Holshausen, Sun May 2 14:30:55 2021 +1000)
* 735c9e7 - chore: bump pact_matching to 0.9 (Ronald Holshausen, Sun Apr 25 13:50:18 2021 +1000)
* fb373b4 - chore: bump version to 0.0.2 (Ronald Holshausen, Sun Apr 25 13:40:52 2021 +1000)
* d010630 - chore: cleanup deprecation and compiler warnings (Ronald Holshausen, Sun Apr 25 12:23:30 2021 +1000)
* 3dd610a - refactor: move structs and code dealing with bodies to a seperate package (Ronald Holshausen, Sun Apr 25 11:20:47 2021 +1000)
* a725ab1 - feat(V4): added synchronous request/response message formats (Ronald Holshausen, Sat Apr 24 16:05:12 2021 +1000)
* 80b7148 - feat(V4): Updated consumer DSL to set comments + mock server initial support for V4 pacts (Ronald Holshausen, Fri Apr 23 17:58:10 2021 +1000)
* 04d810b - feat(V4): display comments when verifying an interaction (Ronald Holshausen, Fri Apr 23 11:48:25 2021 +1000)
* b4bffdb - chore: correct missing changelog (Ronald Holshausen, Fri Apr 23 10:48:18 2021 +1000)
* 4bcd94f - refactor: moved OptionalBody and content types to pact models crate (Ronald Holshausen, Thu Apr 22 14:01:56 2021 +1000)
* 80812d0 - refactor: move Consumer and Provider structs to models crate (Ronald Holshausen, Thu Apr 22 13:11:03 2021 +1000)
* 220fb5e - refactor: move the PactSpecification enum to the pact_models crate (Ronald Holshausen, Thu Apr 22 11:18:26 2021 +1000)
* 2a55838 - chore: fix some Rust 2021 lint warnings (Ronald Holshausen, Wed Apr 21 16:46:47 2021 +1000)
* 9ad1474 - Merge branch 'master' of https://github.com/pact-foundation/pact-reference (Matt Fellows, Sun Apr 11 22:14:30 2021 +1000)
* a0f6a1d - refactor: Use Anyhow instead of `io::Result` (Caleb Stepanian, Wed Apr 7 16:17:35 2021 -0400)
* dcd6bed - bump version to 0.8.16 (Matt Fellows, Wed Apr 7 14:09:37 2021 +1000)

# 0.10.6 - Bugfix Release

* 63fcf49 - feat: enable consumer code to use the new Value matcher (Matt Fellows, Wed Apr 7 14:01:00 +1000)

# 0.10.5 - Bugfix Release

* 32ba4b1 - chore: update pact_matching to latest (Matt Fellows, Wed Apr 7 13:12:36 2021 +1000)
* fdae684 - update changelog for release 0.10.5 (Matt Fellows, Wed Apr 7 12:29:58 2021 +1000)
* 31e5c9c - chore: update pact_matching dependency for pact_verifier (Matt Fellows, Wed Apr 7 12:21:27 2021 +1000)
* 7cded70 - update changelog for release 0.10.5 (Matt Fellows, Wed Apr 7 12:10:43 2021 +1000)
* 89240d8 - Merge pull request #95 from pact-foundation/fix/params-missing-on-provider-state-change (Ronald Holshausen, Sun Mar 14 17:20:01 2021 +1100)
* 17682dc - fix: add missing params to provider state change executor (Matt Fellows, Sat Mar 13 08:37:46 2021 +1100)
* 656201c - feat: add exponental deplay the pact broker client retries #94 (Ronald Holshausen, Sun Mar 14 14:16:57 2021 +1100)
* e38634e - feat: add retry to the pact broker client post and put #94 (Ronald Holshausen, Sun Mar 14 14:12:26 2021 +1100)
* 8541751 - feat: add retry to the pact broker client fetch #94 (Ronald Holshausen, Sun Mar 14 13:04:20 2021 +1100)
* 4fe65fb - feat(V4): Update matching code to use matchingRules.content for V4 messages (Ronald Holshausen, Mon Mar 8 12:10:31 2021 +1100)
* 4dc5373 - bump version to 0.10.5 (Ronald Holshausen, Wed Feb 10 15:54:50 2021 +1100)

# 0.10.5 - pw

* 31e5c9c - chore: update pact_matching dependency for pact_verifier (Matt Fellows, Wed Apr 7 12:21:27 2021 +1000)
* 7cded70 - update changelog for release 0.10.5 (Matt Fellows, Wed Apr 7 12:10:43 2021 +1000)
* 89240d8 - Merge pull request #95 from pact-foundation/fix/params-missing-on-provider-state-change (Ronald Holshausen, Sun Mar 14 17:20:01 2021 +1100)
* 17682dc - fix: add missing params to provider state change executor (Matt Fellows, Sat Mar 13 08:37:46 2021 +1100)
* 656201c - feat: add exponental deplay the pact broker client retries #94 (Ronald Holshausen, Sun Mar 14 14:16:57 2021 +1100)
* e38634e - feat: add retry to the pact broker client post and put #94 (Ronald Holshausen, Sun Mar 14 14:12:26 2021 +1100)
* 8541751 - feat: add retry to the pact broker client fetch #94 (Ronald Holshausen, Sun Mar 14 13:04:20 2021 +1100)
* 4fe65fb - feat(V4): Update matching code to use matchingRules.content for V4 messages (Ronald Holshausen, Mon Mar 8 12:10:31 2021 +1100)
* 4dc5373 - bump version to 0.10.5 (Ronald Holshausen, Wed Feb 10 15:54:50 2021 +1100)

# 0.10.5 - Bugfix Release

* 89240d8 - Merge pull request #95 from pact-foundation/fix/params-missing-on-provider-state-change (Ronald Holshausen, Sun Mar 14 17:20:01 2021 +1100)
* 17682dc - fix: add missing params to provider state change executor (Matt Fellows, Sat Mar 13 08:37:46 2021 +1100)
* 656201c - feat: add exponental deplay the pact broker client retries #94 (Ronald Holshausen, Sun Mar 14 14:16:57 2021 +1100)
* e38634e - feat: add retry to the pact broker client post and put #94 (Ronald Holshausen, Sun Mar 14 14:12:26 2021 +1100)
* 8541751 - feat: add retry to the pact broker client fetch #94 (Ronald Holshausen, Sun Mar 14 13:04:20 2021 +1100)
* 4fe65fb - feat(V4): Update matching code to use matchingRules.content for V4 messages (Ronald Holshausen, Mon Mar 8 12:10:31 2021 +1100)
* 4dc5373 - bump version to 0.10.5 (Ronald Holshausen, Wed Feb 10 15:54:50 2021 +1100)

# 0.10.4 - add final newline to verifier output

* 8c2152e - fix: add final newline to verifier output (Jest will overwrite it with the test name) (Ronald Holshausen, Tue Feb 9 14:15:19 2021 +1100)
* 0a2aad9 - chore: correct release script (Ronald Holshausen, Mon Feb 8 16:14:20 2021 +1100)
* f952467 - bump version to 0.10.4 (Ronald Holshausen, Mon Feb 8 16:04:33 2021 +1100)

# 0.10.3 - Fixes + add callback timeout option for verifcation callbacks

* 49a3cf2 - refactor: use bytes crate instead of vector of bytes for body content (Ronald Holshausen, Sun Feb 7 14:43:40 2021 +1100)
* 4afa86a - fix: add callback timeout option for verifcation callbacks (Ronald Holshausen, Sat Feb 6 12:27:32 2021 +1100)
* 74bd53f - fix: include test results for successful interactions when publishing verification results #92 (Ronald Holshausen, Mon Feb 1 11:24:33 2021 +1100)
* a27ce14 - fix: in callback executors, pass self by value to avoid lifetime issues (Ronald Holshausen, Tue Jan 26 18:41:06 2021 +1100)
* dccd16f - chore: wrap verifier callbacks in Arc<Self> so they can be called across threads (Ronald Holshausen, Tue Jan 26 16:24:09 2021 +1100)
* e5b1f93 - fix: clippy error (Ronald Holshausen, Mon Jan 25 10:26:58 2021 +1100)
* e10047a - bump version to 0.10.3 (Ronald Holshausen, Mon Jan 25 10:20:40 2021 +1100)

# 0.10.2 - made pact broker module public so it can be used by other crates

* c8f7091 - feat: made pact broker module public so it can be used by other crates (Ronald Holshausen, Sun Jan 24 18:24:30 2021 +1100)
* fb4e996 - bump version to 0.10.2 (Ronald Holshausen, Mon Jan 11 10:28:35 2021 +1100)

# 0.10.1 - Updated dependencies

* 1ac3548 - chore: upgrade env_logger to 0.8 (Audun Halland, Sat Jan 9 09:50:27 2021 +0100)
* 9a8a63f - chore: upgrade quickcheck (Audun Halland, Sat Jan 9 08:46:51 2021 +0100)
* 3a6945e - chore: Upgrade reqwest to 0.11 and hence tokio to 1.0 (Ronald Holshausen, Wed Jan 6 15:34:47 2021 +1100)
* b79e3a1 - bump version to 0.10.1 (Ronald Holshausen, Tue Jan 5 14:24:47 2021 +1100)

# 0.10.0 - TLS support via FFI + non-blocking verify interaction

* 39c3816 - fix: using `clone` on a double-reference (Ronald Holshausen, Mon Jan 4 17:32:50 2021 +1100)
* 484b747 - fix: verify interaction was blocking the thread (Ronald Holshausen, Mon Jan 4 17:12:38 2021 +1100)
* 4c4eb85 - chore: bump minor version of pact_verifier crate due to breaking changes (Ronald Holshausen, Mon Jan 4 15:48:41 2021 +1100)
* b583540 - Merge branch 'master' into feat/allow-invalid-certs-during-verification (Matt Fellows, Fri Jan 1 14:22:10 2021 +1100)
* 6cec6c7 - feat: allow https scheme and ability to disable ssl verification (Matt Fellows, Thu Dec 31 12:10:57 2020 +1100)
* ed410bd - bump version to 0.9.6 (Ronald Holshausen, Thu Dec 31 15:14:30 2020 +1100)

# 0.9.5 - Supports generators associated with array contains matcher variants

* 144b6aa - chore: upgrade dependencies to latest (Ronald Holshausen, Thu Dec 31 14:58:09 2020 +1100)
* 09513de - feat: add verifiedBy to the verified results (Ronald Holshausen, Tue Dec 29 12:05:07 2020 +1100)
* 12c42c3 - bump version to 0.9.5 (Matt Fellows, Mon Nov 23 07:44:42 2020 +1100)
* 71a5847 - chore: update rust deps (Matt Fellows, Sun Nov 22 23:59:29 2020 +1100)

# 0.9.4 - Bugfix Release

* 52aa549 - chore: improve mismatch output + notices for pacts for verification (Matt Fellows, Sun Nov 22 23:23:15 2020 +1100)
* d481bc1 - fix: pacts for verification unmarshal fails if 'pending' attr is not returned in response (Matt Fellows, Sun Nov 22 22:31:31 2020 +1100)
* 5058a2d - feat: include the mockserver URL and port in the verification context (Ronald Holshausen, Fri Nov 20 16:43:10 2020 +1100)
* a752d6c - bump version to 0.9.4 (Ronald Holshausen, Tue Nov 17 16:58:25 2020 +1100)

# 0.9.3 - Support provider state injected values

* 850282d - fix: times with millisecond precision less 3 caused chronos to panic (Ronald Holshausen, Tue Nov 17 16:29:47 2020 +1100)
* 13ce2f2 - fix: introduce GeneratorTestMode and restrict provider state generator to the provider side (Ronald Holshausen, Mon Nov 16 15:00:01 2020 +1100)

# 0.9.2 - Support Pacts for Verification API

* bbd5364 - test: add negative test case for pacts for verification api (Matt Fellows, Wed Nov 11 08:42:47 2020 +1100)
* b3cca0d - test: add basic pact test for pacts for verification feature (Matt Fellows, Wed Nov 11 00:30:45 2020 +1100)
* e7f729d - wip: further cleanup, and obfuscate auth details (Matt Fellows, Tue Nov 10 13:56:02 2020 +1100)
* ada3667 - wip: cleanup verifier args (Matt Fellows, Tue Nov 10 08:13:01 2020 +1100)
* db0088e - wip: cleanup pacts for verification hal_client clones (Matt Fellows, Mon Nov 9 22:50:51 2020 +1100)
* 80f4e98 - wip: refactor BrokerWithDynamicConfiguration into a struct enum for better readability (Matt Fellows, Mon Nov 9 22:40:24 2020 +1100)
* 93e9161 - wip: working pending pacts with notices (Matt Fellows, Sun Nov 8 14:51:41 2020 +1100)
* 60c1671 - wip: thread verification context into pact fetching/verification, add env vars to clap args (Matt Fellows, Sun Nov 8 13:25:17 2020 +1100)
* 60eb190 - wip: map tags to consumer version selectors (Matt Fellows, Sat Nov 7 23:35:36 2020 +1100)
* 6612a3a - wip: basic wiring in of the pacts for verification endpoint (Matt Fellows, Sat Nov 7 21:39:25 2020 +1100)
* 5e0e470 - chore: bump minor version of pact_consumer crate (Ronald Holshausen, Fri Oct 16 13:22:12 2020 +1100)
* 3a93fd8 - bump version to 0.9.2 (Ronald Holshausen, Fri Oct 16 12:18:50 2020 +1100)

# 0.9.1 - arrayContains matcher + text/xml content type

* 4ef2db6 - Merge branch 'feat/v4-spec' (Ronald Holshausen, Thu Oct 15 17:02:44 2020 +1100)
* 2fb0c6e - fix: fix the build after refactoring the pact write function (Ronald Holshausen, Wed Oct 14 11:07:57 2020 +1100)
* 7fbc731 - chore: bump minor version of matching lib (Ronald Holshausen, Fri Oct 9 10:42:33 2020 +1100)
* 3e943b1 - fix: set content-type header in message request (Marco Dallagiacoma, Thu Oct 1 23:58:14 2020 +0200)
* 29ba743 - feat: add a mock server config struct (Ronald Holshausen, Thu Sep 24 10:30:59 2020 +1000)
* 0b03551 - bump version to 0.9.1 (Ronald Holshausen, Mon Sep 14 17:21:57 2020 +1000)

# 0.9.0 - Verifying Message Pacts

* ef5f88c - chore: bump minor version of the pact_verifier crate (Ronald Holshausen, Mon Sep 14 17:13:45 2020 +1000)
* 865327d - feat: handle comparing content types correctly (Ronald Holshausen, Mon Sep 14 16:37:11 2020 +1000)
* 258cb96 - feat: cleaned up the error display a bit (Ronald Holshausen, Mon Sep 14 16:05:37 2020 +1000)
* ebee1c0 - feat: implemented matching for message metadata (Ronald Holshausen, Mon Sep 14 15:31:18 2020 +1000)
* 6cba6ad - feat: implemented basic message verification with the verifier cli (Ronald Holshausen, Mon Sep 14 13:48:27 2020 +1000)
* 2d44ffd - chore: bump minor version of the matching crate (Ronald Holshausen, Mon Sep 14 12:06:37 2020 +1000)
* fb6c19c - refactor: allow verifier to handle different types of interactions (Ronald Holshausen, Mon Sep 14 10:41:13 2020 +1000)
* 7baf074 - fix: correct clippy error (Ronald Holshausen, Sun Sep 13 18:41:25 2020 +1000)
* 814c416 - refactor: added a trait for interactions, renamed Interaction to RequestResponseInteraction (Ronald Holshausen, Sun Sep 13 17:09:41 2020 +1000)
* a05bcbb - refactor: renamed Pact to RequestResponsePact (Ronald Holshausen, Sun Sep 13 12:45:34 2020 +1000)
* 19290e8 - bump version to 0.8.4 (Ronald Holshausen, Sun Aug 23 16:58:25 2020 +1000)

# 0.8.3 - implemented provider state generator

* b186ce9 - chore: update all dependent crates (Ronald Holshausen, Sun Aug 23 16:49:00 2020 +1000)
* 61ca3d7 - chore: update matching crate to latest (Ronald Holshausen, Sun Aug 23 16:37:58 2020 +1000)
* d5d3679 - feat: return the values from the state change call so they can be used by the generators (Ronald Holshausen, Sun Aug 23 15:40:41 2020 +1000)
* 76f73c6 - feat: implemented provider state generator (Ronald Holshausen, Sun Aug 23 13:29:55 2020 +1000)
* b242eb1 - refactor: changed the remaining uses of the old content type methods (Ronald Holshausen, Sun Jun 28 17:11:51 2020 +1000)
* ed207a7 - chore: updated readmes for docs site (Ronald Holshausen, Sun Jun 28 10:04:09 2020 +1000)
* 8cdcad0 - bump version to 0.8.3 (Ronald Holshausen, Wed Jun 24 11:46:03 2020 +1000)

# 0.8.2 - Updated XML Matching

* 8cf70cc - chore: update to latest matching crate (Ronald Holshausen, Wed Jun 24 11:37:49 2020 +1000)
* a15edea - chore: try set the content type on the body if known (Ronald Holshausen, Tue Jun 23 16:53:32 2020 +1000)
* 875d426 - chore: switch to Rust TLS so we dont have to link to openssl libs (Ronald Holshausen, Sun May 31 09:57:41 2020 +1000)
* df5796f - bump version to 0.8.2 (Ronald Holshausen, Sun May 24 14:02:11 2020 +1000)

# 0.8.1 - Bugfixes + update matching crate to 0.6.0

* bea787c - chore: bump matching crate version to 0.6.0 (Ronald Holshausen, Sat May 23 17:56:04 2020 +1000)
* 61ab50f - fix: date/time matchers fallback to the old key (Ronald Holshausen, Fri May 15 11:27:27 2020 +1000)
* 754a483 - chore: updated itertools to latest (Ronald Holshausen, Wed May 6 15:49:27 2020 +1000)
* 7616ccb - fix: broken tests after handling multiple header values (Ronald Holshausen, Tue May 5 15:45:27 2020 +1000)
* 76250b5 - chore: correct some clippy warnings (Ronald Holshausen, Wed Apr 29 17:53:40 2020 +1000)
* 43de9c3 - chore: update matching library to latest (Ronald Holshausen, Fri Apr 24 10:20:55 2020 +1000)
* c0b67bf - Use err.to_string() rather than format!("{}", err) (Caleb Stepanian, Tue Mar 31 13:27:27 2020 -0400)
* bd10d00 - Avoid deprecated Error::description in favor of Display trait (Caleb Stepanian, Mon Mar 30 16:49:13 2020 -0400)
* c04c0af - bump version to 0.8.1 (Ronald Holshausen, Fri Mar 13 10:06:29 2020 +1100)

# 0.8.0 - Added callback handlers + Bugfixes

* 2920364 - fix: date and time matchers with JSON (Ronald Holshausen, Thu Mar 12 16:07:05 2020 +1100)
* 126b463 - fix: provider state handlers must be synchronous so they are executed for the actual request (Ronald Holshausen, Thu Mar 12 14:16:03 2020 +1100)
* 0e8bfad - fix: allow the HTTP client to be optional in the provider state executor (Ronald Holshausen, Wed Mar 11 14:47:37 2020 +1100)
* 1cf0199 - refactor: moved state change code to a handler (Ronald Holshausen, Wed Mar 11 14:37:07 2020 +1100)
* 70e6648 - chore: converted verifier to use Reqwest (Ronald Holshausen, Mon Mar 9 12:20:14 2020 +1100)
* fe74376 - feat: implemented publishing provider tags with verification results #57 (Ronald Holshausen, Sun Mar 8 18:37:21 2020 +1100)
* b769753 - chore: remove unused import from provider_client (Matt Fellows, Tue Mar 3 12:14:27 2020 +1100)
* c2b7334 - Fixed broken tests using `VerificationOptions`. (Andrew Lilley Brinker, Mon Mar 2 12:16:45 2020 -0800)
* d198d7d - Make `NullRequestFilterExecutor` unconstructable. (Andrew Lilley Brinker, Mon Mar 2 11:59:16 2020 -0800)
* a6e0c16 - Fix RequestFilterExecutor w/ verify_provider (Andrew Lilley Brinker, Mon Mar 2 11:43:59 2020 -0800)
* d944a60 - chore: added callback executors so test code can called during verification (Ronald Holshausen, Sun Feb 23 18:43:49 2020 +1100)
* 639c1fd - bump version to 0.7.1 (Ronald Holshausen, Sun Jan 19 12:03:44 2020 +1100)

# 0.7.0 - Convert to async/await

* 70a33dd - chore: bump minor version of pact_verifier (Ronald Holshausen, Sun Jan 19 11:51:36 2020 +1100)
* 9d3ad57 - chore: bump minor version of pact consumer crate (Ronald Holshausen, Sun Jan 19 11:40:27 2020 +1100)
* cb4c560 - Upgrade tokio to 0.2.9 (Audun Halland, Fri Jan 10 00:13:02 2020 +0100)
* e8034bf - Remove mock server async spawning. (Audun Halland, Thu Jan 9 21:59:56 2020 +0100)
* 9dec41b - Upgrade reqwest to 0.10 (Audun Halland, Tue Dec 31 07:22:36 2019 +0100)
* d24c434 - pact_verifier/pact_broker: Avoid completely unnecessary clones (Audun Halland, Tue Dec 17 02:54:45 2019 +0100)
* cd1046d - pact_verifier: Actually implement HAL client using async reqwest (Audun Halland, Tue Dec 17 01:42:57 2019 +0100)
* d395d2d - pact_verifier: Upgrade reqwest to latest git alpha (Audun Halland, Tue Dec 17 00:57:16 2019 +0100)
* 8019d6d - pact_verifier: Async mock server shutdown (Audun Halland, Thu Dec 12 21:45:16 2019 +0100)
* 3074059 - Refactor ValidatingMockServer into a trait, with two implementations (Audun Halland, Thu Dec 12 15:58:50 2019 +0100)
* fe72f92 - Temporarily solve a problem where a spawned server prevents the test runtime from terminating (Audun Halland, Thu Dec 12 14:14:02 2019 +0100)
* 23a652d - pact_verifier: Implement hyper requests for provider/state change (Audun Halland, Thu Dec 12 11:46:50 2019 +0100)
* 30b1935 - pact_verifier tests: Change to spawned mock server (Audun Halland, Thu Dec 12 11:22:49 2019 +0100)
* bceb44d - pact_verifier: convert pact broker tests to async (Audun Halland, Thu Dec 12 11:04:53 2019 +0100)
* a8866e8 - pact_verifier: Into async/await, part 1 (Audun Halland, Thu Dec 12 10:43:38 2019 +0100)
* 95e46e5 - pact_verifier: Remove extern crate from lib.rs (Audun Halland, Sun Nov 17 23:22:13 2019 +0100)
* 713cd6a - Explicit edition 2018 in Cargo.toml files (Audun Halland, Sat Nov 16 23:55:37 2019 +0100)
* 924452f - 2018 edition autofix "cargo fix --edition" (Audun Halland, Sat Nov 16 22:27:42 2019 +0100)
* d566d23 - bump version to 0.6.2 (Ronald Holshausen, Fri Sep 27 15:17:24 2019 +1000)

# 0.6.1 - Bugfix + Oniguruma crate for regex matching

* 173bf22 - chore: use the matching lib with the Oniguruma crate #46 (Ronald Holshausen, Fri Sep 27 15:02:03 2019 +1000)
* defe890 - fix: switch to the Oniguruma crate for regex matching #46 (Ronald Holshausen, Fri Sep 27 14:35:16 2019 +1000)
* 665bbd8 - fix: return a failure if any pact verification fails #47 (Ronald Holshausen, Fri Sep 27 12:07:01 2019 +1000)
* 48f998d - bump version to 0.6.1 (Ronald Holshausen, Sun Sep 22 17:56:20 2019 +1000)
* 0c5d6c2 - fix: pact_consumer should be a dev dependency (Ronald Holshausen, Sun Sep 22 17:48:35 2019 +1000)

# 0.6.0 - Publishing verification results

* 2e07d77 - chore: set the version of the pact matching crate (Ronald Holshausen, Sun Sep 22 17:24:02 2019 +1000)
* eef3d97 - feat: added some tests for publishing verification results to the pact broker #44 (Ronald Holshausen, Sun Sep 22 16:44:52 2019 +1000)
* 1110b47 - feat: implemented publishing verification results to the pact broker #44 (Ronald Holshausen, Sun Sep 22 13:53:27 2019 +1000)
* cb30a2f - feat: added the ProviderStateGenerator as a generator type (Ronald Holshausen, Sun Sep 8 16:29:46 2019 +1000)
* 1e17ca8 - bump version to 0.5.2 (Ronald Holshausen, Sat Aug 24 12:39:55 2019 +1000)

# 0.5.1 - Use reqwest for better HTTP/S support, support headers with multiple values

* f79b033 - chore: update terminal support in release scripts (Ronald Holshausen, Sat Aug 24 12:25:28 2019 +1000)
* b8019ba - chore: bump the version of the matching lib (Ronald Holshausen, Sat Aug 24 12:22:35 2019 +1000)
* dac8ae1 - feat: support authentication when fetching pacts from a pact broker (Ronald Holshausen, Sun Aug 11 13:57:29 2019 +1000)
* e007763 - feat: support bearer tokens when fetching pacts from URLs (Ronald Holshausen, Sun Aug 11 13:21:17 2019 +1000)
* 4378110 - Merge pull request #42 from audunhalland/reqwest (Ronald Holshausen, Sun Aug 11 09:32:30 2019 +1000)
* 75c9b3a - Fix hal+json matching (Audun Halland, Sat Aug 10 14:30:17 2019 +0200)
* f0c0d07 - feat: support headers with multiple values (Ronald Holshausen, Sat Aug 10 17:01:10 2019 +1000)
* 9310f78 - Error messages are a bit different using reqwest: Fix tests (Audun Halland, Mon Jul 29 01:48:03 2019 +0200)
* 58b8c3c - Remove unused import (Audun Halland, Sun Jul 28 18:34:20 2019 +0200)
* 9fd6458 - Print errors using Display trait (Audun Halland, Sun Jul 28 18:33:47 2019 +0200)
* 19f11f7 - Avoid unnecessary clone (Audun Halland, Sun Jul 28 16:39:12 2019 +0200)
* 8717cdd - Fix for json_content_type with charset (Audun Halland, Sun Jul 28 16:17:37 2019 +0200)
* aa1b714 - Switch pact_broker/HAL client to use reqwest instead of hyper directly (Audun Halland, Sun Jul 28 15:48:31 2019 +0200)
* 8b9648c - bump version to 0.5.1 (Ronald Holshausen, Sat Jul 27 17:29:57 2019 +1000)

# 0.5.0 - Upgrade to non-blocking Hyper 0.12

* 89e58cc - chore: update release script (Ronald Holshausen, Sat Jul 27 17:10:06 2019 +1000)
* d842100 - chore: bump component versions to 0.5.0 (Ronald Holshausen, Sat Jul 27 15:44:51 2019 +1000)
* 47ab6d0 - Upgrade tokio to 0.1.22 everywhere (Audun Halland, Mon Jul 22 23:47:09 2019 +0200)
* 4df2797 - Rename API function again (Audun Halland, Mon Jul 22 23:38:11 2019 +0200)
* 7f7dcb0 - Don't expose tokio Runtime inside the libraries (Audun Halland, Mon Jul 22 02:18:52 2019 +0200)
* 16cc6b6 - Run pact_verifier tests in async mode + pact write lock (Audun Halland, Sun May 12 04:05:08 2019 +0200)
* fd1296f - Use Runtime explicitly in tests (Audun Halland, Thu May 2 23:48:50 2019 +0200)
* e2a544c - Fix another warning (Audun Halland, Thu May 2 22:07:10 2019 +0200)
* f831a3f - Fix a couple of warnings (Audun Halland, Thu May 2 22:06:13 2019 +0200)
* ac1c678 - Don't use tokio runtime in provider_client. Only expose futures. (Audun Halland, Thu May 2 21:58:47 2019 +0200)
* 684c292 - Improve provider client errors (Audun Halland, Thu May 2 21:52:37 2019 +0200)
* b5accd6 - Move a function (Audun Halland, Thu May 2 18:32:35 2019 +0200)
* c4d98cb - Fix all tests (Audun Halland, Thu May 2 17:32:31 2019 +0200)
* 4831483 - A join_urls function (Audun Halland, Thu May 2 10:56:46 2019 +0200)
* 1b443a5 - Remove unused test commits (Audun Halland, Thu May 2 08:05:25 2019 +0200)
* 5d8c6fa - Uncomment and compile all tests (Audun Halland, Thu May 2 01:19:28 2019 +0200)
* 2f8a997 - Compile everything (except the commented-out tests) (Audun Halland, Thu May 2 00:41:56 2019 +0200)
* fb3a859 - Temporary fixes; temporarily comment out some tests until code compiles (Audun Halland, Tue Apr 30 12:52:42 2019 +0200)
* f2ae258 - Convert provider_client to async hyper (Audun Halland, Tue Apr 30 02:21:17 2019 +0200)
* 84f4969 - Add tokio Runtime param to pact_verifier lib (Audun Halland, Sat Apr 27 23:58:38 2019 +0200)
* c060f29 - Fix all compile errors in provider_client.rs (Audun Halland, Sat Apr 27 23:50:43 2019 +0200)
* 61c5481 - Work on making the state change async (Audun Halland, Sat Apr 27 22:02:35 2019 +0200)
* 692577b - More work on futures (Audun Halland, Sat Apr 27 21:53:27 2019 +0200)
* a32ec67 - Hyper 0.12: Work in progress (Audun Halland, Sat Apr 27 18:15:50 2019 +0200)
* f8fa0d8 - chore: Bump pact matchig version to 0.5.0 (Ronald Holshausen, Sat Jan 5 19:25:53 2019 +1100)
* 386ab52 - fix: corrected the release scripts to check for a version parameter (Ronald Holshausen, Sun Apr 8 13:44:57 2018 +1000)
* b5e0666 - bump version to 0.4.1 (Ronald Holshausen, Sat Apr 7 15:02:43 2018 +1000)

# 0.4.0 - First V3 specification release

* f63f339 - replaced use of try macro with ? (Ronald Holshausen, Tue Nov 7 16:31:39 2017 +1100)
* c4d424b - Wired in the generated request/response into the mock server and verifier (Ronald Holshausen, Tue Nov 7 16:27:01 2017 +1100)
* 13558d6 - Basic generators working (Ronald Holshausen, Tue Nov 7 10:56:55 2017 +1100)
* 7fef36b - Merge branch 'v2-spec' into v3-spec (Ronald Holshausen, Sat Nov 4 12:49:07 2017 +1100)
* 5c8b79b - Correct the changelog and linux release script (Ronald Holshausen, Fri Nov 3 15:12:39 2017 +1100)
* 9575ee8 - bump version to 0.3.1 (Ronald Holshausen, Fri Nov 3 15:03:20 2017 +1100)
* fbe35d8 - Compiling after merge from v2-spec (Ronald Holshausen, Sun Oct 22 11:39:46 2017 +1100)
* 00dc75a - Bump version to 0.4.0 (Ronald Holshausen, Sun Oct 22 10:46:48 2017 +1100)
* e82ee08 - Merge branch 'v2-spec' into v3-spec (Ronald Holshausen, Mon Oct 16 09:24:11 2017 +1100)
* 64ff667 - Upgraded the mock server implemenation to use Hyper 0.11.2 (Ronald Holshausen, Wed Sep 6 12:56:47 2017 +1000)
* e5a93f3 - Merge branch 'master' into v3-spec (Ronald Holshausen, Sun Aug 20 09:53:48 2017 +1000)
* fafb23a - update the verifier to support the new V3 format matchers (Ronald Holshausen, Sun Nov 13 16:49:29 2016 +1100)
* 8765729 - Updated the verifier to handle provider state parameters (Ronald Holshausen, Sun Oct 23 12:10:12 2016 +1100)
* 8797c6c - First successful build after merge from master (Ronald Holshausen, Sun Oct 23 11:59:55 2016 +1100)
* 639ac22 - fixes after merge in from master (Ronald Holshausen, Sun Oct 23 10:45:54 2016 +1100)
* 49e45f7 - Merge branch 'master' into v3-spec (Ronald Holshausen, Sun Oct 23 10:10:40 2016 +1100)
* 9d286b0 - add rlib crate type back (Ronald Holshausen, Wed Aug 24 21:13:51 2016 +1000)
* 5a7a65e - Merge branch 'master' into v3-spec (Ronald Holshausen, Wed Aug 24 21:02:23 2016 +1000)
* 539eb48 - updated all the readmes and cargo manefests for v3 (Ronald Holshausen, Tue Jul 19 15:46:18 2016 +1000)

# 0.3.0 - Backported matching rules from V3 branch

* 3c09f5b - Update the dependent modules for the verifier (Ronald Holshausen, Fri Nov 3 14:42:09 2017 +1100)
* 8c50392 - update changelog for release 0.3.0 (Ronald Holshausen, Fri Nov 3 14:27:40 2017 +1100)
* 24e3f73 - Converted OptionalBody::Present to take a Vec<u8> #19 (Ronald Holshausen, Sun Oct 22 18:04:46 2017 +1100)
* d990729 - Some code cleanup #20 (Ronald Holshausen, Wed Oct 18 16:32:37 2017 +1100)
* c983c63 - Bump versions to 0.3.0 (Ronald Holshausen, Wed Oct 18 13:54:46 2017 +1100)
* da9cfda - Implement new, experimental syntax (API BREAKAGE) (Eric Kidd, Sun Oct 8 13:33:09 2017 -0400)
* 06e92e5 - Refer to local libs using version+paths (Eric Kidd, Tue Oct 3 06:22:23 2017 -0400)
* 7afd258 - Update all the cargo manifest versions and commit the cargo lock files (Ronald Holshausen, Wed May 17 10:37:44 2017 +1000)
* 665aea1 - make release script executable (Ronald Holshausen, Wed May 17 10:30:31 2017 +1000)
* 17d6e98 - bump version to 0.2.2 (Ronald Holshausen, Wed May 17 10:23:34 2017 +1000)


# 0.2.1 - Replace rustc_serialize with serde_json

* a1f78f9 - Move linux specific bits out of the release script (Ronald Holshausen, Wed May 17 10:18:37 2017 +1000)
* efe4ca7 - Cleanup unused imports and unreachable pattern warning messages (Anthony Damtsis, Tue May 16 10:31:29 2017 +1000)
* be8c299 - Cleanup unused BTreeMap usages and use remote pact dependencies (Anthony Damtsis, Mon May 15 17:09:14 2017 +1000)
* a59fb98 - Migrate remaining pact modules over to serde (Anthony Damtsis, Mon May 15 16:59:04 2017 +1000)
* 3ca29d6 - bump version to 0.2.1 (Ronald Holshausen, Sun Oct 9 17:06:35 2016 +1100)

# 0.2.0 - V2 specification implementation

* 91f5315 - update the references to the spec in the verifier library to V2 (Ronald Holshausen, Sun Oct 9 16:59:45 2016 +1100)
* e2f88b8 - update the verifier library to use the published consumer library (Ronald Holshausen, Sun Oct 9 16:57:34 2016 +1100)
* 770010a - update projects to use the published pact matching lib (Ronald Holshausen, Sun Oct 9 16:25:15 2016 +1100)
* 574e072 - upadte versions for V2 branch and fix an issue with loading JSON bodies encoded as a string (Ronald Holshausen, Sun Oct 9 15:31:57 2016 +1100)
* dabe425 - bump version to 0.1.1 (Ronald Holshausen, Sun Oct 9 10:40:39 2016 +1100)

# 0.1.0 - V1.1 specification implementation

* 7b66941 - Update the deps for pact verifier library (Ronald Holshausen, Sun Oct 9 10:32:47 2016 +1100)
* 1f3f3f1 - correct the versions of the inter-dependent projects as they were causing the build to fail (Ronald Holshausen, Sat Oct 8 17:41:57 2016 +1100)
* a46dabb - update all references to V1 spec after merge (Ronald Holshausen, Sat Oct 8 16:20:51 2016 +1100)
* 1246784 - correct the verifier library release script (Ronald Holshausen, Tue Sep 27 20:57:13 2016 +1000)
* f0ce08a - bump version to 0.0.1 (Ronald Holshausen, Tue Sep 27 20:43:34 2016 +1000)

# 0.0.0 - First Release
