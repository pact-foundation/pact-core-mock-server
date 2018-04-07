To generate the log, run `git log --pretty='* %h - %s (%an, %ad)' TAGNAME..HEAD .` replacing TAGNAME and HEAD as appropriate.

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
