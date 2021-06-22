To generate the log, run `git log --pretty='* %h - %s (%an, %ad)' TAGNAME..HEAD .` replacing TAGNAME and HEAD as appropriate.

# 0.0.4 - Refactor + Bugfixes

* ed3ae59a - chore: fix for failing parse_era test on CI (Ronald Holshausen, Tue Jun 22 14:55:46 2021 +1000)
* bbc638be - feat(pact file verification): verify consumer and provider sections (Ronald Holshausen, Fri Jun 18 16:52:15 2021 +1000)
* a7c071bc - feat(pact-file-validation): implemented validation of the metadata section (Ronald Holshausen, Wed Jun 16 09:17:28 2021 +1000)
* 00b65dcf - chore: rename pact_file_verifier -> pact_cli (Ronald Holshausen, Mon Jun 14 14:08:24 2021 +1000)
* 6198538d - refactor: move time_utils to pact_models crate (Ronald Holshausen, Fri Jun 11 12:58:26 2021 +1000)
* 5c670814 - refactor: move expression_parser to pact_models crate (Ronald Holshausen, Fri Jun 11 10:51:51 2021 +1000)
* 457aa5fc - fix(V4): Status code matcher was not converted to JSON correctly (Ronald Holshausen, Sun Jun 6 12:53:37 2021 +1000)
* b0ac7141 - feat: support graphql as a JSON content type (Ronald Holshausen, Sat Jun 5 15:14:06 2021 +1000)
* a44cbbee - fix: verifier was returning a mismatch when the expected body is empty #113 (Ronald Holshausen, Sat Jun 5 15:07:22 2021 +1000)
* 4e328d93 - feat: implement verification for RequestResponsePact, Consumer, Provider (Ronald Holshausen, Thu Jun 3 16:59:23 2021 +1000)
* 2f678213 - feat: initial prototype of a pact file verifier (Ronald Holshausen, Thu Jun 3 14:56:16 2021 +1000)

# 0.0.3 - Moved provider state models

* a7b81af - chore: fix clippy violation (Ronald Holshausen, Sat May 29 17:29:06 2021 +1000)
* 7022625 - refactor: move provider state models to the pact models crate (Ronald Holshausen, Sat May 29 17:18:48 2021 +1000)
* ef37cb9 - refactor(V4): extract common message parts into a seperate struct (Ronald Holshausen, Sat May 29 16:38:38 2021 +1000)
* ebb11df - feat(V4): fixed test _ refactored types for match functions (Ronald Holshausen, Sat May 29 14:56:31 2021 +1000)
* 73a53b8 - feat(V4): add an HTTP status code matcher (Ronald Holshausen, Fri May 28 18:40:11 2021 +1000)
* 8e8075b - refactor: move some more structs to the models crate (Ronald Holshausen, Thu May 27 14:34:03 2021 +1000)

# 0.0.2 - FFI support

* 82711d6 - chore: use a feature to enable FFI representation in the core crates (Ronald Holshausen, Mon May 3 12:14:02 2021 +1000)
* 6af4d3f - feat: allow ffi bindings to set spec version (Matt Fellows, Sun May 2 22:41:41 2021 +1000)

# 0.0.1 - Refactor: moved content type and body code from pact_matching

* 5ea36db - refactor: move content handling functions to pact_models crate (Ronald Holshausen, Sun Apr 25 13:12:22 2021 +1000)
* d010630 - chore: cleanup deprecation and compiler warnings (Ronald Holshausen, Sun Apr 25 12:23:30 2021 +1000)
* 3dd610a - refactor: move structs and code dealing with bodies to a seperate package (Ronald Holshausen, Sun Apr 25 11:20:47 2021 +1000)
* a725ab1 - feat(V4): added synchronous request/response message formats (Ronald Holshausen, Sat Apr 24 16:05:12 2021 +1000)
* 4bcd94f - refactor: moved OptionalBody and content types to pact models crate (Ronald Holshausen, Thu Apr 22 14:01:56 2021 +1000)
* 80812d0 - refactor: move Consumer and Provider structs to models crate (Ronald Holshausen, Thu Apr 22 13:11:03 2021 +1000)
* 220fb5e - refactor: move the PactSpecification enum to the pact_models crate (Ronald Holshausen, Thu Apr 22 11:18:26 2021 +1000)
* 83d3d60 - chore: bump version to 0.0.1 (Ronald Holshausen, Thu Apr 22 10:52:04 2021 +1000)
* 9962e0e - chore: add required metadata fields to Cargo manifest (Ronald Holshausen, Thu Apr 22 10:45:14 2021 +1000)
* 34e7dcd - chore: add a pact models crate (Ronald Holshausen, Thu Apr 22 10:04:40 2021 +1000)

# 0.0.0 - First Release
