To generate the log, run `git log --pretty='* %h - %s (%an, %ad)' TAGNAME..HEAD .` replacing TAGNAME and HEAD as appropriate.

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
