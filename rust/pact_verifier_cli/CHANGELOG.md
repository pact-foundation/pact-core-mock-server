To generate the log, run `git log --pretty='* %h - %s (%an, %ad)' TAGNAME..HEAD .` replacing TAGNAME and HEAD as appropriate.

# 0.9.5 - Bugfix Release


# 0.9.5 - Bugfix Release

* c97f5d1a - fix: shutdown the tokio reactor correctly when there is an error (Ronald Holshausen, Wed Dec 15 16:28:37 2021 +1100)
* 7c31d981 - bump version to 0.9.5 (Ronald Holshausen, Wed Dec 15 15:59:05 2021 +1100)

# 0.9.4 - Bugfix Release

* 00a00461 - fix: add a small delay at the end of validation to allow async tasks to finish (Ronald Holshausen, Wed Dec 15 15:37:30 2021 +1100)
* d26fa4c5 - bump version to 0.9.4 (Ronald Holshausen, Wed Dec 15 13:56:30 2021 +1100)

# 0.9.3 - Add metrics for provider verification

* f8042d6b - feat: add metrics event for provider verification (Ronald Holshausen, Tue Dec 14 17:29:44 2021 +1100)
* 01171ccb - bump version to 0.9.3 (Ronald Holshausen, Thu Dec 2 12:32:36 2021 +1100)

# 0.9.2 - Bugfix Release

* 491e9259 - chore(pact_verifier_cli): upgrade to latest models crate (Ronald Holshausen, Thu Dec 2 12:22:11 2021 +1100)
* 51a147df - chore: fix docker file (Ronald Holshausen, Tue Nov 16 13:56:24 2021 +1100)
* 2780c93b - bump version to 0.9.2 (Ronald Holshausen, Tue Nov 16 13:18:59 2021 +1100)

# 0.9.1 - Fix for branches and consumer version selectors

* 5d974c4a - chore: update to latest models and plugin driver crates (Ronald Holshausen, Tue Nov 16 11:56:53 2021 +1100)
* df23ba3d - fix: allow multiple consumer version selectors (Matt Fellows, Mon Nov 15 14:28:04 2021 +1100)
* 0af18303 - fix: add missing provider-branch to verifier CLI (Ronald Holshausen, Mon Nov 8 11:40:05 2021 +1100)
* 2db1e1bb - bump version to 0.9.1 (Ronald Holshausen, Thu Nov 4 16:44:12 2021 +1100)

# 0.9.0 - Pact V4 release

* 8d05ddcc - chore: remove beta version from verifier cli (Ronald Holshausen, Thu Nov 4 16:25:02 2021 +1100)
* 400a1231 - chore: drop beta from pact_verifier version (Ronald Holshausen, Thu Nov 4 15:56:22 2021 +1100)
* 296b4370 - chore: update project to Rust 2021 edition (Ronald Holshausen, Fri Oct 22 10:44:48 2021 +1100)
* a561f883 - chore: use the non-beta models crate (Ronald Holshausen, Thu Oct 21 18:10:27 2021 +1100)
* 0c72c80e - chore: fixes after merging from master (Ronald Holshausen, Wed Oct 20 14:46:54 2021 +1100)
* ec265d83 - Merge branch 'master' into feat/plugins (Ronald Holshausen, Wed Oct 20 14:40:37 2021 +1100)
* 87944c79 - bump version to 0.9.0-beta.1 (Ronald Holshausen, Tue Oct 19 18:25:48 2021 +1100)
* 1ce39437 - docs: update verifier CLI docs with consumer version selectors (Matt Fellows, Tue Oct 12 13:26:20 2021 +1100)

# 0.9.0-beta.0 - Pact Plugins Support

