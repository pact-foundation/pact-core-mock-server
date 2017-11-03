To generate the log, run `git log --pretty='* %h - %s (%an, %ad)' TAGNAME..HEAD .` replacing TAGNAME and HEAD as appropriate.

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
