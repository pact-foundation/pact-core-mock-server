#!/usr/bin/env bash
# Prints the GitHub release title for a release tag:
# `Pact Mock Server Library v<version>` or `Pact Mock Server Standalone v<version>`.
#
# Usage: release-title.sh <tag>
set -euo pipefail

# shellcheck source=release-tags.sh
source "$(dirname "${BASH_SOURCE[0]}")/release-tags.sh"

tag=$1
crate=$(tag_crate "$tag")
version=$(tag_version "$tag")

case $crate in
    pact_mock_server) echo "Pact Mock Server Library v$version" ;;
    pact_mock_server_cli) echo "Pact Mock Server Standalone v$version" ;;
esac
