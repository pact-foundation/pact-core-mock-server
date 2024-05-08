To generate the log, run `git log --pretty='* %h - %s (%an, %ad)' TAGNAME..HEAD .` replacing TAGNAME and HEAD as appropriate.

# 1.0.6 - Maintenance Release

* cc62ff52 - chore: Update pact_mock_server crate to 1.2.7 (Ronald Holshausen, Wed May 8 10:38:25 2024 +1000)
* 1d5b9300 - chore: Update repo in Cargo.toml (Ronald Holshausen, Tue May 7 17:11:46 2024 +1000)
* eb8e175e - chore: Update readme with new project layout (Ronald Holshausen, Tue May 7 16:14:04 2024 +1000)
* c27bf2b5 - chore: Move the mock server directories to the project root (Ronald Holshausen, Tue May 7 15:55:29 2024 +1000)
* f5ad729f - chore: Moved mock server crates to pact-foundation/pact-core-mock-server (Ronald Holshausen, Tue May 7 17:36:26 2024 +1000)
* fa2b1d09 - chore: Upgrade pact_matching to 1.2.2 (Ronald Holshausen, Tue May 7 10:50:08 2024 +1000)
* 694100fb - chore: Update pact_models to 1.2.0 (Ronald Holshausen, Tue Apr 23 10:51:11 2024 +1000)
* edfac7ba - chore: remove local pact_models from the other pact crates (Ronald Holshausen, Tue Apr 23 10:03:29 2024 +1000)
* 758f4c03 - chore: Update pact_matching to 1.2.1 (Ronald Holshausen, Tue Apr 16 16:29:38 2024 +1000)
* bfd6eae0 - chore: Upgrade pact_mock_server to 1.2.6 (Ronald Holshausen, Tue Apr 16 15:14:00 2024 +1000)
* d6125b75 - chore(pact_matching): Bump minor version (Ronald Holshausen, Tue Apr 16 10:16:44 2024 +1000)
* 0d707388 - chore(pact_mock_server_cli): Create Multi-platform image (Ronald Holshausen, Fri Mar 15 17:12:20 2024 +1100)
* c9da801a - bump version to 1.0.6 (Ronald Holshausen, Fri Mar 15 15:37:22 2024 +1100)

# 1.0.5 - Maintenance Release

* 2f974fc4 - chore(pact_mock_server_cli): Update dependencies (Ronald Holshausen, Fri Mar 15 15:15:40 2024 +1100)
* ffffa61d - chore: Upgrade pact_mock_server to 1.2.5 (Ronald Holshausen, Fri Mar 15 15:13:04 2024 +1100)
* 6406e9ae - refactor(release): Rename OSX to MacOS (Ronald Holshausen, Fri Mar 15 14:28:06 2024 +1100)
* 7732a637 - chore(ci): refactor release pipeline (JP-Ellis, Mon Feb 26 17:54:18 2024 +1100)
* 41faa544 - chore: Lock clap crate to 4.4 as 4.5 requires Rust 1.75 (Ronald Holshausen, Mon Feb 12 15:16:18 2024 +1100)
* 8d197959 - chore: fixup path to aarch64-unknown-linux-musl artifact (Yousaf Nabi, Thu Feb 8 21:13:21 2024 +0000)
* 9196efe4 - feat: verifier/mock_server clis - build for musl (Yousaf Nabi, Thu Feb 8 20:36:10 2024 +0000)
* a52e0ee9 - chore: Upgrade pact_matching to 1.1.10 (Ronald Holshausen, Wed Feb 7 13:20:45 2024 +1100)
* 24a26cca - chore: Update pact_models to 1.1.18 (Ronald Holshausen, Wed Feb 7 10:53:22 2024 +1100)
* 73578350 - chore: use local pact_models (JP-Ellis, Tue Feb 6 10:51:09 2024 +1100)
* 113470ca - chore: Update release script (Ronald Holshausen, Sat Jan 20 19:19:40 2024 +1100)
* ba1f77a9 - bump version to 1.0.5 (Ronald Holshausen, Sat Jan 20 19:18:42 2024 +1100)

# 1.0.4 - Maintenance Release

* 0496fb75 - chore(pact_mock_server_cli): Upgrade dependencies (Ronald Holshausen, Sat Jan 20 19:06:50 2024 +1100)
* def5685c - chore: Upgrade pact_mock_server to 1.2.4 (Ronald Holshausen, Sat Jan 20 18:56:26 2024 +1100)
* 6fa097a5 - Merge branch 'master' into ci/cargo_clean (Ronald Holshausen, Thu Jan 25 10:22:25 2024 +1100)
* 36390097 - Merge pull request #371 from YOU54F/chore/renable_verifier_aarch64_linux (Ronald Holshausen, Thu Jan 25 09:59:53 2024 +1100)
* 00402ba4 - chore: migrate target/artifacts to release_artifacts to avoid cargo clean loss (Yousaf Nabi, Wed Jan 24 15:26:18 2024 +0000)
* 22331310 - ci: cargo clean prior to cross build (Yousaf Nabi, Wed Jan 24 15:24:10 2024 +0000)
* c412829f - feat: build aarch64-pc-windows-msvc target (ffi/mock_server/verifier) (Yousaf Nabi, Wed Jan 24 12:01:50 2024 +0000)
* a2ba1cd5 - fix: pin cross to 0.2.5 for glibc 2.23 (Yousaf Nabi, Wed Jan 24 11:54:22 2024 +0000)
* e552bdce - chore: Upgrade pact_matching to 1.1.9 (Ronald Holshausen, Sat Jan 20 15:13:13 2024 +1100)
* b735df9d - chore: Upgrade pact_models to 1.1.17 (Ronald Holshausen, Sat Jan 20 13:54:03 2024 +1100)
* 1a4bcd27 - chore: Upgrade pact_matching to 1.1.8 (Ronald Holshausen, Fri Jan 19 18:24:54 2024 +1100)
* 944613df - fix: regression - upgrade pact_models to 1.1.16 #359 (Ronald Holshausen, Fri Jan 19 14:52:36 2024 +1100)
* 403c0af1 - chore: Upgrade pact_models to 1.1.14 #355 (Ronald Holshausen, Tue Jan 16 10:31:12 2024 +1100)
* dfd13760 - chore: Upgrade pact_models to 1.1.13 #355 (Ronald Holshausen, Tue Jan 16 07:42:33 2024 +1100)
* 3f0ae7f1 - chore: Upgrade pact_matching to 1.1.7 (Ronald Holshausen, Tue Nov 14 03:10:25 2023 +1100)
* 826758a6 - chore: Upgrade pact_models to 1.1.12 (Ronald Holshausen, Mon Nov 13 17:25:21 2023 +1100)
* 04bad264 - chore: Upgrade pact_matching to 1.1.6 (Ronald Holshausen, Fri Sep 22 11:03:38 2023 +1000)
* d592cd8b - chore: Upgrade pact_mock_server to 1.2.3 (Ronald Holshausen, Tue Aug 29 09:25:14 2023 +1000)
* 3ec99c41 - chore: Upgrade pact_matching to 1.1.5 (Ronald Holshausen, Fri Aug 18 15:40:02 2023 +1000)
* e4da3e42 - chore: Upgrade pact_models to 1.1.11 (Ronald Holshausen, Mon Aug 7 13:59:34 2023 +1000)
* 24ed7835 - chore: Upgrade pact-models to 1.1.10 (Ronald Holshausen, Fri Aug 4 16:11:24 2023 +1000)
* 0586fcf1 - chore: Upgrade pact_mock_server to 1.2.2 (Ronald Holshausen, Thu Jul 27 14:45:02 2023 +1000)
* 8f88192e - chore: Upgrade pact_matching to 1.1.4 (Ronald Holshausen, Thu Jul 27 14:35:27 2023 +1000)
* 4a01919a - chore: Upgrade pact_models to 1.1.9 (Ronald Holshausen, Thu Jul 27 10:24:00 2023 +1000)
* c2aad1ac - chore: Add support for datetime, xml, multipart and plugins crate features (Ronald Holshausen, Wed Jul 12 11:15:37 2023 +1000)
* a3726bb8 - bump version to 1.0.4 (Ronald Holshausen, Tue Jul 11 15:12:25 2023 +1000)

