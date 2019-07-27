To generate the log, run `git log --pretty='* %h - %s (%an, %ad)' TAGNAME..HEAD .` replacing TAGNAME and HEAD as appropriate.

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
