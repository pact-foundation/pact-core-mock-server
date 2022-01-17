To generate the log, run `git log --pretty='* %h - %s (%an, %ad)' TAGNAME..HEAD .` replacing TAGNAME and HEAD as appropriate.

# 0.8.6 - Bugfix Release

* 5e4c68ef - chore: update pact matching to 0.12.2 (Ronald Holshausen, Mon Jan 17 16:29:21 2022 +1100)
* 80b241c5 - chore: Upgrade plugin driver crate to 0.0.17 (Ronald Holshausen, Mon Jan 17 11:22:48 2022 +1100)
* 4f1ecff2 - chore: Upgrade pact-models to 0.2.7 (Ronald Holshausen, Mon Jan 17 10:53:26 2022 +1100)
* c2089645 - fix: log crate version must be fixed across all crates (including plugin driver) (Ronald Holshausen, Fri Jan 14 16:10:50 2022 +1100)
* b33ce2fa - bump version to 0.8.6 (Ronald Holshausen, Tue Jan 4 12:43:22 2022 +1100)

# 0.8.5 - Maintenance Release

* 8259c12a - chore: update pact_models 0.2.6, pact_matching 0.12.1, pact-plugin-driver 0.0.16 (Ronald Holshausen, Tue Jan 4 12:38:26 2022 +1100)
* 9c2810ad - chore: Upgrade pact-plugin-driver to 0.0.15 (Ronald Holshausen, Fri Dec 31 15:12:56 2021 +1100)
* 0a6e7d9d - refactor: Convert MatchingContext to a trait and use DocPath instead of string slices (Ronald Holshausen, Wed Dec 29 14:24:39 2021 +1100)
* d8332686 - bump version to 0.8.5 (Ronald Holshausen, Thu Dec 23 13:19:14 2021 +1100)

# 0.8.4 - Maintenance Release

* 52bc1735 - chore: update pact_matching crate to 0.11.5 (Ronald Holshausen, Thu Dec 23 13:12:08 2021 +1100)
* 5479a634 - chore: Update pact_models (0.2.4) and pact-plugin-driver (0.0.14) (Ronald Holshausen, Thu Dec 23 12:57:02 2021 +1100)
* fc0a8360 - chore: update pact_matching to 0.11.4 (Ronald Holshausen, Mon Dec 20 12:19:36 2021 +1100)
* 8911d5b0 - chore: update to latest plugin driver crate (metrics fixes) (Ronald Holshausen, Mon Dec 20 12:11:35 2021 +1100)
* 71708291 - bump version to 0.8.4 (Ronald Holshausen, Wed Dec 15 13:20:28 2021 +1100)

# 0.8.3 - Maintenance Release

* 84355d3d - chore: Upgrade rustls in the mock server to 0.20.2 (Ronald Holshausen, Wed Dec 15 12:32:28 2021 +1100)
* 4f1ba7d9 - chore: update to the latest plugin driver (Ronald Holshausen, Tue Dec 14 13:55:02 2021 +1100)
* e21879f7 - bump version to 0.8.3 (Ronald Holshausen, Wed Nov 17 15:08:23 2021 +1100)

# 0.8.2 - Support setting pact spec version on the mock servers

* 9cfe897a - feat(mock server): default pact spec to V3 if unknown (Ronald Holshausen, Wed Nov 17 14:53:19 2021 +1100)
* 5d4a09c6 - feat: store the pact specification version with the mock server (Ronald Holshausen, Wed Nov 17 14:46:56 2021 +1100)
* fc5be202 - fix: update to latest driver crate (Ronald Holshausen, Tue Nov 16 16:19:02 2021 +1100)
* 33891ccb - bump version to 0.8.2 (Ronald Holshausen, Tue Nov 16 12:17:47 2021 +1100)

# 0.8.1 - Support for using plugins via FFI

* 5d974c4a - chore: update to latest models and plugin driver crates (Ronald Holshausen, Tue Nov 16 11:56:53 2021 +1100)
* 2027537d - refactor: update FFI to use V4 models internally (Ronald Holshausen, Mon Nov 8 16:44:39 2021 +1100)
* 400a1231 - chore: drop beta from pact_verifier version (Ronald Holshausen, Thu Nov 4 15:56:22 2021 +1100)
* 01dbf7b5 - bump version to 0.8.1 (Ronald Holshausen, Thu Nov 4 15:36:31 2021 +1100)

# 0.8.0 - Pact V4 release

* fc4580b8 - chore: drop beta from pact_mock_server version (Ronald Holshausen, Thu Nov 4 15:28:51 2021 +1100)
* bd2bd0ec - chore: drop beta from pact_matching version (Ronald Holshausen, Wed Nov 3 13:28:35 2021 +1100)
* 296b4370 - chore: update project to Rust 2021 edition (Ronald Holshausen, Fri Oct 22 10:44:48 2021 +1100)
* a561f883 - chore: use the non-beta models crate (Ronald Holshausen, Thu Oct 21 18:10:27 2021 +1100)
* 5532c730 - bump version to 0.8.0-beta.5 (Ronald Holshausen, Tue Oct 19 17:26:58 2021 +1100)

# 0.8.0-beta.4 - Bugfix Release

* 918e5beb - fix: update to latest models and plugin driver crates (Ronald Holshausen, Tue Oct 19 17:09:48 2021 +1100)
* 3819522d - chore: update to the latest matching and mock server crates (Ronald Holshausen, Tue Oct 19 11:34:18 2021 +1100)
* ece992af - bump version to 0.8.0-beta.4 (Ronald Holshausen, Tue Oct 19 11:27:07 2021 +1100)

# 0.8.0-beta.3 - Support matching synchronous request/response messages

* aa434ba3 - chore: update to latest driver crate (Ronald Holshausen, Tue Oct 19 11:09:46 2021 +1100)
* df386c8a - chore: use the published version of pact-plugin-driver (Ronald Holshausen, Mon Oct 18 13:41:36 2021 +1100)
* 2b4b7cc3 - feat(plugins): Support matching synchronous request/response messages (Ronald Holshausen, Fri Oct 15 16:01:50 2021 +1100)
* d69b0617 - bump version to 0.8.0-beta.3 (Ronald Holshausen, Tue Oct 12 16:31:49 2021 +1100)

# 0.8.0-beta.2 - Support synchronous messages

