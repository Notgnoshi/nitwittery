#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset
set -o noclobber

PACKAGE="${1:?Usage: $0 <PACKAGE> [KEY]}"
MANIFEST_KEY="${2:-version}"

# Select the named package explicitly rather than packages[0]: version is uniform across the
# workspace, but description is per-crate, so ordering must not matter.
cargo metadata --format-version=1 --no-deps |
    jq -r ".packages[] | select(.name == \"$PACKAGE\") | .$MANIFEST_KEY"
