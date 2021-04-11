To generate the log, run `git log --pretty='* %h - %s (%an, %ad)' TAGNAME..HEAD .` replacing TAGNAME and HEAD as appropriate.

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