* 9bbbb52e - chore: bump pact matching crate version (Ronald Holshausen, Tue Oct 12 16:24:01 2021 +1100)
* b7018002 - Revert "update changelog for release 0.8.0-beta.2" (Ronald Holshausen, Tue Oct 12 16:09:49 2021 +1100)
* 9319f650 - update changelog for release 0.8.0-beta.2 (Ronald Holshausen, Tue Oct 12 16:05:18 2021 +1100)
* 3dbd609a - chore: bump version of pact-plugin-driver (Ronald Holshausen, Tue Oct 12 15:59:53 2021 +1100)
* d0bfb8a8 - feat: Support consumer tests with synchronous messages (Ronald Holshausen, Tue Oct 12 15:51:08 2021 +1100)
* 35ff0993 - feat: record the version of the lib that created the pact in the metadata (Ronald Holshausen, Tue Oct 12 14:52:43 2021 +1100)
* 3df879d4 - bump version to 0.8.0-beta.2 (Ronald Holshausen, Wed Oct 6 12:30:24 2021 +1100)

# 0.8.0-beta.1 - Fixes from master + Plugin support (driver version 0.0.3)

* dfabfac0 - chore: use the published version of the models crate (Ronald Holshausen, Wed Oct 6 12:24:04 2021 +1100)
* 2c47023c - chore: pin plugin driver version to 0.0.3 (Ronald Holshausen, Wed Oct 6 11:21:07 2021 +1100)
* 288e2168 - chore: use the published version of the plugin driver lib (Ronald Holshausen, Tue Oct 5 15:36:06 2021 +1100)
* 6d23796f - feat(plugins): support each key and each value matchers (Ronald Holshausen, Wed Sep 29 11:10:46 2021 +1000)
* 6f20282d - Merge branch 'master' into feat/plugins (Ronald Holshausen, Tue Sep 28 14:51:34 2021 +1000)
* 54615e1b - bump version to 0.7.22 (Ronald Holshausen, Tue Sep 28 13:41:40 2021 +1000)
* 97c8de3c - update changelog for release 0.7.21 (Ronald Holshausen, Tue Sep 28 13:39:32 2021 +1000)
* df715cd5 - feat: support native TLS. Fixes #144 (Matt Fellows, Mon Sep 20 13:00:33 2021 +1000)
* ee3212a8 - refactor(plugins): do not expose the catalogue statics, but rather a function to initialise it (Ronald Holshausen, Tue Sep 14 15:13:12 2021 +1000)
* b71dcabf - refactor(plugins): rename ContentTypeOverride -> ContentTypeHint (Ronald Holshausen, Tue Sep 14 15:08:52 2021 +1000)
* e63ade0d - bump version to 0.8.0-beta.1 (Ronald Holshausen, Mon Sep 13 11:53:04 2021 +1000)

# 0.7.21 - support native TLS certs

* df715cd5 - feat: support native TLS. Fixes #144 (Matt Fellows, Mon Sep 20 13:00:33 2021 +1000)
* c9165bd3 - bump version to 0.7.21 (Ronald Holshausen, Tue Aug 17 10:37:33 2021 +1000)

# 0.8.0-beta.0 - Support for plugins with mock server

* fd6f8f40 - chore: Bump pact_mock_server version to 0.8.0-beta.0 (Ronald Holshausen, Mon Sep 13 11:46:11 2021 +1000)
* 716809f6 - chore: Get CI build passing (Ronald Holshausen, Fri Sep 10 14:55:46 2021 +1000)
* 4aaaafd8 - feat(plugins): Support non-blocking mock server in consumer tests + shutting down plugins when mock servers shutdown (Ronald Holshausen, Fri Sep 10 13:20:01 2021 +1000)
* b77498c8 - chore: fix tests after updating plugin API (Ronald Holshausen, Fri Sep 3 16:48:18 2021 +1000)
* e8ae81b3 - refactor: matching req/res with plugins requires data from the pact and interaction (Ronald Holshausen, Thu Sep 2 11:57:50 2021 +1000)
* b9aa7ecb - feat(Plugins): allow plugins to override text/binary format of the interaction content (Ronald Holshausen, Mon Aug 30 10:48:04 2021 +1000)
* eb34b011 - chore: use the published version of pact-plugin-driver (Ronald Holshausen, Mon Aug 23 15:48:55 2021 +1000)
* 0c5cede2 - chore: bump models crate to 0.2 (Ronald Holshausen, Mon Aug 23 12:56:14 2021 +1000)
* e3a2660f - chore: fix tests after updating test builders to be async (Ronald Holshausen, Fri Aug 20 12:41:10 2021 +1000)
* 779f099c - feat(plugins): Got generators from plugin working (Ronald Holshausen, Thu Aug 19 17:20:47 2021 +1000)
* b75fea5d - Merge branch 'master' into feat/plugins (Ronald Holshausen, Wed Aug 18 12:27:41 2021 +1000)
* c9165bd3 - bump version to 0.7.21 (Ronald Holshausen, Tue Aug 17 10:37:33 2021 +1000)
* 2662241e - feat(plugins): Call out to plugins when comparing content owned by the plugin during verification (Ronald Holshausen, Fri Aug 13 14:29:30 2021 +1000)
* 60869969 - feat(plugins): Add core features to the plugin catalogue (Ronald Holshausen, Thu Aug 12 13:00:41 2021 +1000)

# 0.7.20 - Refactor

* 9baa714d - chore: bump minor version of matching crate (Ronald Holshausen, Fri Jul 23 14:03:20 2021 +1000)
* 533c9e1f - chore: bump minor version of the Pact models crate (Ronald Holshausen, Fri Jul 23 13:15:32 2021 +1000)
* 3dccf866 - refacfor: moved the pact structs to the models crate (Ronald Holshausen, Sun Jul 18 16:58:14 2021 +1000)
* e8046d84 - refactor: moved interaction structs to the models crate (Ronald Holshausen, Sun Jul 18 14:36:03 2021 +1000)
* cf00f528 - bump version to 0.7.20 (Ronald Holshausen, Sun Jul 11 17:11:29 2021 +1000)

# 0.7.19 - Refactor: Moved structs to models crate