# 1.0.3 - Maintenance release

* de1f04e7 - chore: Update readme (Ronald Holshausen, Tue Jul 11 15:07:02 2023 +1000)
* 66d010fb - chore: Add support for datetime, xml, multipart. plugins and tls crate features (Ronald Holshausen, Tue Jul 11 15:04:13 2023 +1000)
* e07ca36c - chore: Upgrade pact_mock_server to 1.2.1 (Ronald Holshausen, Tue Jul 11 14:47:17 2023 +1000)
* 348eb3f3 - chore: Upgrade pact_matcing to 1.1.3 (Ronald Holshausen, Tue Jul 11 11:38:26 2023 +1000)
* b18b9dff - chore: Upgrade pact_matching to 1.1.2 (Ronald Holshausen, Mon Jul 10 16:42:27 2023 +1000)
* 1deca59a - chore: Upgrade pact_models to 1.1.8 (Ronald Holshausen, Mon Jul 10 16:15:43 2023 +1000)
* 2662cdfc - chore: Upgrade pact_models to 1.1.7 (Ronald Holshausen, Thu Jul 6 10:27:25 2023 +1000)
* e95ae4d0 - chore: Upgrade pact_models to 1.1.6 (Ronald Holshausen, Thu Jun 22 15:40:55 2023 +1000)
* 5331f99e - bump version to 1.0.3 (Ronald Holshausen, Wed Jun 14 16:19:39 2023 +1000)

# 1.0.2 - Fixes a deadlock caused when getting mock server JSON

* 834f77cc - chore: Upgrade pact_mock_server to 1.2.0 (Ronald Holshausen, Wed Jun 14 15:22:11 2023 +1000)
* 41147cf5 - chore(pact_mock_server_cli): Upgrade Clap to v4 (Ronald Holshausen, Wed Jun 14 15:01:45 2023 +1000)
* fc4ddeb7 - chore(pact_mock_server_cli): Add actions to all Clap args (Ronald Holshausen, Wed Jun 14 14:03:44 2023 +1000)
* e58aa917 - fix: no need to wrap the Pact for a mock server in a mutex (mock server is already behind a mutex) as this can cause deadlocks #274 (Ronald Holshausen, Wed Jun 14 13:26:54 2023 +1000)
* d6775858 - chore(pact_mock_server_cli): Fix all the Clap deprecation warnings (Ronald Holshausen, Tue Jun 13 17:16:03 2023 +1000)
* 17a81361 - chore(pact_mock_server_cli): Switch to using tracing crate (Ronald Holshausen, Tue Jun 13 14:42:52 2023 +1000)
* 2047ce09 - bump version to 1.0.2 (Ronald Holshausen, Tue Jun 13 11:35:52 2023 +1000)

# 1.0.1 - Adds aarch64 osx binary + bug fixes

* 2a21bc0e - chore: correct the pact_matching version (Ronald Holshausen, Tue Jun 13 11:31:32 2023 +1000)
* 666535ab - Revert "update changelog for release 1.0.1" (Ronald Holshausen, Tue Jun 13 11:30:16 2023 +1000)
* cf2914ec - update changelog for release 1.0.1 (Ronald Holshausen, Tue Jun 13 11:28:38 2023 +1000)
* bc68ed7f - chore: Upgrade pact_models to 1.1.4 (Ronald Holshausen, Thu Jun 1 10:22:38 2023 +1000)
* 397c837f - chore: Upgrade pact_models to 1.1.3 (fixes MockServerURL generator) (Ronald Holshausen, Mon May 29 15:12:22 2023 +1000)
* 8e9bd503 - chore: Upgrade pact_mock_server to 1.1.0 (Ronald Holshausen, Tue May 23 12:20:01 2023 +1000)
* ac2e24da - chore: Use "Minimum version, with restricted compatibility range" for all Pact crate versions (Ronald Holshausen, Tue May 23 11:46:52 2023 +1000)
* 6df4670c - chore: Upgrade pact_matching to 1.1.1 (Ronald Holshausen, Tue May 23 11:32:51 2023 +1000)
* 54887690 - chore: Bump pact_matching to 1.1 (Ronald Holshausen, Tue May 23 11:13:14 2023 +1000)
* 261ecf47 - fix: Add RefUnwindSafe trait bound to all Pact and Interaction uses (Ronald Holshausen, Mon May 15 13:59:31 2023 +1000)
* 6a71b12d - chore: Upgrade pact_mock_server to 1.0.2 (Ronald Holshausen, Tue Apr 18 13:30:21 2023 +1000)
* 0bcba082 - chore: Upgrade pact_matching to 1.0.8 (Ronald Holshausen, Tue Apr 18 13:14:38 2023 +1000)
* 6c14abfd - chore: Upgrade pact_models to 1.0.13 (Ronald Holshausen, Tue Apr 18 13:00:01 2023 +1000)
* ce16d43f - chore: Upgrade pact-plugin-driver to 0.4.2 (supports auto-installing known plugins) (Ronald Holshausen, Tue Apr 18 11:49:52 2023 +1000)
* 10bf1a48 - chore: Upgrade pact_models to 1.0.12 (fixes generators hash function) (Ronald Holshausen, Mon Apr 17 10:31:09 2023 +1000)
* 84b9d9e9 - fix: Upgrade pact models to 1.0.11 (fixes generated key for V4 Pacts) (Ronald Holshausen, Fri Apr 14 17:10:58 2023 +1000)
* 669f7812 - chore: Upgrade pact_models to 1.0.10 (Ronald Holshausen, Thu Apr 13 15:32:34 2023 +1000)
* a7349c74 - feat: pact_mock_server_cli aarch64 osx (Yousaf Nabi, Wed Apr 12 12:29:14 2023 +0100)
* b81d1d86 - bump version to 1.0.1 (Ronald Holshausen, Wed Apr 5 09:27:43 2023 +1000)

# 1.0.0 - Maintenance Release

