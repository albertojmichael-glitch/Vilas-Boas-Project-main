# Vilas Boas Offline

O jogo roda em uma janela nativa, com a engine e a interface em Rust. O executável incorpora mapas, textos, imagens, áudio e a fonte VT323. Abra a TV, digite `dir` e escolha uma dificuldade.

## Compilar

No Windows x64, instale o Rust stable com a toolchain MSVC e as ferramentas de compilação C++ do Visual Studio. Na raiz do repositório, execute `offline\build_windows.bat`. O arquivo final é `offline\dist\Vilas-Boas.exe`.

No Linux x64, instale o Rust stable e os pacotes `pkg-config` e `libasound2-dev` em Debian ou Ubuntu. Execute `bash offline/build_linux.sh` para gerar `offline/dist/vilas_boas`. O áudio usa a biblioteca ALSA do sistema.

O Cargo pode baixar dependências na primeira compilação. Para jogar, basta o executável. O workflow **Build offline** publica um artefato por sistema.

## Partidas locais

Ao iniciar a próxima sessão, escolha `7` no menu para retomar o autosave. O painel de saves permite copiar e importar o JSON de uma partida local. O jogo guarda o autosave, as configurações, as conquistas, os finais e o placar em `%LOCALAPPDATA%\VilasBoas` no Windows e em `${XDG_DATA_HOME:-$HOME/.local/share}/VilasBoas` no Linux. É possível escolher outra pasta pela variável `VILAS_BOAS_DATA_DIR`. Saves da versão Rust anterior são lidos de `autosave.json` ou `savegame.json` no diretório de trabalho quando ainda não há um autosave na nova pasta.

A edição offline mantém o placar e as conquistas no próprio computador. Login, compartilhamento, replay por link e telemetria pertencem à edição web. Os saves web criptografados usam outro formato.

## Desenvolvimento

Use `cargo fmt --all -- --check` e `cargo test --locked` dentro de `offline/`. O mapa e os parâmetros de balanceamento vêm dos mesmos JSONs da edição web em `server/data/`. Imagens e áudios vêm de `client/assets/`.

A fonte VT323 é distribuída com a [licença SIL Open Font License](fonts/OFL.txt).