* 1aa21870 - chore: update readme with details on plugins (Ronald Holshausen, Tue Oct 19 18:12:51 2021 +1100)
* 5bbdbcfa - refactor: move the CLI functions back from the FFI crate (Ronald Holshausen, Tue Oct 19 18:03:29 2021 +1100)
* e98a91fe - chore: update to latest verifier lib (Ronald Holshausen, Tue Oct 19 17:42:07 2021 +1100)
* 918e5beb - fix: update to latest models and plugin driver crates (Ronald Holshausen, Tue Oct 19 17:09:48 2021 +1100)
* 6f20282d - Merge branch 'master' into feat/plugins (Ronald Holshausen, Tue Sep 28 14:51:34 2021 +1000)
* f14a02b2 - bump version to 0.8.9 (Ronald Holshausen, Tue Sep 28 14:20:41 2021 +1000)
* 75e13fd8 - Merge branch 'master' into feat/plugins (Ronald Holshausen, Mon Aug 23 10:33:45 2021 +1000)
* dfe3cd42 - chore: bump minor version of Pact verifier libs (Ronald Holshausen, Mon Aug 9 15:10:47 2021 +1000)

# 0.8.8 - support native TLS certs

* df715cd5 - feat: support native TLS. Fixes #144 (Matt Fellows, Mon Sep 20 13:00:33 2021 +1000)
* 4458a677 - bump version to 0.8.8 (Ronald Holshausen, Sun Aug 22 16:03:00 2021 +1000)

# 0.8.7 - Bugfix Release

* 38ccd5f6 - bump version to 0.8.7 (Ronald Holshausen, Wed Jul 21 13:38:53 2021 +1000)

# 0.8.6 - Bugfix Release

* b3a6f193 - chore: rename header PACT_MESSAGE_METADATA -> Pact-Message-Metadata (Matt Fellows, Tue Jul 13 11:32:25 2021 +1000)
* 0d5ec68a - feat: copied verfier_ffi crate to pact_ffi (Ronald Holshausen, Sat Jul 10 16:54:28 2021 +1000)
* ac9a657d - chore: updated verifier readme about base64 encoded headers (Matt Fellows, Tue Jul 6 09:17:58 2021 +1000)
* a835e684 - feat: support message metadata in verifications (Matt Fellows, Sun Jul 4 21:02:35 2021 +1000)
* e8d6d844 - fix: pact_verifier_cli was printing the version from the FFI crate (Ronald Holshausen, Sat Jun 5 14:43:38 2021 +1000)
* 2f678213 - feat: initial prototype of a pact file verifier (Ronald Holshausen, Thu Jun 3 14:56:16 2021 +1000)
* 913b7b17 - chore: correct CLI docker release files (Ronald Holshausen, Tue Jun 1 11:25:28 2021 +1000)
* 47046ef5 - bump version to 0.8.6 (Ronald Holshausen, Sun May 30 18:52:34 2021 +1000)

# 0.8.5 - V4 features + updated Tokio to 1.0

* 3a6945e - chore: Upgrade reqwest to 0.11 and hence tokio to 1.0 (Ronald Holshausen, Wed Jan 6 15:34:47 2021 +1100)
* 9eb107a - Revert "Revert "chore: bump version to 0.0.1"" (Ronald Holshausen, Tue Jan 5 17:25:37 2021 +1100)
* 4b4d4a8 - Revert "chore: bump version to 0.0.1" (Ronald Holshausen, Tue Jan 5 17:11:54 2021 +1100)
* 0a210bb - chore: bump version to 0.0.1 (Ronald Holshausen, Tue Jan 5 16:57:47 2021 +1100)
* 2ebeef9 - fix: pact_verifier_cli needs to use Tokio 0.2 (Ronald Holshausen, Tue Jan 5 16:24:29 2021 +1100)
* d9f0e8b - refactor: split pact_verifier ffi functions into seperate crate (Ronald Holshausen, Tue Jan 5 16:17:46 2021 +1100)
* c9e0694 - Revert "Revert "bump version to 0.8.5"" (Ronald Holshausen, Tue Jan 5 15:37:25 2021 +1100)
* 1a4b9a5 - chore: correct the pact_verifier_cli windows release script (Ronald Holshausen, Tue Jan 5 15:36:58 2021 +1100)

