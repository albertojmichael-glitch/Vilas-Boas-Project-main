#!/usr/bin/env bash
set -euo pipefail

# Build Linux. O artefato Windows é compilado no runner Windows do GitHub Actions.
exec bash "$(dirname -- "$0")/build_linux.sh" "$@"
