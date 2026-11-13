#!/usr/bin/env bash
# Prints the body of one version's section from a changelog whose
# sections are `##` headings. It finds release-plz's `## [<version>]`
# headings only. Any other `##` heading, such as `## <version> - <title>`,
# ends the previous section; the script exits 1 when asked for its version.
#
# Usage: changelog-section.sh <CHANGELOG.md> <version>
# Exits 1, with an error on stderr, when the changelog has no section
# for <version>.
set -euo pipefail

changelog=$1
version=$2

if ! awk -v heading="## [${version}]" '
    /^## / {
        if (found) exit
        if (index($0, heading) == 1) { found = 1; next }
    }
    found { print }
    END { exit found ? 0 : 1 }
' "$changelog"; then
    echo "::error file=${changelog}::No \"## [${version}]\" section in ${changelog}." \
        "release-plz writes none when every commit since the last release is skipped" \
        "by commit_parsers in release-plz.toml." >&2
    exit 1
fi