* bb314a73 - chore(pact_mock_server_cli): Upgrade clap to v3 (Ronald Holshausen, Wed Apr 5 09:06:54 2023 +1000)
* 7e99decf - chore: add CLI tests (Ronald Holshausen, Tue Apr 4 16:33:28 2023 +1000)
* d2c8752e - chore: Update dependencies (Ronald Holshausen, Tue Apr 4 15:47:50 2023 +1000)
* 81a9b306 - chore: Upgrade pact_mock_server to 1.0.1 (Ronald Holshausen, Tue Apr 4 15:40:20 2023 +1000)
* 126cf462 - chore: Upgrade pact_matching to 1.0.7 (Ronald Holshausen, Tue Apr 4 15:12:28 2023 +1000)
* 11c701b4 - fix: Upgrade pact_matching to 1.0.6 (fixes some issues with matching HTTP headers) (Ronald Holshausen, Wed Mar 15 14:54:54 2023 +1100)
* e96bc54e - fix: Upgrade pact_models to 1.0.9 (fixes issues with headers) (Ronald Holshausen, Wed Mar 15 14:31:00 2023 +1100)
* f7e0b669 - chore: Upgrade pact_models to 1.0.8 (Ronald Holshausen, Wed Mar 15 12:19:22 2023 +1100)
* 7589b9b0 - chore: Bump pact_mock_server version to 1.0.0 (Ronald Holshausen, Fri Feb 10 14:43:53 2023 +1100)
* 019bd2fe - chore: Upgrade pact_matching to 1.0.5 (Ronald Holshausen, Wed Feb 8 13:53:15 2023 +1100)
* 0f4178e5 - chore: Upgrade pact_matching to 1.0.4 (Ronald Holshausen, Mon Feb 6 15:40:43 2023 +1100)
* c1b22f1c - chore: Upgrade pact_matching to 1.0.3 (Ronald Holshausen, Wed Jan 11 15:19:29 2023 +1100)
* 7d84d941 - chore: Upgrade pact_models to 1.0.4 (Ronald Holshausen, Wed Jan 11 14:33:13 2023 +1100)
* e4a44252 - chore: removed the quicktest for port validation as it fails from time to time (Ronald Holshausen, Mon Jan 9 09:45:01 2023 +1100)
* 33f39f2c - chore: removed the quicktest for port validation as it fails from time to time (Ronald Holshausen, Mon Jan 9 09:35:16 2023 +1100)
* 1bdb1054 - chore: Upgrade pact_models to 1.0.3 #239 (Ronald Holshausen, Thu Dec 22 15:37:53 2022 +1100)
* 81e55220 - chore: Upgrade pact_mock_server to 0.9.7 (Ronald Holshausen, Mon Dec 19 16:04:55 2022 +1100)
* e827f591 - chore: Upgrade pact_matching to 1.0.2 (Ronald Holshausen, Mon Dec 19 15:30:14 2022 +1100)
* fb2f4204 - chore: Upgrade pact_matching to 1.0.1 (Ronald Holshausen, Wed Dec 14 17:03:31 2022 +1100)
* 9be00044 - chore: Upgrade pact_mock_server to 0.9.6 (Ronald Holshausen, Mon Dec 12 10:06:25 2022 +1100)
* e7a1b9f2 - chore: Upgrade pact_matching to 1.0 and plugin driver to 0.2 (Ronald Holshausen, Fri Dec 9 17:29:33 2022 +1100)
* 246c0730 - chore: Upgrade pact_mock_server to 0.9.5 (Ronald Holshausen, Mon Nov 28 14:52:37 2022 +1100)
* 2802fffd - chore: Upgrade pact_matching to 0.12.15 (Ronald Holshausen, Mon Nov 28 14:29:43 2022 +1100)
* c9721fd5 - chore: Upgrade pact_models to 1.0.1 and pact-plugin-driver to 0.1.16 (Ronald Holshausen, Mon Nov 28 14:10:53 2022 +1100)
* a3110bd6 - chore: Upgrade pact_mock_server to 0.9.4 (Ronald Holshausen, Mon Nov 7 11:50:05 2022 +1100)
* 123060e3 - chore: Upgrade pact_matching to 0.12.14 (Ronald Holshausen, Mon Nov 7 11:34:36 2022 +1100)
* 577824e7 - fix: Upgrade pact_models to 1.0 and pact-plugin-driver to 0.1.15 to fix cyclic dependency issue (Ronald Holshausen, Mon Nov 7 11:14:20 2022 +1100)
* e1f985ad - chore: Upgrade pact_models to 0.4.6 and pact-plugin-driver to 0.1.14 (Ronald Holshausen, Fri Nov 4 16:38:36 2022 +1100)
* 6ad00a5d - fix: Update onig to latest master to fix  Regex Matcher Fails On Valid Inputs #214 (Ronald Holshausen, Fri Nov 4 15:23:50 2022 +1100)
* 02d9e2cb - chore: Upgrade pact matching crate to 0.12.12 (Ronald Holshausen, Wed Sep 28 10:11:11 2022 +1000)
* ac4fe73f - chore: fix to release scripts (Ronald Holshausen, Wed Sep 7 10:51:01 2022 +1000)
* f8db90d2 - fix: Upgrade pact_models to 0.4.5 - fixes FFI bug with generators for request paths (Ronald Holshausen, Fri Aug 26 11:44:08 2022 +1000)
* 9d1e8e89 - chore: Upgrade pact_mock_server to 0.9.3 (Ronald Holshausen, Thu Aug 18 16:03:38 2022 +1000)
* 1d5fb787 - chore: Upgrade pact_matching to 0.12.11 (Ronald Holshausen, Thu Aug 18 15:07:23 2022 +1000)
* 32a70382 - chore: Upgrade pact_models (0.4.4), plugin driver (0.1.10), tracing and tracing core crates (Ronald Holshausen, Thu Aug 18 14:41:52 2022 +1000)
* 98c5e2db - chore: correct pact_mock_server_cli docker file (Ronald Holshausen, Tue Aug 16 10:31:50 2022 +1000)
* f2e3d9ff - bump version to 0.7.9 (Ronald Holshausen, Tue Aug 16 10:15:14 2022 +1000)

# 0.7.8 - Maintenance Release

