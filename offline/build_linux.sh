#!/usr/bin/env bash
set -euo pipefail

cd -- "$(dirname -- "$0")"
cargo build --release --locked
install -D -m755 target/release/vilas_boas dist/vilas_boas
printf '%s\n' 'Executável disponível em offline/dist/vilas_boas'
