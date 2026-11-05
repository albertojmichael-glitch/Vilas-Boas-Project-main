mod actions;
mod audio;
mod data;
mod devmode;
mod engine;
mod media;
mod minigames;
mod parser;
mod state;
mod ui;

use audio::AudioManager;
use engine::processar_fluxo_jogo;
use media::MediaManager;
use state::GameState;
use std::io::{self, BufRead, Write};
use ui::{imprimir_tela_boot, TerminalUI, DOS_VERDE, RESET};

fn main() {
    let mut audio = AudioManager::new();
    let mut ui = TerminalUI {
        animacao_ativa: audio.config.animacao_texto,
    };
    let mut media = MediaManager::new();
    media.abrir_janela_externa = audio.config.abrir_janela_externa;

    let mut jogo = GameState::new();
    jogo.conquistas = audio.config.conquistas_globais.clone();

    imprimir_tela_boot(&ui);

    let stdin = io::stdin();
    loop {
        print!("{}C:\\> {}", DOS_VERDE, RESET);
        let _ = io::stdout().flush();

        let mut linha = String::new();
        match stdin.lock().read_line(&mut linha) {
            Ok(0) => break,
            Ok(_) => {
                let entrada = linha.trim();
                if entrada.eq_ignore_ascii_case("exit") || entrada.eq_ignore_ascii_case("quit") {
                    break;
                }
                processar_fluxo_jogo(entrada, &mut jogo, &mut ui, &mut audio, &mut media);
            }
            Err(_) => break,
        }
    }
}