* a41fe69c - chore: Upgrade pact_mock_server to 0.9.2 (Ronald Holshausen, Mon Aug 15 17:40:09 2022 +1000)
* e3bef155 - feat: Add ARM64 (aarch64) linux targets to the release build #160 (Ronald Holshausen, Mon Aug 15 16:13:22 2022 +1000)
* 7b6a919b - chore: Upgrade pact_matching crate to 0.12.10 (Ronald Holshausen, Wed Aug 10 12:37:11 2022 +1000)
* 195ad07b - chore: Updated dependant crates (uuid, simplelog) (Ronald Holshausen, Wed Aug 10 10:22:07 2022 +1000)
* a3fe5e7f - chore: Update pact models to 0.4.2 (Ronald Holshausen, Wed Aug 10 10:10:41 2022 +1000)
* fcbee0c2 - fix: rust/pact_mock_server_cli/Dockerfile to reduce vulnerabilities (snyk-bot, Thu Jul 7 11:09:31 2022 +0000)
* 9a6c846f - chore: Upgrade pact_matching to 0.12.9 (Ronald Holshausen, Fri Jun 10 15:46:07 2022 +1000)
* f42026d5 - chore: Upgrade pact_mock_server to 0.9.1 (Ronald Holshausen, Mon May 30 12:09:06 2022 +1000)
* bcddbcfb - chore: Upgrade pact_matching to 0.12.8 (Ronald Holshausen, Mon May 30 11:52:26 2022 +1000)
* 26bb612b - chore: changes required for 0.9.0 of mock server crate (Ronald Holshausen, Fri May 20 16:00:41 2022 +1000)
* d9b9fe72 - chore: Upgrade pact-plugin-driver to 0.1.7 (Ronald Holshausen, Fri May 20 15:56:23 2022 +1000)
* f8471bb7 - chore: switch from log crate to tracing crate (Ronald Holshausen, Fri May 13 13:47:18 2022 +1000)
* f6b942da - chore: Upgrade pact_mock_server to 0.8.11 (Ronald Holshausen, Wed May 11 17:00:46 2022 +1000)
* 08f28e4a - chore: Upgrade pact_matching to 0.12.7 (Ronald Holshausen, Wed May 11 15:57:36 2022 +1000)
* 020b5715 - chore: upgrade pact_models to 0.4.1 (Ronald Holshausen, Wed May 11 11:36:57 2022 +1000)
* 563ae9fc - chore: Upgrade pact_mock_server to 0.8.10 (Ronald Holshausen, Wed Apr 27 15:06:50 2022 +1000)
* bcae77b4 - chore: upgrade pact_matching to 0.12.6 (Ronald Holshausen, Wed Apr 27 14:29:26 2022 +1000)
* cdf72b05 - feat: forward provider details to plugin when verifying (Ronald Holshausen, Fri Apr 22 14:12:34 2022 +1000)
* 2395143a - feat: forward verification to plugin for transports provided by the plugin (Ronald Holshausen, Fri Apr 22 12:02:05 2022 +1000)
* 1e8ae855 - chore: Upgrade pact_mock_server to 0.8.9 (Ronald Holshausen, Wed Apr 13 15:49:03 2022 +1000)
* 0df06dd2 - chore: Upgrade pact_matching to 0.12.5 (Ronald Holshausen, Wed Apr 13 15:38:49 2022 +1000)
* d043f6c7 - chore: upgrade pact_models to 0.3.3 (Ronald Holshausen, Wed Apr 13 15:24:33 2022 +1000)
* 73ae0ef0 - fix: Upgrade reqwest to 0.11.10 to resolve #156 (Ronald Holshausen, Wed Apr 13 13:31:55 2022 +1000)
* 89027c87 - chore: update pact_matching (0.12.4) and pact_mock_server (0.8.8) (Ronald Holshausen, Thu Mar 24 14:09:45 2022 +1100)
* 345b0011 - feat: support mock servers provided from plugins (Ronald Holshausen, Mon Mar 21 15:59:46 2022 +1100)
* eb92d665 - fix: rust/pact_mock_server_cli/Dockerfile to reduce vulnerabilities (snyk-bot, Thu Mar 17 22:20:08 2022 +0000)
* 5a4a8a1c - chore: update pact_mock_server to 0.8.7 (Ronald Holshausen, Fri Mar 4 14:24:23 2022 +1100)
* 8894fdfd - chore: update pact_matching to 0.12.3 (Ronald Holshausen, Fri Mar 4 14:09:17 2022 +1100)
* 8e864502 - chore: update all dependencies (Ronald Holshausen, Fri Mar 4 13:29:59 2022 +1100)
* d1bdd132 - chore: use docker builder image that supports Rust 2021 (Ronald Holshausen, Tue Jan 25 11:46:54 2022 +1100)
* e22067be - bump version to 0.7.8 (Ronald Holshausen, Tue Jan 25 11:17:43 2022 +1100)

# 0.7.7 - Maintenance Release

* 10c9b842 - chore: Upgrade pact_mock_server to 0.8.6 (Ronald Holshausen, Mon Jan 17 16:57:31 2022 +1100)
* 5e4c68ef - chore: update pact matching to 0.12.2 (Ronald Holshausen, Mon Jan 17 16:29:21 2022 +1100)
* 4f1ecff2 - chore: Upgrade pact-models to 0.2.7 (Ronald Holshausen, Mon Jan 17 10:53:26 2022 +1100)
* c2089645 - fix: log crate version must be fixed across all crates (including plugin driver) (Ronald Holshausen, Fri Jan 14 16:10:50 2022 +1100)
* 0a6e7d9d - refactor: Convert MatchingContext to a trait and use DocPath instead of string slices (Ronald Holshausen, Wed Dec 29 14:24:39 2021 +1100)
* 4d088317 - chore: Update pact_mock_server crate to 0.8.4 (Ronald Holshausen, Thu Dec 23 13:24:15 2021 +1100)
* 52bc1735 - chore: update pact_matching crate to 0.11.5 (Ronald Holshausen, Thu Dec 23 13:12:08 2021 +1100)
* 5479a634 - chore: Update pact_models (0.2.4) and pact-plugin-driver (0.0.14) (Ronald Holshausen, Thu Dec 23 12:57:02 2021 +1100)
* fc0a8360 - chore: update pact_matching to 0.11.4 (Ronald Holshausen, Mon Dec 20 12:19:36 2021 +1100)
* 631167fa - chore: update to latest mock server crate (Ronald Holshausen, Wed Nov 17 15:13:32 2021 +1100)
* 5d4a09c6 - feat: store the pact specification version with the mock server (Ronald Holshausen, Wed Nov 17 14:46:56 2021 +1100)
* 5d974c4a - chore: update to latest models and plugin driver crates (Ronald Holshausen, Tue Nov 16 11:56:53 2021 +1100)
* fc4580b8 - chore: drop beta from pact_mock_server version (Ronald Holshausen, Thu Nov 4 15:28:51 2021 +1100)
* bd2bd0ec - chore: drop beta from pact_matching version (Ronald Holshausen, Wed Nov 3 13:28:35 2021 +1100)
* 296b4370 - chore: update project to Rust 2021 edition (Ronald Holshausen, Fri Oct 22 10:44:48 2021 +1100)
* a561f883 - chore: use the non-beta models crate (Ronald Holshausen, Thu Oct 21 18:10:27 2021 +1100)
* a3d321cb - chore: update to latest mock server crate (Ronald Holshausen, Tue Oct 19 17:28:24 2021 +1100)
* 46a404c0 - chore: update to latest pact matching crate (Ronald Holshausen, Tue Oct 19 17:20:27 2021 +1100)
* 918e5beb - fix: update to latest models and plugin driver crates (Ronald Holshausen, Tue Oct 19 17:09:48 2021 +1100)
* 3819522d - chore: update to the latest matching and mock server crates (Ronald Holshausen, Tue Oct 19 11:34:18 2021 +1100)
* df386c8a - chore: use the published version of pact-plugin-driver (Ronald Holshausen, Mon Oct 18 13:41:36 2021 +1100)
* 1dc6f543 - chore: bump pact_mock_server version (Ronald Holshausen, Tue Oct 12 16:36:51 2021 +1100)
* 9bbbb52e - chore: bump pact matching crate version (Ronald Holshausen, Tue Oct 12 16:24:01 2021 +1100)
* 1eb37c13 - chore: use the published version of the models crate (Ronald Holshausen, Thu Oct 7 10:49:11 2021 +1100)
* ddc64246 - chore: use the published version of the models crate (Ronald Holshausen, Wed Oct 6 12:40:52 2021 +1100)
* 6d23796f - feat(plugins): support each key and each value matchers (Ronald Holshausen, Wed Sep 29 11:10:46 2021 +1000)
* 6f20282d - Merge branch 'master' into feat/plugins (Ronald Holshausen, Tue Sep 28 14:51:34 2021 +1000)
* 5599fe38 - bump version to 0.7.7 (Ronald Holshausen, Tue Sep 28 13:48:12 2021 +1000)
* b71dcabf - refactor(plugins): rename ContentTypeOverride -> ContentTypeHint (Ronald Holshausen, Tue Sep 14 15:08:52 2021 +1000)
* 03ebe632 - Merge branch 'master' into feat/plugins (Ronald Holshausen, Mon Sep 13 12:01:13 2021 +1000)
* fd6f8f40 - chore: Bump pact_mock_server version to 0.8.0-beta.0 (Ronald Holshausen, Mon Sep 13 11:46:11 2021 +1000)
* 716809f6 - chore: Get CI build passing (Ronald Holshausen, Fri Sep 10 14:55:46 2021 +1000)
* ceb1c35f - Merge branch 'master' into feat/plugins (Ronald Holshausen, Tue Sep 7 10:07:45 2021 +1000)
* b9aa7ecb - feat(Plugins): allow plugins to override text/binary format of the interaction content (Ronald Holshausen, Mon Aug 30 10:48:04 2021 +1000)
* 0c5cede2 - chore: bump models crate to 0.2 (Ronald Holshausen, Mon Aug 23 12:56:14 2021 +1000)