* e2e10241 - refactor: moved Request and Response structs to the models crate (Ronald Holshausen, Wed Jul 7 18:09:36 2021 +1000)
* 9e8b01d7 - refactor: move HttpPart struct to models crate (Ronald Holshausen, Wed Jul 7 15:59:34 2021 +1000)
* 01ff9877 - refactor: moved matching rules and generators to models crate (Ronald Holshausen, Sun Jul 4 17:17:30 2021 +1000)
* c3c22ea8 - Revert "refactor: moved matching rules and generators to models crate (part 1)" (Ronald Holshausen, Wed Jun 23 14:37:46 2021 +1000)
* 53bb86c4 - Merge branch 'release-verifier' (Ronald Holshausen, Wed Jun 23 13:59:59 2021 +1000)
* 3a02d1eb - bump version to 0.7.19 (Ronald Holshausen, Wed Jun 23 13:25:12 2021 +1000)
* d3406650 - refactor: moved matching rules and generators to models crate (part 1) (Ronald Holshausen, Wed Jun 23 12:58:30 2021 +1000)

# 0.7.18 - accumulating log entries + bugfix

* b4e26844 - fix: reqwest is dyn linked to openssl by default, which causes a SIGSEGV on alpine linux (Ronald Holshausen, Tue Jun 1 14:21:31 2021 +1000)
* 68f8f84e - chore: skip failing tests in alpine to get the build going (Ronald Holshausen, Tue Jun 1 13:47:20 2021 +1000)
* 17beef62 - feat: support accumulating log entries per running mock server (Ronald Holshausen, Mon May 31 15:09:20 2021 +1000)
* 0fc54642 - bump version to 0.7.18 (Ronald Holshausen, Sun May 30 17:36:27 2021 +1000)

# 0.7.17 - V4 features + bugfixes/enhancements

* 735c9e7 - chore: bump pact_matching to 0.9 (Ronald Holshausen, Sun Apr 25 13:50:18 2021 +1000)
* fb373b4 - chore: bump version to 0.0.2 (Ronald Holshausen, Sun Apr 25 13:40:52 2021 +1000)
* d010630 - chore: cleanup deprecation and compiler warnings (Ronald Holshausen, Sun Apr 25 12:23:30 2021 +1000)
* 3dd610a - refactor: move structs and code dealing with bodies to a seperate package (Ronald Holshausen, Sun Apr 25 11:20:47 2021 +1000)
* a725ab1 - feat(V4): added synchronous request/response message formats (Ronald Holshausen, Sat Apr 24 16:05:12 2021 +1000)
* e588bb2 - fix: clippy violation: using `clone` on a double-reference (Ronald Holshausen, Sat Apr 24 12:52:58 2021 +1000)
* 80b7148 - feat(V4): Updated consumer DSL to set comments + mock server initial support for V4 pacts (Ronald Holshausen, Fri Apr 23 17:58:10 2021 +1000)
* 4bcd94f - refactor: moved OptionalBody and content types to pact models crate (Ronald Holshausen, Thu Apr 22 14:01:56 2021 +1000)
* 220fb5e - refactor: move the PactSpecification enum to the pact_models crate (Ronald Holshausen, Thu Apr 22 11:18:26 2021 +1000)
* a0f6a1d - refactor: Use Anyhow instead of `io::Result` (Caleb Stepanian, Wed Apr 7 16:17:35 2021 -0400)
* 97ac20d - bump version to 0.7.17 (Ronald Holshausen, Sun Mar 14 14:41:36 2021 +1100)

# 0.7.16 - Bugfix Release

* 5a529fd - feat: add ability of mock server to expose metrics #94 (Ronald Holshausen, Sun Mar 14 11:41:16 2021 +1100)
* e81482e - chore: correct import (Ronald Holshausen, Tue Feb 9 16:27:38 2021 +1100)
* b23c845 - chore: cleanup some debug logging (Ronald Holshausen, Tue Feb 9 16:18:34 2021 +1100)
* 7f054e8 - fix: correctly assemble UTF-8 percent encoded query parameters (Ronald Holshausen, Tue Feb 9 14:02:04 2021 +1100)
* 2002b67 - bump version to 0.7.16 (Ronald Holshausen, Mon Feb 8 15:44:20 2021 +1100)

# 0.7.15 - use a file system lock when merging pact files

* eae1b16 - chore: fix clippy errors (Ronald Holshausen, Mon Feb 8 14:57:42 2021 +1100)
* 9976e80 - feat: added read locks and a mutex guard to reading and writing pacts (Ronald Holshausen, Mon Feb 8 11:58:52 2021 +1100)
* 61e16ed - feat: use a file system lock when merging pact files (Ronald Holshausen, Sun Feb 7 17:00:29 2021 +1100)
* 49a3cf2 - refactor: use bytes crate instead of vector of bytes for body content (Ronald Holshausen, Sun Feb 7 14:43:40 2021 +1100)
* e43fdb8 - chore: upgrade maplit, itertools (Audun Halland, Mon Jan 11 05:30:10 2021 +0100)
* 45ae48e - bump version to 0.7.15 (Ronald Holshausen, Mon Jan 11 10:05:50 2021 +1100)

# 0.7.14 - Updated dependencies

* 4a70bef - chore: upgrade expectest to 0.12 (Audun Halland, Sat Jan 9 11:29:29 2021 +0100)
* 1ac3548 - chore: upgrade env_logger to 0.8 (Audun Halland, Sat Jan 9 09:50:27 2021 +0100)
* 9a8a63f - chore: upgrade quickcheck (Audun Halland, Sat Jan 9 08:46:51 2021 +0100)
* 3a6945e - chore: Upgrade reqwest to 0.11 and hence tokio to 1.0 (Ronald Holshausen, Wed Jan 6 15:34:47 2021 +1100)
* 598352b - bump version to 0.7.14 (Ronald Holshausen, Tue Jan 5 13:04:25 2021 +1100)

# 0.7.13 - Upgrade Tokio to 1.0

* ef76f38 - chore: cleanup compiler warnings (Ronald Holshausen, Tue Jan 5 10:10:39 2021 +1100)
* 4636982 - chore: update other crates to use Tokio 1.0 (Ronald Holshausen, Mon Jan 4 17:26:59 2021 +1100)
* 211a4fc - chore: got code compiling, which is 90% of the battle (Ronald Holshausen, Mon Jan 4 10:45:27 2021 +1100)
* 62454d5 - chore: upgrade tokio and hyper (Ronald Holshausen, Sun Jan 3 12:22:38 2021 +1100)
* 5dddde1 - bump version to 0.7.13 (Ronald Holshausen, Thu Dec 31 12:40:46 2020 +1100)

