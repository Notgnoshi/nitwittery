#!/bin/bash
set -o errexit
set -o pipefail
set -o nounset

VERSION="${1:?Usage: $0 <VERSION> <DESCRIPTION>}"
DESCRIPTION="${2:?Usage: $0 <VERSION> <DESCRIPTION>}"

CHANGELOG="$(git rev-parse --show-toplevel)/CHANGELOG.md"

if ! grep -q "^# Nitwittery - $VERSION -" "$CHANGELOG"; then
    echo "ERROR: Could not find version '$VERSION' in '$CHANGELOG'" >&2
    exit 1
fi

# Project description, then a blank line, then the version's section body.
printf '%s\n\n' "$DESCRIPTION"

# 0,/pat/d deletes lines 1..header inclusive; /pat2/Q quits (without printing) at the next version
# header. The trailing sed drops leading blank lines so the body starts at real content.
sed "0,/^# Nitwittery - $VERSION -/d;/^# Nitwittery - /Q" "$CHANGELOG" | sed '/./,$!d'