# 0.7.6 - support native TLS certs

* df715cd5 - feat: support native TLS. Fixes #144 (Matt Fellows, Mon Sep 20 13:00:33 2021 +1000)
* 971b980e - chore: fix clippy warnings (Ronald Holshausen, Fri Sep 10 17:31:16 2021 +1000)
* 9baa714d - chore: bump minor version of matching crate (Ronald Holshausen, Fri Jul 23 14:03:20 2021 +1000)
* 533c9e1f - chore: bump minor version of the Pact models crate (Ronald Holshausen, Fri Jul 23 13:15:32 2021 +1000)
* 47177971 - Revert "chore: pin the pact_mock_server_cli dependencies" (Ronald Holshausen, Wed Jul 21 15:53:47 2021 +1000)
* 3dbb2b1c - bump version to 0.7.6 (Ronald Holshausen, Wed Jul 21 15:49:42 2021 +1000)

# 0.7.5 - Bugfix Release

* 15e5d587 - chore: pin the pact_mock_server_cli dependencies (Ronald Holshausen, Wed Jul 21 15:14:08 2021 +1000)
* c958c24a - Revert "update changelog for release 0.7.5" (Ronald Holshausen, Wed Jul 21 15:07:34 2021 +1000)
* 4193beb8 - update changelog for release 0.7.5 (Ronald Holshausen, Wed Jul 21 14:01:09 2021 +1000)
* 3dccf866 - refacfor: moved the pact structs to the models crate (Ronald Holshausen, Sun Jul 18 16:58:14 2021 +1000)
* ed73b98a - chore: fix compiler warnings (Ronald Holshausen, Wed Jul 7 13:54:53 2021 +1000)
* 5c670814 - refactor: move expression_parser to pact_models crate (Ronald Holshausen, Fri Jun 11 10:51:51 2021 +1000)
* 6932c6d5 - Revert "chore: remove unused imports" (Ronald Holshausen, Sat Jun 5 16:26:52 2021 +1000)
* b4e26844 - fix: reqwest is dyn linked to openssl by default, which causes a SIGSEGV on alpine linux (Ronald Holshausen, Tue Jun 1 14:21:31 2021 +1000)
* 913b7b17 - chore: correct CLI docker release files (Ronald Holshausen, Tue Jun 1 11:25:28 2021 +1000)
* 13221ad9 - chore: Mock server CLI release build was overwriting the Windows exe with the SHA checksum (Ronald Holshausen, Sun May 30 18:30:44 2021 +1000)
* 11885733 - bump version to 0.7.5 (Ronald Holshausen, Sun May 30 18:08:28 2021 +1000)

# 0.7.4 - Upgraded crates + V4 features

* 62a653c - chore: remove unused imports (Matt Fellows, Thu May 27 23:40:27 2021 +1000)
* 4224088 - chore: add shasums to all release artifacts (Matt Fellows, Wed May 5 15:18:31 2021 +1000)
* 735c9e7 - chore: bump pact_matching to 0.9 (Ronald Holshausen, Sun Apr 25 13:50:18 2021 +1000)
* fb373b4 - chore: bump version to 0.0.2 (Ronald Holshausen, Sun Apr 25 13:40:52 2021 +1000)
* d010630 - chore: cleanup deprecation and compiler warnings (Ronald Holshausen, Sun Apr 25 12:23:30 2021 +1000)
* a725ab1 - feat(V4): added synchronous request/response message formats (Ronald Holshausen, Sat Apr 24 16:05:12 2021 +1000)
* 728465d - fix: clippy violation - caused a compiler error (Ronald Holshausen, Sat Apr 24 13:07:32 2021 +1000)
* e3d48a0 - chore: cleanup some clippy voilations (Ronald Holshausen, Sat Apr 24 12:57:14 2021 +1000)
* 80b7148 - feat(V4): Updated consumer DSL to set comments + mock server initial support for V4 pacts (Ronald Holshausen, Fri Apr 23 17:58:10 2021 +1000)
* 220fb5e - refactor: move the PactSpecification enum to the pact_models crate (Ronald Holshausen, Thu Apr 22 11:18:26 2021 +1000)
* 9976e80 - feat: added read locks and a mutex guard to reading and writing pacts (Ronald Holshausen, Mon Feb 8 11:58:52 2021 +1100)
* 49a3cf2 - refactor: use bytes crate instead of vector of bytes for body content (Ronald Holshausen, Sun Feb 7 14:43:40 2021 +1100)
* 4afa86a - fix: add callback timeout option for verifcation callbacks (Ronald Holshausen, Sat Feb 6 12:27:32 2021 +1100)
* e43fdb8 - chore: upgrade maplit, itertools (Audun Halland, Mon Jan 11 05:30:10 2021 +0100)
* 5e5c320 - chore: upgrade rand, rand_regex (Audun Halland, Sat Jan 9 09:33:13 2021 +0100)
* 4a70bef - chore: upgrade expectest to 0.12 (Audun Halland, Sat Jan 9 11:29:29 2021 +0100)
* 3a28a6c - chore: upgrade regex, chrono-tz (Audun Halland, Sat Jan 9 11:12:49 2021 +0100)
* afeb679 - chore: upgrade simplelog (Audun Halland, Sat Jan 9 10:55:08 2021 +0100)
* 9a8a63f - chore: upgrade quickcheck (Audun Halland, Sat Jan 9 08:46:51 2021 +0100)
* 39fc84d - chore: upgrade http to get rid of bytes 0.5.6 (Audun Halland, Sat Jan 9 07:18:50 2021 +0100)
* 3a6945e - chore: Upgrade reqwest to 0.11 and hence tokio to 1.0 (Ronald Holshausen, Wed Jan 6 15:34:47 2021 +1100)
* 3bef361 - chore: add apt clean to docker file (Ronald Holshausen, Tue Jan 5 13:31:31 2021 +1100)
* b9ba322 - bump version to 0.7.4 (Ronald Holshausen, Tue Jan 5 13:13:50 2021 +1100)

# 0.7.3 - Add TLS (self-signed) option

* 773b4b1 - fix: pinning version of webmachine until reqwest is updated (Ronald Holshausen, Tue Jan 5 12:41:05 2021 +1100)
* 76f052b - feat: add self-signed tls option to mockserver cli (to test TLS with Tokio 1.0) (Ronald Holshausen, Tue Jan 5 11:39:53 2021 +1100)
* 3d531b3 - bump version to 0.7.3 (Ronald Holshausen, Thu Dec 31 13:11:40 2020 +1100)
* d85e9ee - chore: correct changelog (Ronald Holshausen, Thu Dec 31 13:06:58 2020 +1100)

# 0.7.2 - support generators associated with array contains matcher variants

