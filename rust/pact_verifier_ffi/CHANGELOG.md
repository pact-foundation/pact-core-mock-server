To generate the log, run `git log --pretty='* %h - %s (%an, %ad)' TAGNAME..HEAD .` replacing TAGNAME and HEAD as appropriate.

# 0.0.5 - Bugfix Release

* 12e51704 - fix: linux verifier ffi shasum path was incorrect. Fixes #114 (Matt Fellows, Mon Jun 7 09:25:14 2021 +1000)
* e8d6d844 - fix: pact_verifier_cli was printing the version from the FFI crate (Ronald Holshausen, Sat Jun 5 14:43:38 2021 +1000)
* 2b7a415e - bump version to 0.0.5 (Ronald Holshausen, Sun May 30 18:40:13 2021 +1000)

# 0.0.4 - FFI enhancements

* a3f272b - Merge pull request #104 from pact-foundation/feat/consumer-version-selectors (Matt Fellows, Thu May 27 13:16:46 2021 +1000)
* b40b76f - wip: add validator to request timeout (Matt Fellows, Thu May 27 13:06:45 2021 +1000)
* f4a7d52 - fix: consumer version selectors (Matt Fellows, Thu May 27 09:04:20 2021 +1000)
* af6721a - feat: rename callback_timeout to request_timeout, and support timeouts for all http requests during verification (Matt Fellows, Thu May 27 09:04:05 2021 +1000)
* 100ae2d - wip: tidy up selectors code (Matt Fellows, Wed May 26 19:10:26 2021 +1000)
* 755333f - feat: support callback timeout option on verifier (Matt Fellows, Wed May 26 19:07:47 2021 +1000)
* 2649696 - feat: support env vars for filters (Matt Fellows, Wed May 26 19:07:05 2021 +1000)
* 904ca31 - feat: enable consumer version selectors (Matt Fellows, Wed May 26 19:05:29 2021 +1000)
* 61c9d0b - feat: return error code 4 when verifier receives invalid args (Matt Fellows, Wed May 26 15:09:09 2021 +1000)
* 4224088 - chore: add shasums to all release artifacts (Matt Fellows, Wed May 5 15:18:31 2021 +1000)
* 735c9e7 - chore: bump pact_matching to 0.9 (Ronald Holshausen, Sun Apr 25 13:50:18 2021 +1000)
* fb373b4 - chore: bump version to 0.0.2 (Ronald Holshausen, Sun Apr 25 13:40:52 2021 +1000)
* dfd3ba9 - chore: cleanup some clippy violations (Ronald Holshausen, Sun Apr 25 13:21:29 2021 +1000)
* 220fb5e - refactor: move the PactSpecification enum to the pact_models crate (Ronald Holshausen, Thu Apr 22 11:18:26 2021 +1000)

# 0.0.3 - Bugfix Release

* f4881db - feat: set non-hard coded install name on Mac dylib (Matt Fellows, Wed Feb 24 14:29:52 2021 +1100)
* dc64860 - bump version to 0.0.3 (Ronald Holshausen, Mon Feb 8 16:13:22 2021 +1100)

# 0.0.2 - add callback timeout option for verifcation callbacks

* 4afa86a - fix: add callback timeout option for verifcation callbacks (Ronald Holshausen, Sat Feb 6 12:27:32 2021 +1100)
* dccd16f - chore: wrap verifier callbacks in Arc<Self> so they can be called across threads (Ronald Holshausen, Tue Jan 26 16:24:09 2021 +1100)
* c8f7091 - feat: made pact broker module public so it can be used by other crates (Ronald Holshausen, Sun Jan 24 18:24:30 2021 +1100)
* 03c6969 - bump version to 0.0.2 (Ronald Holshausen, Mon Jan 11 10:36:23 2021 +1100)

# 0.0.1 - Updated dependencies

* 5e5c320 - chore: upgrade rand, rand_regex (Audun Halland, Sat Jan 9 09:33:13 2021 +0100)
* afeb679 - chore: upgrade simplelog (Audun Halland, Sat Jan 9 10:55:08 2021 +0100)
* 1ac3548 - chore: upgrade env_logger to 0.8 (Audun Halland, Sat Jan 9 09:50:27 2021 +0100)
* 9a8a63f - chore: upgrade quickcheck (Audun Halland, Sat Jan 9 08:46:51 2021 +0100)
* 3a6945e - chore: Upgrade reqwest to 0.11 and hence tokio to 1.0 (Ronald Holshausen, Wed Jan 6 15:34:47 2021 +1100)
* 9eb107a - Revert "Revert "chore: bump version to 0.0.1"" (Ronald Holshausen, Tue Jan 5 17:25:37 2021 +1100)

# 0.0.0 - Initial release

* d9f0e8b - refactor: split pact_verifier ffi functions into seperate crate (Ronald Holshausen, Tue Jan 5 16:17:46 2021 +1100)