# 0.8.4 - TLS support + FFI support

* 41096dc - chore: update release scripts for pact_verifier_cli DLLs (Ronald Holshausen, Tue Jan 5 14:34:55 2021 +1100)
* ef76f38 - chore: cleanup compiler warnings (Ronald Holshausen, Tue Jan 5 10:10:39 2021 +1100)
* 484b747 - fix: verify interaction was blocking the thread (Ronald Holshausen, Mon Jan 4 17:12:38 2021 +1100)
* 4c4eb85 - chore: bump minor version of pact_verifier crate due to breaking changes (Ronald Holshausen, Mon Jan 4 15:48:41 2021 +1100)
* b583540 - Merge branch 'master' into feat/allow-invalid-certs-during-verification (Matt Fellows, Fri Jan 1 14:22:10 2021 +1100)
* 6cec6c7 - feat: allow https scheme and ability to disable ssl verification (Matt Fellows, Thu Dec 31 12:10:57 2020 +1100)
* 79f62ce - Merge branch 'master' into feat/add-verifier-ffi (Matt Fellows, Wed Dec 30 23:21:12 2020 +1100)
* 8aeb567 - wip: minor updates to get FFI interface working (Matt Fellows, Tue Dec 1 19:12:53 2020 +1100)
* c71c78d - wip: add verifier FFI bindings (Matt Fellows, Tue Dec 1 07:04:48 2020 +1100)
* a480e76 - bump version to 0.8.4 (Matt Fellows, Tue Nov 24 11:06:22 2020 +1100)

# 0.8.3 - Bugfix Release

* 280c066 - bump version to 0.8.3 (Matt Fellows, Wed Nov 11 13:30:12 2020 +1100)

# 0.8.2 - Support Pacts for Verification API

* 087fee2 - docs: update verifier docs with new pacts for verification properties (Matt Fellows, Wed Nov 11 10:16:57 2020 +1100)
* e7f729d - wip: further cleanup, and obfuscate auth details (Matt Fellows, Tue Nov 10 13:56:02 2020 +1100)
* ada3667 - wip: cleanup verifier args (Matt Fellows, Tue Nov 10 08:13:01 2020 +1100)
* 80f4e98 - wip: refactor BrokerWithDynamicConfiguration into a struct enum for better readability (Matt Fellows, Mon Nov 9 22:40:24 2020 +1100)
* 60c1671 - wip: thread verification context into pact fetching/verification, add env vars to clap args (Matt Fellows, Sun Nov 8 13:25:17 2020 +1100)
* 60eb190 - wip: map tags to consumer version selectors (Matt Fellows, Sat Nov 7 23:35:36 2020 +1100)
* 6612a3a - wip: basic wiring in of the pacts for verification endpoint (Matt Fellows, Sat Nov 7 21:39:25 2020 +1100)
* 33864a5 - bump version to 0.8.2 (Ronald Holshausen, Fri Oct 16 12:40:37 2020 +1100)

# 0.8.1 - arrayContains matcher + text/xml content type

* 7fbc731 - chore: bump minor version of matching lib (Ronald Holshausen, Fri Oct 9 10:42:33 2020 +1100)
* c2fda15 - chore: update readme on verifying message pacts (Ronald Holshausen, Tue Sep 15 11:13:16 2020 +1000)
* 0dbcda9 - bump version to 0.8.1 (Ronald Holshausen, Mon Sep 14 17:34:25 2020 +1000)

# 0.8.0 - Supports verifying Message Pacts

* ef5f88c - chore: bump minor version of the pact_verifier crate (Ronald Holshausen, Mon Sep 14 17:13:45 2020 +1000)
* 2d44ffd - chore: bump minor version of the matching crate (Ronald Holshausen, Mon Sep 14 12:06:37 2020 +1000)
* fb6c19c - refactor: allow verifier to handle different types of interactions (Ronald Holshausen, Mon Sep 14 10:41:13 2020 +1000)
* 814c416 - refactor: added a trait for interactions, renamed Interaction to RequestResponseInteraction (Ronald Holshausen, Sun Sep 13 17:09:41 2020 +1000)
* 77c8c8d - bump version to 0.7.2 (Ronald Holshausen, Sun Aug 23 17:19:24 2020 +1000)