* 56a13d3: update pact_matching and pact_mock_server crates to latest
* bfba4bd - bump version to 0.7.2 (Ronald Holshausen, Fri Oct 16 11:50:58 2020 +1100)

# 0.7.1 - arrayContains matcher + text/xml content type

* 2fb0c6e - fix: fix the build after refactoring the pact write function (Ronald Holshausen, Wed Oct 14 11:07:57 2020 +1100)
* 7fbc731 - chore: bump minor version of matching lib (Ronald Holshausen, Fri Oct 9 10:42:33 2020 +1100)
* 7232e89 - feat: Add initial V4 models and example pact files (Ronald Holshausen, Tue Oct 6 09:13:21 2020 +1100)
* eb0389c - bump version to 0.7.1 (Ronald Holshausen, Mon Sep 28 12:17:05 2020 +1000)

# 0.7.0 - Async changes (using Hyper 0.13) + CORS pre-flight support

* 7e68e4c - feat: enable CORS behaviour based on the mock server config (Ronald Holshausen, Mon Sep 28 11:42:23 2020 +1000)
* 4eb9188 - chore: cleanup warnings (Ronald Holshausen, Mon Sep 28 10:30:42 2020 +1000)
* d0b84f9 - refactor: remove the use of lazy_static (Ronald Holshausen, Sun Sep 27 17:08:58 2020 +1000)
* bdbfccc - refactor: update mock server CLI to be async (Ronald Holshausen, Sun Sep 27 13:12:51 2020 +1000)
* 7fd4dd2 - refactor: update the mock server CLI to use webmachine 0.2 and hyper 0.13 (Ronald Holshausen, Sun Sep 27 09:39:23 2020 +1000)
* 29ba743 - feat: add a mock server config struct (Ronald Holshausen, Thu Sep 24 10:30:59 2020 +1000)
* 2e662a6 - feat: handle CORS pre-flight requests in the mock server (Ronald Holshausen, Wed Sep 23 17:59:32 2020 +1000)
* 2d44ffd - chore: bump minor version of the matching crate (Ronald Holshausen, Mon Sep 14 12:06:37 2020 +1000)
* a05bcbb - refactor: renamed Pact to RequestResponsePact (Ronald Holshausen, Sun Sep 13 12:45:34 2020 +1000)
* ed207a7 - chore: updated readmes for docs site (Ronald Holshausen, Sun Jun 28 10:04:09 2020 +1000)
* 56258d7 - bump version to 0.6.3 (Ronald Holshausen, Wed Jun 24 11:01:06 2020 +1000)

# 0.6.2 - Updated XML Matching

* 218239c - chore: update to latest matching crate (Ronald Holshausen, Wed Jun 24 10:53:35 2020 +1000)

# 0.6.1 - Updated crates

* 98d7abb - chore: update GH action to build pact_mock_server_cli (Ronald Holshausen, Wed May 27 14:55:14 2020 +1000)
* bea787c - chore: bump matching crate version to 0.6.0 (Ronald Holshausen, Sat May 23 17:56:04 2020 +1000)
* 411f697 - chore: correct some clippy warnings (Ronald Holshausen, Wed Apr 29 16:49:36 2020 +1000)
* f84e672 - chore: update mock server library to latest (Ronald Holshausen, Fri Apr 24 11:00:34 2020 +1000)
* 43de9c3 - chore: update matching library to latest (Ronald Holshausen, Fri Apr 24 10:20:55 2020 +1000)
* 1651af1 - fix: upgrade uuid crate (Ronald Holshausen, Thu Apr 23 14:56:34 2020 +1000)
* d457221 - chore: update dependant crates to use mock server lib 0.7.0 (Ronald Holshausen, Sun Jan 19 11:31:21 2020 +1100)
* 8a0c5c2 - fix: docker file needs to be able to build Oniguruma lib (Ronald Holshausen, Sat Dec 14 19:23:45 2019 +1100)
* e1a0f16 - bump version to 0.6.1 (Ronald Holshausen, Sat Dec 14 17:32:50 2019 +1100)

# 0.6.0 - Bugfix Release

* d2908af - chore: bump minor version (Ronald Holshausen, Sat Dec 14 17:15:41 2019 +1100)
* 2d95535 - pact_mock_server_cli: Remove extern crate from main.rs (Audun Halland, Sun Nov 17 23:10:10 2019 +0100)
* abc2a36 - pact_mock_server_cli: Upgrade log, simplelog (Audun Halland, Sun Nov 17 23:01:50 2019 +0100)
* 713cd6a - Explicit edition 2018 in Cargo.toml files (Audun Halland, Sat Nov 16 23:55:37 2019 +0100)
* 924452f - 2018 edition autofix "cargo fix --edition" (Audun Halland, Sat Nov 16 22:27:42 2019 +0100)
* 097d045 - refactor: added a mock server ffi module and bumped the mock server minor version (Ronald Holshausen, Sat Sep 7 09:39:27 2019 +1000)
* f79b033 - chore: update terminal support in release scripts (Ronald Holshausen, Sat Aug 24 12:25:28 2019 +1000)
* da1956a - chore: bump the version of the matching lib (Ronald Holshausen, Sat Aug 24 12:06:51 2019 +1000)
* c5e55ab - bump version to 0.5.2 (Ronald Holshausen, Sat Aug 24 11:29:20 2019 +1000)

# 0.5.1 - support headers with multiple values

* 5b22076 - fix: docker release script (Ronald Holshausen, Sat Jul 27 16:52:12 2019 +1000)
* 2e59235 - bump version to 0.5.1 (Ronald Holshausen, Sat Jul 27 16:36:51 2019 +1000)

# 0.5.0 - Upgrade to non-blocking Hyper 0.12

* d842100 - chore: bump component versions to 0.5.0 (Ronald Holshausen, Sat Jul 27 15:44:51 2019 +1000)
* 2826bb0 - Make pact_mock_server_cli use ServerManager (Audun Halland, Tue Jul 23 01:40:46 2019 +0200)
* 4df2797 - Rename API function again (Audun Halland, Mon Jul 22 23:38:11 2019 +0200)
* f8fa0d8 - chore: Bump pact matchig version to 0.5.0 (Ronald Holshausen, Sat Jan 5 19:25:53 2019 +1100)
* 074569a - feat: Add a parameter for the server key to the start command #26 (Ronald Holshausen, Sun Apr 8 18:24:36 2018 +1000)
* 40ad75b - feat: Add a command to shut the master mock server down #26 (Ronald Holshausen, Sun Apr 8 18:15:08 2018 +1000)
* e5af1b0 - fix: global options no longer incorrectly display a warning about being provided twice #27 (Ronald Holshausen, Sun Apr 8 16:11:41 2018 +1000)
* 3c33294 - fix: Only print errors in the CLI to STDERR #28 (Ronald Holshausen, Sun Apr 8 15:57:56 2018 +1000)
* 386ab52 - fix: corrected the release scripts to check for a version parameter (Ronald Holshausen, Sun Apr 8 13:44:57 2018 +1000)
* 6c2d6c8 - chore: added docker release scripts for the CLIs (Ronald Holshausen, Sun Apr 8 13:44:18 2018 +1000)
* a45d5f8 - fix: corrected the docker build for the mock server cli #14 (Ronald Holshausen, Sun Apr 8 12:52:53 2018 +1000)
* 6343607 - fix: CLI was reporting incorrect pact specification version (Ronald Holshausen, Sun Apr 8 12:36:56 2018 +1000)
* 9ea039f - bump version to 0.4.1 (Ronald Holshausen, Sat Apr 7 14:55:59 2018 +1000)

