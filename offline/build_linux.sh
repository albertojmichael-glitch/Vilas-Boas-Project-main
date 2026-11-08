#!/usr/bin/env bash
set -e

cd "$(dirname "$0")"

echo "=== 1. Verificando dependências de sistema para Linux (ALSA / X11 / Wayland) ==="
if command -v apt-get >/dev/null 2>&1; then
    sudo apt-get update -qq
    sudo apt-get install -y -qq libasound2-dev pkg-config \
        libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev
fi

echo "=== 2. Compilando Vilas-Boas Entertainment v20.08 para Linux (x86_64-unknown-linux-gnu) ==="
cargo build --release

mkdir -p dist
cp target/release/vilas_boas vilas_boas
cp target/release/vilas_boas dist/vilas_boas_linux
chmod +x vilas_boas dist/vilas_boas_linux

echo "=== Concluído com sucesso! ==="
echo "Binário Linux disponível em:"
echo "  - offline/vilas_boas"
echo "  - offline/target/release/vilas_boas"
echo "  - offline/dist/vilas_boas_linux"