# 0.7.12 - Mockserver URL and array contains generators

* f2086d8 - chore: cleanup warnings (Ronald Holshausen, Tue Dec 29 15:46:46 2020 +1100)
* 528c9b5 - chore: skip test that fails intermittently on Windows (Ronald Holshausen, Tue Dec 29 14:52:38 2020 +1100)
* 6491ce9 - chore: add longer sleep for test failing on windows (Ronald Holshausen, Tue Dec 29 14:38:45 2020 +1100)
* 5e56ecb - refactor: support generators associated with array contains matcher variants (Ronald Holshausen, Tue Dec 29 11:46:56 2020 +1100)
* 5058a2d - feat: include the mockserver URL and port in the verification context (Ronald Holshausen, Fri Nov 20 16:43:10 2020 +1100)
* 118daa1 - feat: when merging pact files, upcast to the higher spec version (Ronald Holshausen, Thu Nov 19 18:09:13 2020 +1100)
* 08852e4 - bump version to 0.7.12 (Ronald Holshausen, Tue Nov 17 16:45:35 2020 +1100)

# 0.7.11 - Support provider state injected values

* 13ce2f2 - fix: introduce GeneratorTestMode and restrict provider state generator to the provider side (Ronald Holshausen, Mon Nov 16 15:00:01 2020 +1100)
* 0a10c7c - bump version to 0.7.11 (Ronald Holshausen, Fri Oct 30 12:18:53 2020 +1100)

# 0.7.10 - Bugfix Release

* 326d02d - fix: jsdom does not support access-control-allow-headers: * for CORS pre-flight responses (Ronald Holshausen, Fri Oct 30 11:54:03 2020 +1100)
* a732c41 - bump version to 0.7.10 (Ronald Holshausen, Fri Oct 16 11:25:56 2020 +1100)

# 0.7.9 - arrayContains matcher

* 2fb0c6e - fix: fix the build after refactoring the pact write function (Ronald Holshausen, Wed Oct 14 11:07:57 2020 +1100)
* f334a4f - refactor: introduce a MatchingContext into all matching functions + delgate to matchers for collections (Ronald Holshausen, Mon Oct 12 14:06:00 2020 +1100)
* 7fbc731 - chore: bump minor version of matching lib (Ronald Holshausen, Fri Oct 9 10:42:33 2020 +1100)
* 172f505 - chore: cleaned up some compiler warnings (Ronald Holshausen, Thu Oct 8 15:02:49 2020 +1100)
* facc898 - refactor: moved the shutdown code to a method in mock server crate (Ronald Holshausen, Sun Oct 4 11:56:10 2020 +1100)
* 44e7414 - fix: access-control-allow-methods header was duplicated (Ronald Holshausen, Thu Oct 1 15:29:14 2020 +1000)
* d3c5cf2 - feat: add all the CORS headers (Ronald Holshausen, Wed Sep 30 13:19:31 2020 +1000)
* 584aa08 - bump version to 0.7.9 (Ronald Holshausen, Mon Sep 28 12:06:28 2020 +1000)

# 0.7.8 - CORS pre-flight requests

* 7e68e4c - feat: enable CORS behaviour based on the mock server config (Ronald Holshausen, Mon Sep 28 11:42:23 2020 +1000)
* bdbfccc - refactor: update mock server CLI to be async (Ronald Holshausen, Sun Sep 27 13:12:51 2020 +1000)
* 29ba743 - feat: add a mock server config struct (Ronald Holshausen, Thu Sep 24 10:30:59 2020 +1000)
* 2e662a6 - feat: handle CORS pre-flight requests in the mock server (Ronald Holshausen, Wed Sep 23 17:59:32 2020 +1000)
* 2676b51 - bump version to 0.7.8 (Ronald Holshausen, Mon Sep 14 16:55:22 2020 +1000)

# 0.7.7 - Updated to latest pact matching crate

* 6cba6ad - feat: implemented basic message verification with the verifier cli (Ronald Holshausen, Mon Sep 14 13:48:27 2020 +1000)
* 2d44ffd - chore: bump minor version of the matching crate (Ronald Holshausen, Mon Sep 14 12:06:37 2020 +1000)
* 814c416 - refactor: added a trait for interactions, renamed Interaction to RequestResponseInteraction (Ronald Holshausen, Sun Sep 13 17:09:41 2020 +1000)
* a05bcbb - refactor: renamed Pact to RequestResponsePact (Ronald Holshausen, Sun Sep 13 12:45:34 2020 +1000)
* 1682eda - bump version to 0.7.7 (Ronald Holshausen, Sun Aug 23 14:47:38 2020 +1000)

# 0.7.6 - Implemented provider state generator

* e9955c4 - chore: update to latest matching crate (Ronald Holshausen, Sun Aug 23 14:41:42 2020 +1000)
* 8499b7d - chore: fix link in readme #72 (Ronald Holshausen, Sat Aug 22 15:38:08 2020 +1000)
* da53bac - fix: return the most relevant response from the mock server #69 (Ronald Holshausen, Tue Jul 21 16:10:54 2020 +1000)
* 420f5e2 - Merge pull request #70 from pact-foundation/fix/v2-pacts (Ronald Holshausen, Tue Jul 21 09:46:05 2020 +1000)
* d7632cb - fix: write_pact_file was always serialising a v3 pact even if the spec version was set to 2 (Matt Fellows, Tue Jul 21 09:42:30 2020 +1000)
* b242eb1 - refactor: changed the remaining uses of the old content type methods (Ronald Holshausen, Sun Jun 28 17:11:51 2020 +1000)
* ed207a7 - chore: updated readmes for docs site (Ronald Holshausen, Sun Jun 28 10:04:09 2020 +1000)
* 3d44484 - bump version to 0.7.6 (Ronald Holshausen, Wed Jun 24 10:48:13 2020 +1000)
* f123357 - chore: bump to latest matching crate (Ronald Holshausen, Wed Jun 24 10:43:01 2020 +1000)

# 0.7.5 - Updated XML Matching

* a15edea - chore: try set the content type on the body if known (Ronald Holshausen, Tue Jun 23 16:53:32 2020 +1000)
* b4fe61f - bump version to 0.7.5 (Ronald Holshausen, Sun May 24 11:56:22 2020 +1000)

# 0.7.4 - multi-part form post bodies