# 0.4.0 - First V3 specification release

* 398edaf - Upgrade UUID library to latest (Ronald Holshausen, Sat Apr 7 12:29:58 2018 +1000)
* 7fef36b - Merge branch 'v2-spec' into v3-spec (Ronald Holshausen, Sat Nov 4 12:49:07 2017 +1100)
* a306b12 - bump version to 0.3.2 (Ronald Holshausen, Fri Nov 3 14:07:07 2017 +1100)
* 940a0e3 - Reverted hyper to 0.9.x (Ronald Holshausen, Sun Oct 22 12:01:17 2017 +1100)
* fbe35d8 - Compiling after merge from v2-spec (Ronald Holshausen, Sun Oct 22 11:39:46 2017 +1100)
* 00dc75a - Bump version to 0.4.0 (Ronald Holshausen, Sun Oct 22 10:46:48 2017 +1100)
* 184127a - Merge branch 'v2-spec' into v3-spec (Ronald Holshausen, Sun Oct 22 10:32:31 2017 +1100)
* e82ee08 - Merge branch 'v2-spec' into v3-spec (Ronald Holshausen, Mon Oct 16 09:24:11 2017 +1100)
* 64ff667 - Upgraded the mock server implemenation to use Hyper 0.11.2 (Ronald Holshausen, Wed Sep 6 12:56:47 2017 +1000)
* e5a93f3 - Merge branch 'master' into v3-spec (Ronald Holshausen, Sun Aug 20 09:53:48 2017 +1000)
* 639ac22 - fixes after merge in from master (Ronald Holshausen, Sun Oct 23 10:45:54 2016 +1100)
* 49e45f7 - Merge branch 'master' into v3-spec (Ronald Holshausen, Sun Oct 23 10:10:40 2016 +1100)
* a7533dc - updated the mockserver lib and cli to generate V3 pacts (Ronald Holshausen, Thu Aug 4 22:13:20 2016 +1000)
* 539eb48 - updated all the readmes and cargo manefests for v3 (Ronald Holshausen, Tue Jul 19 15:46:18 2016 +1000)

# 0.3.1 - Bugfixes plus changes for running with docker

* cdf01f3 - Add a docker file for the pact mock server cli (Ronald Holshausen, Fri Nov 3 11:51:01 2017 +1100)
* a56b6a6 - Change the column heading to verification state in the mock server list output #24 (Ronald Holshausen, Sun Oct 22 15:15:30 2017 +1100)
* 814fe12 - Modify AssafKatz3's implementation to scan for next available port from a base port number #15 (Ronald Holshausen, Sun Oct 22 14:40:13 2017 +1100)
* 37abe19 - Pulled in changes from https://github.com/AssafKatz3/pact-reference.git #14 (Assaf Katz, Mon Sep 25 12:28:17 2017 +0300)
* 9cda328 - bump version to 0.3.1 (Ronald Holshausen, Fri Oct 20 11:01:04 2017 +1100)

# 0.3.0 - Backported the matching rules from the V3 branch

* c8595cc - Correct the paths in the release scripts for pact_mock_server_cli (Ronald Holshausen, Fri Oct 20 10:48:03 2017 +1100)
* ac94388 - Tests are now all passing #20 (Ronald Holshausen, Thu Oct 19 15:14:25 2017 +1100)
* c983c63 - Bump versions to 0.3.0 (Ronald Holshausen, Wed Oct 18 13:54:46 2017 +1100)
* 06e92e5 - Refer to local libs using version+paths (Eric Kidd, Tue Oct 3 06:22:23 2017 -0400)
* 7afd258 - Update all the cargo manifest versions and commit the cargo lock files (Ronald Holshausen, Wed May 17 10:37:44 2017 +1000)
* be8c299 - Cleanup unused BTreeMap usages and use remote pact dependencies (Anthony Damtsis, Mon May 15 17:09:14 2017 +1000)
* a59fb98 - Migrate remaining pact modules over to serde (Anthony Damtsis, Mon May 15 16:59:04 2017 +1000)
* c5f9c27 - bump version to 0.2.4 (Ronald Holshausen, Sun Apr 23 17:39:49 2017 +1000)

# 0.2.3 - Bugfix Release

* 224ad98 - Change no-console-log to no-term-log and use a simple logger if it is set #6 (Ronald Holshausen, Sun Apr 23 17:19:53 2017 +1000)
* cec2358 - bump version to 0.2.3 (Ronald Holshausen, Fri Apr 21 14:33:13 2017 +1000)

# 0.2.2 - Bugfix Release

* 53074cf - Merge branch 'v1.1-spec' into v2-spec (Ronald Holshausen, Fri Apr 21 14:17:05 2017 +1000)
* 01fa713 - bump version to 0.1.3 (Ronald Holshausen, Fri Apr 21 14:08:58 2017 +1000)
* e4b59e5 - update changelog for release 0.1.2 (Ronald Holshausen, Fri Apr 21 14:05:14 2017 +1000)
* 07b1827 - Merge branch 'v1-spec' into v1.1-spec (Ronald Holshausen, Fri Apr 21 13:39:50 2017 +1000)
* da4e32f - bump version to 0.0.3 (Ronald Holshausen, Fri Apr 21 13:31:18 2017 +1000)
* 9b4b5fb - update changelog for release 0.0.2 (Ronald Holshausen, Fri Apr 21 13:27:54 2017 +1000)
* 2276cd0 - upgraded simple log crate and added cli options to disable file or console logging #6 (Ronald Holshausen, Fri Apr 21 13:15:27 2017 +1000)
* ea5cec8 - bump version to 0.2.2 (Ronald Holshausen, Sun Oct 9 16:43:59 2016 +1100)
* 0b83b06 - correct the displayed help for the pact_mock_server_cli (Ronald Holshausen, Sat Oct 8 17:29:19 2016 +1100)

# 0.1.2 - Bugfix Release

* 07b1827 - Merge branch 'v1-spec' into v1.1-spec (Ronald Holshausen, Fri Apr 21 13:39:50 2017 +1000)
* da4e32f - bump version to 0.0.3 (Ronald Holshausen, Fri Apr 21 13:31:18 2017 +1000)
* 9b4b5fb - update changelog for release 0.0.2 (Ronald Holshausen, Fri Apr 21 13:27:54 2017 +1000)
* 2276cd0 - upgraded simple log crate and added cli options to disable file or console logging #6 (Ronald Holshausen, Fri Apr 21 13:15:27 2017 +1000)
* 91d1216 - bump version to 0.1.2 (Ronald Holshausen, Sat Oct 8 17:49:20 2016 +1100)
* 0b83b06 - correct the displayed help for the pact_mock_server_cli (Ronald Holshausen, Sat Oct 8 17:29:19 2016 +1100)

# 0.0.2 - Bugfix Release

* 2276cd0 - upgraded simple log crate and added cli options to disable file or console logging #6 (Ronald Holshausen, Fri Apr 21 13:15:27 2017 +1000)
* 0b83b06 - correct the displayed help for the pact_mock_server_cli (Ronald Holshausen, Sat Oct 8 17:29:19 2016 +1100)
* 04d9e5f - update the docs for the pact consumer library (Ronald Holshausen, Mon Sep 26 23:06:19 2016 +1000)
* 40c9e02 - exclude IntelliJ files from publishing (Ronald Holshausen, Mon Sep 26 21:22:35 2016 +1000)
* c1d97a0 - correct the repository paths in the cargo manifests (Ronald Holshausen, Tue Jun 28 14:52:46 2016 +1000)
* 91d6d62 - removed the v1 from the project path, will use a git branch instead (Ronald Holshausen, Mon Jun 27 22:09:32 2016 +1000)