# 0.7.1 - implemented provider state generator

* b186ce9 - chore: update all dependent crates (Ronald Holshausen, Sun Aug 23 16:49:00 2020 +1000)
* 61ca3d7 - chore: update matching crate to latest (Ronald Holshausen, Sun Aug 23 16:37:58 2020 +1000)
* ed207a7 - chore: updated readmes for docs site (Ronald Holshausen, Sun Jun 28 10:04:09 2020 +1000)

# 0.7.0 - Updated XML Matching

* 62b0bda - chore: update to latest matching library (Ronald Holshausen, Wed Jun 24 12:17:04 2020 +1000)
* bea787c - chore: bump matching crate version to 0.6.0 (Ronald Holshausen, Sat May 23 17:56:04 2020 +1000)
* 76250b5 - chore: correct some clippy warnings (Ronald Holshausen, Wed Apr 29 17:53:40 2020 +1000)
* 43de9c3 - chore: update matching library to latest (Ronald Holshausen, Fri Apr 24 10:20:55 2020 +1000)
* bd10d00 - Avoid deprecated Error::description in favor of Display trait (Caleb Stepanian, Mon Mar 30 16:49:13 2020 -0400)
* 1cf0199 - refactor: moved state change code to a handler (Ronald Holshausen, Wed Mar 11 14:37:07 2020 +1100)
* 70e6648 - chore: converted verifier to use Reqwest (Ronald Holshausen, Mon Mar 9 12:20:14 2020 +1100)
* fe74376 - feat: implemented publishing provider tags with verification results #57 (Ronald Holshausen, Sun Mar 8 18:37:21 2020 +1100)
* a6e0c16 - Fix RequestFilterExecutor w/ verify_provider (Andrew Lilley Brinker, Mon Mar 2 11:43:59 2020 -0800)
* d944a60 - chore: added callback executors so test code can called during verification (Ronald Holshausen, Sun Feb 23 18:43:49 2020 +1100)
* f238ca1 - Make pact_verifier_cli actually runnable by using tokio::main (Audun Halland, Sun Jan 19 10:12:17 2020 +0100)
* 70a33dd - chore: bump minor version of pact_verifier (Ronald Holshausen, Sun Jan 19 11:51:36 2020 +1100)
* cb4c560 - Upgrade tokio to 0.2.9 (Audun Halland, Fri Jan 10 00:13:02 2020 +0100)
* deaf4b3 - pact_verifier_cli: Increase type length limit for big generated future type (Audun Halland, Tue Dec 17 01:53:24 2019 +0100)
* 87d787f - pact_verifier_cli: Block on async function from pact_verifier (Audun Halland, Thu Dec 12 11:15:44 2019 +0100)
* c168d0b - pact_verifier_cli: Remove extern crate from main.rs (Audun Halland, Sun Nov 17 23:25:17 2019 +0100)
* 713cd6a - Explicit edition 2018 in Cargo.toml files (Audun Halland, Sat Nov 16 23:55:37 2019 +0100)
* 9f3ad74 - fix: docker build now requires libclang system library (Ronald Holshausen, Fri Sep 27 17:14:05 2019 +1000)
* 834a60b - bump version to 0.6.2 (Ronald Holshausen, Fri Sep 27 15:37:03 2019 +1000)

# 0.6.1 - Bugfix + Oniguruma crate for regex matching

* e32350e - chore: use the latest matching lib (Ronald Holshausen, Fri Sep 27 15:22:12 2019 +1000)
* 0cc03db - bump version to 0.6.1 (Ronald Holshausen, Sun Sep 22 18:13:48 2019 +1000)

# 0.6.0 - Publishing verification results