* ce94df9 - feat: cleaned up the logging of request matches (Ronald Holshausen, Sun May 24 11:17:08 2020 +1000)
* bea787c - chore: bump matching crate version to 0.6.0 (Ronald Holshausen, Sat May 23 17:56:04 2020 +1000)
* 2d11c17 - chore: set version of patch matching crate to 0.5.14 (Ronald Holshausen, Fri May 15 16:33:21 2020 +1000)
* a4b2a6a - bump version to 0.7.4 (Ronald Holshausen, Tue May 12 12:40:50 2020 +1000)

# 0.7.3 - matching of binary payloads

* 4a28e7c - chore: add debug log entry when request does not match (Ronald Holshausen, Tue May 12 11:59:02 2020 +1000)
* 708db47 - feat: implement matching of binary payloads (application/octet-stream) (Ronald Holshausen, Fri May 8 15:52:03 2020 +1000)
* 754a483 - chore: updated itertools to latest (Ronald Holshausen, Wed May 6 15:49:27 2020 +1000)
* 215eb67 - bump version to 0.7.3 (Ronald Holshausen, Tue May 5 16:53:48 2020 +1000)

# 0.7.2 - TLS suppport + bugfixes

* d85f28c - fix: mock server matching requests with headers with multiple values (Ronald Holshausen, Tue May 5 15:23:11 2020 +1000)
* da885a3 - feat: add support for TLS with the mock server #65 (Ronald Holshausen, Thu Apr 30 16:41:30 2020 +1000)
* 34103aa - bump version to 0.7.2 (Ronald Holshausen, Fri Apr 24 10:45:24 2020 +1000)

# 0.7.1 - Changes to support C++ DSL

* 43de9c3 - chore: update matching library to latest (Ronald Holshausen, Fri Apr 24 10:20:55 2020 +1000)
* 5f8d0a0 - feat: handle bodies with embedded matchers and generators (Ronald Holshausen, Thu Apr 23 12:25:05 2020 +1000)
* 734723d - chore: increase timeout for test on Appveyor (Ronald Holshausen, Fri Apr 17 09:06:55 2020 +1000)
* a0b2c7b - chore: add a wait for test on Appveyor (Ronald Holshausen, Thu Apr 16 14:42:46 2020 +1000)
* 7e89ca9 - chore: update matching crate to latest (Ronald Holshausen, Thu Apr 16 14:06:02 2020 +1000)
* 9ff6f20 - chore: cleaned up some debug logging (Ronald Holshausen, Tue Apr 7 12:10:12 2020 +1000)
* f9b690e - bump version to 0.7.1 (Ronald Holshausen, Sun Jan 19 11:30:42 2020 +1100)

# 0.7.0 - Convert to async/await

* cf452f5 - chore: bump minor version (Ronald Holshausen, Sun Jan 19 11:18:03 2020 +1100)
* 2b85b71 - chore: dump pact matching crate to 0.5.8 (Ronald Holshausen, Sun Jan 19 11:15:07 2020 +1100)
* cb4c560 - Upgrade tokio to 0.2.9 (Audun Halland, Fri Jan 10 00:13:02 2020 +0100)
* 3dec6ff - Upgrade tokio to 0.2.6 (Audun Halland, Tue Dec 31 07:40:14 2019 +0100)
* 6747a98 - pact_mock_server: Try to fix windows test fail by awaiting the server shutdown (Audun Halland, Thu Dec 19 23:39:07 2019 +0100)
* fda11e4 - Merge remote-tracking branch 'upstream/master' into async-await (Audun Halland, Tue Dec 17 02:13:58 2019 +0100)
* 65a4452 - chore: set min matching lib version to 0.5.7 (Ronald Holshausen, Sat Dec 14 17:09:03 2019 +1100)
* b6dda08 - bump version to 0.6.3 (Ronald Holshausen, Sat Dec 14 17:07:50 2019 +1100)
* 23a652d - pact_verifier: Implement hyper requests for provider/state change (Audun Halland, Thu Dec 12 11:46:50 2019 +0100)
* 6a43f82 - Cut down tokio features to the bone (Audun Halland, Wed Dec 11 22:15:03 2019 +0100)
* 353cb5b - pact_mock_server: Use std future trait instead of futures-rs (Audun Halland, Wed Dec 11 21:56:05 2019 +0100)
* 2136306 - pact_mock_server: Pass all tests (Audun Halland, Wed Dec 11 01:08:24 2019 +0100)
* 6699bc8 - pact_mock_server: Make it all compile with async/await (Audun Halland, Wed Dec 11 00:32:49 2019 +0100)
* 42f72f2 - mock_server: Convert hyper_server to async await. Use 4 space indent (Audun Halland, Wed Dec 11 00:13:15 2019 +0100)

# 0.6.2 - Rust 2018 edition

* 8bfeb0b - pact_mock_server: Remove extern crate from lib.rs (Audun Halland, Sun Nov 17 22:53:52 2019 +0100)
* 713cd6a - Explicit edition 2018 in Cargo.toml files (Audun Halland, Sat Nov 16 23:55:37 2019 +0100)
* 924452f - 2018 edition autofix "cargo fix --edition" (Audun Halland, Sat Nov 16 22:27:42 2019 +0100)
* 99fdde2 - bump version to 0.6.2 (Ronald Holshausen, Sat Sep 28 14:19:43 2019 +1000)

# 0.6.1 - Bugfix Release

* 37d89dd - chore: use the latest matching lib (Ronald Holshausen, Sat Sep 28 14:04:55 2019 +1000)
* eef3d97 - feat: added some tests for publishing verification results to the pact broker #44 (Ronald Holshausen, Sun Sep 22 16:44:52 2019 +1000)
* 1110b47 - feat: implemented publishing verification results to the pact broker #44 (Ronald Holshausen, Sun Sep 22 13:53:27 2019 +1000)
* 2488ab9 - Merge branch 'master' of https://github.com/pact-foundation/pact-reference (milleniumbug, Wed Sep 18 11:32:03 2019 +0200)
* cb30a2f - feat: added the ProviderStateGenerator as a generator type (Ronald Holshausen, Sun Sep 8 16:29:46 2019 +1000)
* bdcf655 - bump version to 0.6.1 (Ronald Holshausen, Sat Sep 7 12:29:06 2019 +1000)
* adf1a97 - fix: correct the release script (Ronald Holshausen, Sat Sep 7 12:28:22 2019 +1000)
* b48ee72 - Provide public API for passing in a listener address and post (milleniumbug, Thu Sep 5 15:20:37 2019 +0200)

