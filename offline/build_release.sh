#!/usr/bin/env bash
set -e

echo "=== 1. Instalando dependências de sistema e cross-compile no Codespace ==="
sudo apt-get update -qq
sudo apt-get install -y -qq libasound2-dev pkg-config mingw-w64

if ! command -v cargo &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

rustup target add x86_64-pc-windows-gnu

echo "=== 2. Compilando binário para LINUX (x86_64) ==="
cargo build --release

echo "=== 3. Compilando executável para WINDOWS (.exe) ==="
cargo build --release --target x86_64-pc-windows-gnu

echo "=== 4. Montando pacote offline final em ./dist ==="
mkdir -p dist/assets dist/data
cp target/release/vilas_boas dist/vilas_boas_linux
cp target/x86_64-pc-windows-gnu/release/vilas_boas.exe dist/vilas_boas_windows.exe

# Copia os áudios, imagens, secret images e mapa para funcionar 100% offline
cp -r ../client/assets/* dist/assets/ 2>/dev/null || true
cp ../server/data/game_data.json dist/data/ 2>/dev/null || true

echo "Pronto! Seus executáveis estão em: offline/dist/"