* 0e1da1b - chore: bump minor version (Ronald Holshausen, Sun Sep 22 17:59:51 2019 +1000)
* 2e07d77 - chore: set the version of the pact matching crate (Ronald Holshausen, Sun Sep 22 17:24:02 2019 +1000)
* 1110b47 - feat: implemented publishing verification results to the pact broker #44 (Ronald Holshausen, Sun Sep 22 13:53:27 2019 +1000)
* 7b5a404 - bump version to 0.5.2 (Ronald Holshausen, Sat Aug 24 13:00:10 2019 +1000)

# 0.5.1 - Use reqwest for better HTTP/S support, support headers with multiple values

* f79b033 - chore: update terminal support in release scripts (Ronald Holshausen, Sat Aug 24 12:25:28 2019 +1000)
* b8019ba - chore: bump the version of the matching lib (Ronald Holshausen, Sat Aug 24 12:22:35 2019 +1000)
* dac8ae1 - feat: support authentication when fetching pacts from a pact broker (Ronald Holshausen, Sun Aug 11 13:57:29 2019 +1000)
* e007763 - feat: support bearer tokens when fetching pacts from URLs (Ronald Holshausen, Sun Aug 11 13:21:17 2019 +1000)
* f947d43 - chore: upgrade the logging crates (Ronald Holshausen, Sun Aug 11 12:05:14 2019 +1000)
* 0dd10e6 - fix: docker release script (Ronald Holshausen, Sat Jul 27 18:02:11 2019 +1000)
* aa336e6 - bump version to 0.5.1 (Ronald Holshausen, Sat Jul 27 17:48:41 2019 +1000)

# 0.5.0 - Upgrade to non-blocking Hyper 0.12

* d842100 - chore: bump component versions to 0.5.0 (Ronald Holshausen, Sat Jul 27 15:44:51 2019 +1000)
* 47ab6d0 - Upgrade tokio to 0.1.22 everywhere (Audun Halland, Mon Jul 22 23:47:09 2019 +0200)
* 2f8a997 - Compile everything (except the commented-out tests) (Audun Halland, Thu May 2 00:41:56 2019 +0200)
* f8fa0d8 - chore: Bump pact matchig version to 0.5.0 (Ronald Holshausen, Sat Jan 5 19:25:53 2019 +1100)
* 3c33294 - fix: Only print errors in the CLI to STDERR #28 (Ronald Holshausen, Sun Apr 8 15:57:56 2018 +1000)
* 386ab52 - fix: corrected the release scripts to check for a version parameter (Ronald Holshausen, Sun Apr 8 13:44:57 2018 +1000)
* 6c2d6c8 - chore: added docker release scripts for the CLIs (Ronald Holshausen, Sun Apr 8 13:44:18 2018 +1000)
* 9d24b7e - fix: corrected the docker build for the verifier cli #14 (Ronald Holshausen, Sun Apr 8 13:39:29 2018 +1000)
* 4b8fb64 - fix: verification CLI was reporting incorrect pact specification version (Ronald Holshausen, Sun Apr 8 13:12:45 2018 +1000)
* fb8ecf5 - bump version to 0.4.1 (Ronald Holshausen, Sat Apr 7 15:23:33 2018 +1000)

# 0.4.0 - First V3 specification release