# 0.6.0 - moved the ffi functions into the ffi module

* e4355d5 - refactor: removed the ffi suffix from the exported functions (Ronald Holshausen, Sat Sep 7 10:36:19 2019 +1000)
* 9abde6c - refactor: moved the ffi functions into the ffi module (Ronald Holshausen, Sat Sep 7 10:16:54 2019 +1000)
* 097d045 - refactor: added a mock server ffi module and bumped the mock server minor version (Ronald Holshausen, Sat Sep 7 09:39:27 2019 +1000)
* 3adf21d - bump version to 0.5.2 (Ronald Holshausen, Sun Aug 11 15:03:24 2019 +1000)

# 0.5.1 - support headers with multiple values

* 1971e2a - chore: remove the p-macro crate (Ronald Holshausen, Sun Aug 11 14:51:24 2019 +1000)
* 63c180f - chore: set the version of the matching lib top 0.5.3 (Ronald Holshausen, Sun Aug 11 14:48:03 2019 +1000)
* b5c7842 - fix: corrected some spelling (Ronald Holshausen, Sun Aug 11 14:31:42 2019 +1000)
* 152682e - chore: cleanup crates and warnings (Ronald Holshausen, Sun Aug 11 14:28:02 2019 +1000)
* f0c0d07 - feat: support headers with multiple values (Ronald Holshausen, Sat Aug 10 17:01:10 2019 +1000)
* 2057f2c - fix: correct the release scripts (Ronald Holshausen, Sat Jul 27 16:07:13 2019 +1000)
* ba7f7e1 - bump version to 0.5.1 (Ronald Holshausen, Sat Jul 27 15:59:52 2019 +1000)

# 0.5.0 - Upgrade to non-blocking Hyper 0.12

* d842100 - chore: bump component versions to 0.5.0 (Ronald Holshausen, Sat Jul 27 15:44:51 2019 +1000)
* a7c674a - fix: remove duplicated line (Ronald Holshausen, Sat Jul 27 15:41:00 2019 +1000)
* ee8a898 - Rewrite server matches sync from mpsc queue to Arc<Mutex<Vec>>. Avoids awkward synchronization (Audun Halland, Tue Jul 23 02:10:55 2019 +0200)
* 5ea7815 - Merge remote-tracking branch 'upstream/master' into hyper_upgrade_merge (Audun Halland, Tue Jul 23 01:46:51 2019 +0200)
* 2826bb0 - Make pact_mock_server_cli use ServerManager (Audun Halland, Tue Jul 23 01:40:46 2019 +0200)
* 47ab6d0 - Upgrade tokio to 0.1.22 everywhere (Audun Halland, Mon Jul 22 23:47:09 2019 +0200)
* 4df2797 - Rename API function again (Audun Halland, Mon Jul 22 23:38:11 2019 +0200)
* 7f7dcb0 - Don't expose tokio Runtime inside the libraries (Audun Halland, Mon Jul 22 02:18:52 2019 +0200)
* 2230be9 - bump version to 0.4.2 (Ronald Holshausen, Sun Jun 30 16:23:22 2019 +1000)
* 0223d31 - Remove warning about unused macros in production code (Audun Halland, Sun May 12 10:57:35 2019 +0200)
* 0e83d41 - Comment the use of PACT_FILE_MUTEX (Audun Halland, Sun May 12 10:55:55 2019 +0200)
* 9c1d5a3 - Fix missing documentation (Audun Halland, Sun May 12 10:48:58 2019 +0200)
* 522e7ba - Set runtime::Builder core_threads instead of blocking_threads (Audun Halland, Sun May 12 10:36:54 2019 +0200)
* a0dc885 - Shut down MockServer without consuming self, by putting shutdown_tx in an Option (Audun Halland, Sun May 12 10:28:27 2019 +0200)
* 16cc6b6 - Run pact_verifier tests in async mode + pact write lock (Audun Halland, Sun May 12 04:05:08 2019 +0200)
* 39d231d - pact_consumer async support (untested) (Audun Halland, Sun May 12 03:45:05 2019 +0200)
* 2b34371 - Refactor MockServer; move to separate file (Audun Halland, Sun May 12 02:51:22 2019 +0200)
* cd2ef48 - Rename server.rs to hyper_server.rs (Audun Halland, Sun May 12 02:04:00 2019 +0200)
* ab1ff4d - Remove unused import (Audun Halland, Sun May 12 01:44:24 2019 +0200)
* 9e34c33 - Make the old tests in tests.rs work (Audun Halland, Sun May 12 01:42:22 2019 +0200)
* 32b52cd - Manager should not block waiting for match requests. (Audun Halland, Sun May 12 01:19:10 2019 +0200)
* 71dc054 - A failing test for mock server on current_thread runtime (Audun Halland, Sat May 11 22:57:12 2019 +0200)
* 56768ff - Move pact_mock_server_async into pact_mock_server, making it the official implementation (Audun Halland, Sat May 11 22:04:38 2019 +0200)

# 0.4.1 - pact matchig version to 0.5.0

* 61a6b7f - chore: updated release script (Ronald Holshausen, Sun Jun 30 16:15:49 2019 +1000)
* f8fa0d8 - chore: Bump pact matchig version to 0.5.0 (Ronald Holshausen, Sat Jan 5 19:25:53 2019 +1100)
* 386ab52 - fix: corrected the release scripts to check for a version parameter (Ronald Holshausen, Sun Apr 8 13:44:57 2018 +1000)
* 736a6a4 - bump version to 0.4.1 (Ronald Holshausen, Sat Apr 7 14:29:37 2018 +1000)

# 0.4.0 - First V3 specification release

