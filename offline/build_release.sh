#!/usr/bin/env bash
set -e

echo "=== 1. Instalando dependências para Linux e Windows (.exe) ==="
sudo apt-get update -qq
sudo apt-get install -y -qq libasound2-dev pkg-config mingw-w64 \
    libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev

rustup target add x86_64-pc-windows-gnu

echo "=== 2. Compilando para Linux e Windows ==="
cargo build --release
cargo build --release --target x86_64-pc-windows-gnu

echo "=== 3. Empacotando Executáveis + /data (*.json) + /assets (MP3 e Imagens) ==="
mkdir -p dist/assets dist/data
cp target/release/vilas_boas dist/vilas_boas_linux
cp target/x86_64-pc-windows-gnu/release/vilas_boas.exe dist/vilas_boas_windows.exe

cp -r ../client/assets/* dist/assets/
cp -r ../server/data/*.json dist/data/

echo "Concluído! Pacote completo gerado em: offline/dist/"