* 6597141 - WIP - start of implementation of applying generators to the bodies (Ronald Holshausen, Sun Mar 4 17:01:11 2018 +1100)
* f63f339 - replaced use of try macro with ? (Ronald Holshausen, Tue Nov 7 16:31:39 2017 +1100)
* 7fef36b - Merge branch 'v2-spec' into v3-spec (Ronald Holshausen, Sat Nov 4 12:49:07 2017 +1100)
* 5c05f18 - Added docker file for pact verifier (Ronald Holshausen, Fri Nov 3 16:20:02 2017 +1100)
* 6a0548c - Correct release scripts (Ronald Holshausen, Fri Nov 3 15:51:52 2017 +1100)
* 9f20613 - bump version to 0.3.1 (Ronald Holshausen, Fri Nov 3 15:51:27 2017 +1100)
* 91a5673 - Correct the release script (Ronald Holshausen, Fri Nov 3 15:42:48 2017 +1100)
* 00dc75a - Bump version to 0.4.0 (Ronald Holshausen, Sun Oct 22 10:46:48 2017 +1100)
* 184127a - Merge branch 'v2-spec' into v3-spec (Ronald Holshausen, Sun Oct 22 10:32:31 2017 +1100)
* e82ee08 - Merge branch 'v2-spec' into v3-spec (Ronald Holshausen, Mon Oct 16 09:24:11 2017 +1100)
* 64ff667 - Upgraded the mock server implemenation to use Hyper 0.11.2 (Ronald Holshausen, Wed Sep 6 12:56:47 2017 +1000)
* e5a93f3 - Merge branch 'master' into v3-spec (Ronald Holshausen, Sun Aug 20 09:53:48 2017 +1000)
* 639ac22 - fixes after merge in from master (Ronald Holshausen, Sun Oct 23 10:45:54 2016 +1100)
* 49e45f7 - Merge branch 'master' into v3-spec (Ronald Holshausen, Sun Oct 23 10:10:40 2016 +1100)
* 539eb48 - updated all the readmes and cargo manefests for v3 (Ronald Holshausen, Tue Jul 19 15:46:18 2016 +1000)

# 0.3.0 - Backported matching rules from V3 branch

* b2ad496 - Updated the verifier cli dep modules (Ronald Holshausen, Fri Nov 3 15:14:57 2017 +1100)
* ac94388 - Tests are now all passing #20 (Ronald Holshausen, Thu Oct 19 15:14:25 2017 +1100)
* c983c63 - Bump versions to 0.3.0 (Ronald Holshausen, Wed Oct 18 13:54:46 2017 +1100)
* 06e92e5 - Refer to local libs using version+paths (Eric Kidd, Tue Oct 3 06:22:23 2017 -0400)
* 7afd258 - Update all the cargo manifest versions and commit the cargo lock files (Ronald Holshausen, Wed May 17 10:37:44 2017 +1000)
* be8c299 - Cleanup unused BTreeMap usages and use remote pact dependencies (Anthony Damtsis, Mon May 15 17:09:14 2017 +1000)
* a59fb98 - Migrate remaining pact modules over to serde (Anthony Damtsis, Mon May 15 16:59:04 2017 +1000)
* d5e6ce0 - bump version to 0.2.1 (Ronald Holshausen, Sun Oct 9 17:20:25 2016 +1100)

# 0.2.0 - V2 specification implementation

* 38027f8 - updated the pact_verifier_cli to V2 (Ronald Holshausen, Sun Oct 9 17:12:35 2016 +1100)
* 770010a - update projects to use the published pact matching lib (Ronald Holshausen, Sun Oct 9 16:25:15 2016 +1100)
* 574e072 - upadte versions for V2 branch and fix an issue with loading JSON bodies encoded as a string (Ronald Holshausen, Sun Oct 9 15:31:57 2016 +1100)
* b0bebb7 - bump version to 0.1.1 (Ronald Holshausen, Sun Oct 9 11:27:41 2016 +1100)

# 0.1.0 - V1.1 specification implementation

* ea1ef54 - Updated dependencies and version for release of pact_verifier_cli (Ronald Holshausen, Sun Oct 9 10:56:34 2016 +1100)
* 1f3f3f1 - correct the versions of the inter-dependent projects as they were causing the build to fail (Ronald Holshausen, Sat Oct 8 17:41:57 2016 +1100)
* a46dabb - update all references to V1 spec after merge (Ronald Holshausen, Sat Oct 8 16:20:51 2016 +1100)
* b6df52f - bump version to 0.0.1 (Ronald Holshausen, Tue Sep 27 22:27:26 2016 +1000)

# 0.0.0 - First Release
