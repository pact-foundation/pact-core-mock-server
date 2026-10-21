#!/usr/bin/env bash
# Rebuilds the pre-dist asset names from a dist release's archives,
# without recompiling: each binary is gzipped under its old name with an
# `openssl dgst -r`-style `.sha256` file.
#
# Usage: legacy-artifacts.sh <tag> <out-dir>
# Reads the release's assets with gh (needs GH_TOKEN and
# GITHUB_REPOSITORY), or, with LEGACY_ARCHIVES set, from that directory:
# for example a release gate run's artifacts, to check them before a tag.
set -euo pipefail

# MARK: Cut-off

# CUTOFF is the single source of the legacy cut-off date. From that date
# this script writes nothing, so no legacy assets are published. After it,
# delete:
# - .github/workflows/legacy-artifacts.yml;
# - the `legacy-artifacts` entry in dist-workspace.toml, then run
#   `dist generate` and re-apply any hand edits to release.yml;
# - .github/scripts/legacy-artifacts.sh.
CUTOFF=2027-06-30

if [[ ! "$(date -u +%Y-%m-%d)" < "$CUTOFF" ]]; then
    echo "::notice::Past the $CUTOFF legacy cut-off; no legacy assets are written."
    exit 0
fi

# MARK: Build

# shellcheck source=release-tags.sh
source "$(dirname "${BASH_SOURCE[0]}")/release-tags.sh"

tag=$1
out=$(mkdir -p "$2" && cd "$2" && pwd)
package=$(tag_crate "$tag")
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cd "$work"

# Copies release asset <name> into <dir>.
download() {
    local name=$1 dir=$2
    if [[ -n ${LEGACY_ARCHIVES-} ]]; then
        cp "$LEGACY_ARCHIVES/$name" "$dir/"
    else
        gh release download "$tag" --repo "$GITHUB_REPOSITORY" -p "$name" -D "$dir" --clobber
    fi
}

gzip_and_sum() {
    local src=$1 name=$2
    gzip --no-name --best --stdout "$src" > "$out/$name"
    (cd "$out" && sha256sum --binary "$name" > "$name.sha256")
}

# Prints the path of <file> after unpacking <target>'s archive. Archives
# may or may not wrap their files in a top-level directory.
extract() {
    local target=$1 file=$2 archive path
    if [[ ! -d "$target" ]]; then
        if [[ $target == *windows* ]]; then archive="$package-$target.zip"; else archive="$package-$target.tar.xz"; fi
        download "$archive" .
        mkdir "$target"
        case "$archive" in
            *.zip) unzip -q "$archive" -d "$target" ;;
            *) tar xJf "$archive" -C "$target" ;;
        esac
    fi
    path=$(find "$target" -type f -name "$file" | head -1)
    [[ -n "$path" ]] || { echo "$file not found in $target" >&2; return 1; }
    echo "$path"
}

case "$package" in
    pact_mock_server_cli)
        # Linux names always pointed at the static musl builds.
        for entry in \
            x86_64-unknown-linux-musl:pact-mock-server:pact-mock-server-linux-x86_64.gz \
            aarch64-unknown-linux-musl:pact-mock-server:pact-mock-server-linux-aarch64.gz \
            x86_64-apple-darwin:pact-mock-server:pact-mock-server-macos-x86_64.gz \
            aarch64-apple-darwin:pact-mock-server:pact-mock-server-macos-aarch64.gz \
            x86_64-pc-windows-msvc:pact-mock-server.exe:pact-mock-server-windows-x86_64.exe.gz \
            aarch64-pc-windows-msvc:pact-mock-server.exe:pact-mock-server-windows-aarch64.exe.gz
        do
            IFS=: read -r target file name <<<"$entry"
            path=$(extract "$target" "$file")
            gzip_and_sum "$path" "$name"
        done
        ;;
    *)
        echo "No legacy assets for $package" >&2
        ;;
esac
