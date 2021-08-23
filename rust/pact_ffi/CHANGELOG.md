To generate the log, run `git log --pretty='* %h - %s (%an, %ad)' TAGNAME..HEAD .` replacing TAGNAME and HEAD as appropriate.

# 0.0.2 - Bugfix Release

* 9370327c - feat(FFI): Added initial verifier FFI prototype (Ronald Holshausen, Sun Aug 22 15:01:17 2021 +1000)
* c274ca1a - fix: use the pacts for verification endpoint if the conusmer selectors are specified #133 (Ronald Holshausen, Sun Aug 22 11:51:22 2021 +1000)
* 3215821e - chore: correct the OSX release (Ronald Holshausen, Tue Aug 17 12:47:20 2021 +1000)
* 64cf38e9 - bump version to 0.0.2 (Ronald Holshausen, Tue Aug 17 11:00:54 2021 +1000)

# 0.0.1 - M1 architecture support + Bugfixes

* a9940325 - chore: release m1 arm package for new Mac hardware (Matt Fellows, Wed Aug 11 22:57:46 2021 +1000)
* b5a7b779 - feat: support new selectors (Matt Fellows, Mon Aug 9 13:27:33 2021 +1000)
* 8bcd1c7e - fix: min/max type matchers must not apply the limits when cascading (Ronald Holshausen, Sun Aug 8 15:50:40 2021 +1000)
* 738e9961 - chore: generate both a C and C++ header (Ronald Holshausen, Sun Aug 8 14:28:11 2021 +1000)
* 6124ed0b - refactor: Introduce DocPath struct for path expressions (Caleb Stepanian, Thu Jul 29 12:27:32 2021 -0400)
* 9baa714d - chore: bump minor version of matching crate (Ronald Holshausen, Fri Jul 23 14:03:20 2021 +1000)
* 533c9e1f - chore: bump minor version of the Pact models crate (Ronald Holshausen, Fri Jul 23 13:15:32 2021 +1000)
* 3dccf866 - refacfor: moved the pact structs to the models crate (Ronald Holshausen, Sun Jul 18 16:58:14 2021 +1000)
* 372cbdd1 - chore: remove CMake step from CI build (Ronald Holshausen, Sun Jul 18 15:57:30 2021 +1000)
* 996761a6 - chore: remove CMake step from CI build (Ronald Holshausen, Sun Jul 18 15:26:31 2021 +1000)
* 685a2df2 - chore: cmake is failing on CI to find cargo (Ronald Holshausen, Sun Jul 18 15:12:51 2021 +1000)
* e8046d84 - refactor: moved interaction structs to the models crate (Ronald Holshausen, Sun Jul 18 14:36:03 2021 +1000)
* 0c528ea0 - chore: update pact_ffi readme (Ronald Holshausen, Mon Jul 12 15:28:53 2021 +1000)
* 95c873ec - chore: correct conan packages (Ronald Holshausen, Mon Jul 12 11:18:44 2021 +1000)
* bd989b8f - chore: add homepage and repositry URL to FFI manifest (Ronald Holshausen, Mon Jul 12 10:25:36 2021 +1000)
* 5e725866 - chore: add conan publish to release script (Ronald Holshausen, Mon Jul 12 10:23:33 2021 +1000)
* 6d1ff318 - fix: conan packages for pact_ffi (Ronald Holshausen, Mon Jul 12 09:23:48 2021 +1000)
* d7079e43 - bump version to 0.0.1 (Ronald Holshausen, Sun Jul 11 17:47:18 2021 +1000)

# 0.0.0 - Initial Release