* 398edaf - Upgrade UUID library to latest (Ronald Holshausen, Sat Apr 7 12:29:58 2018 +1000)
* 691b14c - Merge branch 'master' into v3-spec (Ronald Holshausen, Sun Mar 4 17:10:14 2018 +1100)
* 6597141 - WIP - start of implementation of applying generators to the bodies (Ronald Holshausen, Sun Mar 4 17:01:11 2018 +1100)
* 3d01d6e - Merge pull request #31 from andrewspinks/master (Ronald Holshausen, Sun Mar 4 14:18:21 2018 +1100)
* 542b7a3 - Add release script for building an apple universal binary (required for iOS). (Andrew Spinks, Wed Dec 13 11:24:53 2017 +0900)
* 41f1729 - Return a String instead of a serde_json value (Eduard Litau, Mon Dec 4 23:32:19 2017 +0100)
* a76bb5a - Cleaned up all the compiler warnings (Ronald Holshausen, Sun Nov 19 11:29:47 2017 +1100)
* efb17a1 - When there is no content type, default to text/plain (Ronald Holshausen, Sun Nov 19 10:58:17 2017 +1100)
* c4d424b - Wired in the generated request/response into the mock server and verifier (Ronald Holshausen, Tue Nov 7 16:27:01 2017 +1100)
* 13558d6 - Basic generators working (Ronald Holshausen, Tue Nov 7 10:56:55 2017 +1100)
* 7fef36b - Merge branch 'v2-spec' into v3-spec (Ronald Holshausen, Sat Nov 4 12:49:07 2017 +1100)
* ed20d42 - bump version to 0.3.2 (Ronald Holshausen, Fri Nov 3 12:24:46 2017 +1100)
* a905bed - Cleaned up some compiler warnings (Ronald Holshausen, Sun Oct 22 12:26:09 2017 +1100)
* 940a0e3 - Reverted hyper to 0.9.x (Ronald Holshausen, Sun Oct 22 12:01:17 2017 +1100)
* 00dc75a - Bump version to 0.4.0 (Ronald Holshausen, Sun Oct 22 10:46:48 2017 +1100)
* 184127a - Merge branch 'v2-spec' into v3-spec (Ronald Holshausen, Sun Oct 22 10:32:31 2017 +1100)
* e82ee08 - Merge branch 'v2-spec' into v3-spec (Ronald Holshausen, Mon Oct 16 09:24:11 2017 +1100)
* 64ff667 - Upgraded the mock server implemenation to use Hyper 0.11.2 (Ronald Holshausen, Wed Sep 6 12:56:47 2017 +1000)
* e5a93f3 - Merge branch 'master' into v3-spec (Ronald Holshausen, Sun Aug 20 09:53:48 2017 +1000)
* bb46822 - update the mock server to support the V3 format matchers (Ronald Holshausen, Sun Nov 13 16:44:30 2016 +1100)
* 8797c6c - First successful build after merge from master (Ronald Holshausen, Sun Oct 23 11:59:55 2016 +1100)
* 639ac22 - fixes after merge in from master (Ronald Holshausen, Sun Oct 23 10:45:54 2016 +1100)
* 7361688 - moved missing files after merge from master (Ronald Holshausen, Sun Oct 23 10:19:31 2016 +1100)
* 49e45f7 - Merge branch 'master' into v3-spec (Ronald Holshausen, Sun Oct 23 10:10:40 2016 +1100)

# 0.3.1 - Bugfixes plus changes for running with docker

* 5606c0c - Refactored the remaining exported functions into an exported rust one and a FFI one (Ronald Holshausen, Wed Nov 1 11:06:48 2017 +1100)
* dab7fb9 - Renamed the exported functions and refactored what the create_mock_server was doing into a new function (Ronald Holshausen, Wed Nov 1 09:30:07 2017 +1100)
* 24e3f73 - Converted OptionalBody::Present to take a Vec<u8> #19 (Ronald Holshausen, Sun Oct 22 18:04:46 2017 +1100)
* a56b6a6 - Change the column heading to verification state in the mock server list output #24 (Ronald Holshausen, Sun Oct 22 15:15:30 2017 +1100)
* 814fe12 - Modify AssafKatz3's implementation to scan for next available port from a base port number #15 (Ronald Holshausen, Sun Oct 22 14:40:13 2017 +1100)
* 37abe19 - Pulled in changes from https://github.com/AssafKatz3/pact-reference.git #14 (Assaf Katz, Mon Sep 25 12:28:17 2017 +0300)
* c8595cc - Correct the paths in the release scripts for pact_mock_server_cli (Ronald Holshausen, Fri Oct 20 10:48:03 2017 +1100)
* e11bff6 - Correct the paths in the release script after changing to cargo workspace (Ronald Holshausen, Fri Oct 20 10:33:44 2017 +1100)
* 654e875 - bump version to 0.3.1 (Ronald Holshausen, Fri Oct 20 09:50:46 2017 +1100)

# 0.3.0 - Backported matching rules from V3 branch

* aff5b6c - Added cargo update after to release script after bumping the version (Ronald Holshausen, Fri Oct 20 09:41:09 2017 +1100)
* d990729 - Some code cleanup #20 (Ronald Holshausen, Wed Oct 18 16:32:37 2017 +1100)
* c983c63 - Bump versions to 0.3.0 (Ronald Holshausen, Wed Oct 18 13:54:46 2017 +1100)
* 941d0de - Backported the matching rules from the V3 branch #20 (Ronald Holshausen, Mon Oct 31 16:41:03 2016 +1100)
* 06e92e5 - Refer to local libs using version+paths (Eric Kidd, Tue Oct 3 06:22:23 2017 -0400)
* 7afd258 - Update all the cargo manifest versions and commit the cargo lock files (Ronald Holshausen, Wed May 17 10:37:44 2017 +1000)
* 0f22f14 - bump version to 0.2.3 (Ronald Holshausen, Wed May 17 09:57:56 2017 +1000)
* 7d93682 - Move linux specific bits out of the release script (Ronald Holshausen, Wed May 17 09:56:31 2017 +1000)
* adc1505 - Move linux specific bits out of the release script (Ronald Holshausen, Wed May 17 08:50:12 2017 +1000)

# 0.2.2 - Bugfix Release

* be8c299 - Cleanup unused BTreeMap usages and use remote pact dependencies (Anthony Damtsis, Mon May 15 17:09:14 2017 +1000)
* a59fb98 - Migrate remaining pact modules over to serde (Anthony Damtsis, Mon May 15 16:59:04 2017 +1000)
* 84867ac - bump version to 0.2.2 (Ronald Holshausen, Sun Oct 9 16:31:07 2016 +1100)

# 0.2.1 - Changes required for verifying V2 pacts