# 0.2.1 - Changes required for verifying V2 pacts

* e3eebbd -  update projects to use the published pact mock server library (Ronald Holshausen, Sun Oct 9 16:36:25 2016 +1100)
* 770010a - update projects to use the published pact matching lib (Ronald Holshausen, Sun Oct 9 16:25:15 2016 +1100)
* a21973a - Get the build passing after merge from V1.1 branch (Ronald Holshausen, Sun Oct 9 13:47:09 2016 +1100)
* 341607c - Merge branch 'v1.1-spec' into v2-spec (Ronald Holshausen, Sun Oct 9 12:10:12 2016 +1100)
* 91d1216 - bump version to 0.1.2 (Ronald Holshausen, Sat Oct 8 17:49:20 2016 +1100)
* 0d324a8 - bump version to 0.2.1 (Ronald Holshausen, Wed Jul 13 14:26:40 2016 +1000)
* 377b372 - update changelog for release 0.2.0 (Ronald Holshausen, Wed Jul 13 14:22:15 2016 +1000)
* 7ed156e - updated project for the V2 spec release (Ronald Holshausen, Wed Jul 13 14:19:12 2016 +1000)
* 22b0bb9 - fix for failing build (Ronald Holshausen, Tue Jul 12 16:59:56 2016 +1000)
* 534e7a1 - updated readmes and bump versions for the V2 implementation (Ronald Holshausen, Wed Jun 29 10:38:32 2016 +1000)

# 0.1.1 - Changes required for verifying V1.1 pacts

* 28928ef - correct the displayed help for the pact_mock_server_cli (Ronald Holshausen, Sat Oct 8 17:29:19 2016 +1100)
* 3ca2df8 - update dependencies (Ronald Holshausen, Sat Oct 8 17:22:48 2016 +1100)
* a46dabb - update all references to V1 spec after merge (Ronald Holshausen, Sat Oct 8 16:20:51 2016 +1100)
* 1d6d4f8 - Merge branch 'v1-spec' into v1.1-spec (Ronald Holshausen, Sat Oct 8 15:44:25 2016 +1100)
* 04d9e5f - update the docs for the pact consumer library (Ronald Holshausen, Mon Sep 26 23:06:19 2016 +1000)
* 40c9e02 - exclude IntelliJ files from publishing (Ronald Holshausen, Mon Sep 26 21:22:35 2016 +1000)
* efe036c - bump version to 0.1.1 (Ronald Holshausen, Tue Jun 28 21:54:59 2016 +1000)
* c1d97a0 - correct the repository paths in the cargo manifests (Ronald Holshausen, Tue Jun 28 14:52:46 2016 +1000)

# 0.2.0 - V2 Specification Implementation

* 7ed156e - updated project for the V2 spec release (Ronald Holshausen, Wed Jul 13 14:19:12 2016 +1000)
* 22b0bb9 - fix for failing build (Ronald Holshausen, Tue Jul 12 16:59:56 2016 +1000)
* 534e7a1 - updated readmes and bump versions for the V2 implementation (Ronald Holshausen, Wed Jun 29 10:38:32 2016 +1000)
* efe036c - bump version to 0.1.1 (Ronald Holshausen, Tue Jun 28 21:54:59 2016 +1000)

# 0.1.0 - V1.1 Specification Implementation

* f91bb6e - use the published versions of the matching and mock server libraries (Ronald Holshausen, Tue Jun 28 21:38:21 2016 +1000)
* 140526d - Implement V1.1 matching (Ronald Holshausen, Tue Jun 28 15:58:35 2016 +1000)
* 4224875 - update readmes and bump versions for V1.1 implementation (Ronald Holshausen, Tue Jun 28 15:05:39 2016 +1000)
* 91d6d62 - removed the v1 from the project path, will use a git branch instead (Ronald Holshausen, Mon Jun 27 22:09:32 2016 +1000)

# 0.0.1 - Feature Release

* 18c009b - added changelog (Ronald Holshausen, Mon Jun 27 19:42:26 2016 +1000)
* 78126ab - no point publishing the rust docs as pact_mock_server_cli is not a library (Ronald Holshausen, Mon Jun 27 19:38:56 2016 +1000)
* 8867836 - correct the release script (Ronald Holshausen, Mon Jun 27 19:36:46 2016 +1000)
* aa2d2dd - added release script for pact_mock_server_cli (Ronald Holshausen, Mon Jun 27 17:20:38 2016 +1000)
* 2a78f40 - updated the README for the pact_mock_server_cli (Ronald Holshausen, Mon Jun 27 17:01:16 2016 +1000)
* 3f77f3f - update pact_mock_server_cli to depend on libpact_mock_server from crates.io (Ronald Holshausen, Mon Jun 27 15:50:15 2016 +1000)
* 3b6bf66 - fix the project deps for the travis build (Ronald Holshausen, Mon Jun 27 14:46:19 2016 +1000)
* f7d9960 - implemented the shutdown mock server command (Ronald Holshausen, Sun Jun 26 15:05:40 2016 +1000)
* f91b9fd - compile against the published webmachine crate (Ronald Holshausen, Sun Jun 26 13:14:34 2016 +1000)
* b7635b8 - correctly handle the status codes from the master mock server (Ronald Holshausen, Sun Jun 26 10:49:47 2016 +1000)
* 6234bbd - implemented delete on the master server to shut a mock server down (Ronald Holshausen, Sat Jun 25 16:59:39 2016 +1000)
* ec23a8b - use a Hyper Handler instead of a closure as it is easier to be thread safe (Ronald Holshausen, Fri Jun 24 16:30:28 2016 +1000)
* dd850bc - Got POST to main resource working with webmachine (Ronald Holshausen, Thu Jun 23 13:01:25 2016 +1000)
* b5b41ee - got GET to main resource working with webmachine (Ronald Holshausen, Thu Jun 23 11:30:10 2016 +1000)
* 079fdd4 - correct the webmachine-rust reference (Ronald Holshausen, Thu Jun 16 19:35:39 2016 +1000)
* 4c60f07 - replace rustful with webmachine (Ronald Holshausen, Thu Jun 16 17:31:11 2016 +1000)
* 44daccc - add an optional port number to start the mock server with (Ronald Holshausen, Wed Jun 15 12:40:51 2016 +1000)
* 0cfc690 - add the webmachine project as a dependency (Ronald Holshausen, Thu Jun 9 22:26:16 2016 +1000)
* 7dc4b52 - implemented merging of pact files when writing (Ronald Holshausen, Thu Jun 9 17:34:02 2016 +1000)
* 34fd827 - implement a write_pact exported function to the mock server library (Ronald Holshausen, Thu Jun 9 12:15:01 2016 +1000)
* dcde5dc - add a newline at the end of the help for people with crazy terminal settings (Ronald Holshausen, Thu Jun 9 11:12:16 2016 +1000)
* 511d7a1 - bump version of pact mock server cli (Ronald Holshausen, Wed Jun 8 20:27:53 2016 +1000)
* 5157386 - add rustdoc comment to the cli main file (Ronald Holshausen, Wed Jun 8 20:01:12 2016 +1000)


# 0.0.0 - First Release
