#!/usr/bin/env bash
# Maps workspace crates to release tags. Meant to be sourced.
# `release-plz.toml` repeats this mapping in `git_tag_name`.

# Prints the tag prefix for a crate; returns 1 for an unknown crate.
tag_prefix() {
    case $1 in
        pact_mock_server) echo libpact_mock_server ;;
        pact_mock_server_cli) echo pact_mock_server_cli ;;
        *) return 1 ;;
    esac
}

# Prints the crate for a `<prefix>-v<version>` tag; returns 1 for an unknown prefix.
tag_crate() {
    case ${1%-v*} in
        libpact_mock_server) echo pact_mock_server ;;
        pact_mock_server_cli) echo pact_mock_server_cli ;;
        *) return 1 ;;
    esac
}

# Prints the version of a `<prefix>-v<version>` tag.
tag_version() {
    echo "${1##*-v}"
}