* 770010a - update projects to use the published pact matching lib (Ronald Holshausen, Sun Oct 9 16:25:15 2016 +1100)
* 574e072 - upadte versions for V2 branch and fix an issue with loading JSON bodies encoded as a string (Ronald Holshausen, Sun Oct 9 15:31:57 2016 +1100)
* a21973a - Get the build passing after merge from V1.1 branch (Ronald Holshausen, Sun Oct 9 13:47:09 2016 +1100)
* 341607c - Merge branch 'v1.1-spec' into v2-spec (Ronald Holshausen, Sun Oct 9 12:10:12 2016 +1100)
* 797c9b9 - correct the URLs to the repos (Ronald Holshausen, Sat Oct 8 17:10:56 2016 +1100)
* ca29349 - bump version to 0.1.2 (Ronald Holshausen, Sat Oct 8 17:09:57 2016 +1100)

# 0.1.1 - Changes required for verifying V1.1 pacts

* a54abd7 - update the dependencies (Ronald Holshausen, Sat Oct 8 17:01:35 2016 +1100)
* a46dabb - update all references to V1 spec after merge (Ronald Holshausen, Sat Oct 8 16:20:51 2016 +1100)
* 63ae7e4 - get project compiling after merge from V1 branch (Ronald Holshausen, Sat Oct 8 15:53:22 2016 +1100)
* 1d6d4f8 - Merge branch 'v1-spec' into v1.1-spec (Ronald Holshausen, Sat Oct 8 15:44:25 2016 +1100)
* 04d9e5f - update the docs for the pact consumer library (Ronald Holshausen, Mon Sep 26 23:06:19 2016 +1000)
* 7dd04e6 - update the release scripts to point the docs to docs.rs (Ronald Holshausen, Mon Sep 26 21:49:35 2016 +1000)
* d8ef338 - bump version to 0.0.3 (Ronald Holshausen, Mon Sep 26 21:48:37 2016 +1000)

# 0.2.0 - V2 specification implementation

* ea9644d - added some V2 matcher tests (Ronald Holshausen, Wed Jul 13 13:35:24 2016 +1000)
* 0e75490 - link to 0.2.0 of the matching library and updated the rust docs (Ronald Holshausen, Tue Jul 12 14:10:02 2016 +1000)
* 534e7a1 - updated readmes and bump versions for the V2 implementation (Ronald Holshausen, Wed Jun 29 10:38:32 2016 +1000)
* f235684 - bump version to 0.1.1 (Ronald Holshausen, Tue Jun 28 21:25:58 2016 +1000)

# 0.1.0 - V1.1 Specification Implementation

* 1e7ab5a - use the V1.1 matching library (Ronald Holshausen, Tue Jun 28 21:17:01 2016 +1000)
* 140526d - Implement V1.1 matching (Ronald Holshausen, Tue Jun 28 15:58:35 2016 +1000)
* 4224875 - update readmes and bump versions for V1.1 implementation (Ronald Holshausen, Tue Jun 28 15:05:39 2016 +1000)
* 91d6d62 - removed the v1 from the project path, will use a git branch instead (Ronald Holshausen, Mon Jun 27 22:09:32 2016 +1000)

# 0.0.2 - Fixes required for verifing pacts

* a0954f9 - prepare for release (Ronald Holshausen, Mon Sep 26 21:32:05 2016 +1000)
* 40c9e02 - exclude IntelliJ files from publishing (Ronald Holshausen, Mon Sep 26 21:22:35 2016 +1000)
* c3a8a30 - renamed the pact_matching and pact_mock_server directories (Ronald Holshausen, Sun Sep 18 11:07:32 2016 +1000)

# 0.0.1 - Feature Release

* 21ca473 - add changelog to libpact_mock_server (Ronald Holshausen, Mon Jun 27 14:59:49 2016 +1000)
* 60077b4 - release script needs to be executable (Ronald Holshausen, Mon Jun 27 14:54:14 2016 +1000)
* 6712635 - added release script for libpact_mock_server (Ronald Holshausen, Mon Jun 27 14:10:20 2016 +1000)
* 0f7965a - updated README for libpact_mock_server (Ronald Holshausen, Mon Jun 27 13:36:37 2016 +1000)
* 518e14a - If the mock server has been shutdown, return a 401 Not Implemented (Ronald Holshausen, Sun Jun 26 11:04:58 2016 +1000)
* 6234bbd - implemented delete on the master server to shut a mock server down (Ronald Holshausen, Sat Jun 25 16:59:39 2016 +1000)
* 4c60f07 - replace rustful with webmachine (Ronald Holshausen, Thu Jun 16 17:31:11 2016 +1000)
* 44daccc - add an optional port number to start the mock server with (Ronald Holshausen, Wed Jun 15 12:40:51 2016 +1000)
* 60bbae5 - handle the result from setting up the logger framework (Ronald Holshausen, Fri Jun 10 11:21:10 2016 +1000)
* 4b8a98a - upgrade hyper to latest version in the mock server library (Ronald Holshausen, Thu Jun 9 21:50:22 2016 +1000)
* b769277 - also add static library as an artifact (Ronald Holshausen, Thu Jun 9 21:22:26 2016 +1000)
* 1c0c7cd - remove rustful from the mock server library (Ronald Holshausen, Thu Jun 9 21:09:32 2016 +1000)
* 7dc4b52 - implemented merging of pact files when writing (Ronald Holshausen, Thu Jun 9 17:34:02 2016 +1000)
* 34fd827 - implement a write_pact exported function to the mock server library (Ronald Holshausen, Thu Jun 9 12:15:01 2016 +1000)
* 769f840 - update the mock server cli readme (Ronald Holshausen, Wed Jun 8 16:05:56 2016 +1000)
* 5f99bb3 - links in readmes are relative to the file they are in (Ronald Holshausen, Wed Jun 8 11:58:05 2016 +1000)
* 0178f8b - change the link to the javascript examples (Ronald Holshausen, Wed Jun 8 11:55:32 2016 +1000)
* 2ba2a08 - correct the link to the javascript examples (Ronald Holshausen, Wed Jun 8 11:46:32 2016 +1000)
* e0130c5 - small tweaks to the libpact_mock_server library readme (Ronald Holshausen, Wed Jun 8 10:46:08 2016 +1000)
* 801f24c - update the github readmes to point to the published rust docs (Ronald Holshausen, Wed Jun 8 10:42:30 2016 +1000)
* 1577eeb - bump the version of libpact_mock_server (Ronald Holshausen, Wed Jun 1 21:59:48 2016 +1000)

# 0.0.0 - First Release
