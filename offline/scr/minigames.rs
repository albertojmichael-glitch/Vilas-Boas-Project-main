use crate::audio::AudioManager;
use crate::data::{MAX_TENTATIVAS_COFRE, SENHA_COFRE};
use crate::state::GameState;
use crate::ui::{
    TerminalUI, DOS_AMARELO, DOS_BRANCO, DOS_CIANO, DOS_VERDE, DOS_VERMELHO, RESET,
};
use rand::seq::SliceRandom;
use rand::Rng;

pub const ARTE_COFRE: &str = r#"
  __________________________
 /  ______________________  \
|  |  __   __   __   __   |  |
|  | |  | |  | |  | |  |  |  |
|  | |__| |__| |__| |__|  |  |
|  |                      |  |
|  |       .------.       |  |
|  |      /        \      |  |
|  |     |   [**]   |     |  |
|  |      \        /      |  |
|  |       `------'       |  |
|  |                      |  |
|  |______________________|  |
 \__________________________/
"#;

pub const ARTE_COFRE_TENSO: &str = r#"
  __________________________
 /  ______________/\_______  \
|  |  __   __   __\/  __   |  |
|  | |  | |  | |  |  |  |  |  |
|  | |__| |__|/|__|  |__|  |  |
|  |         /            |  |
|  |       .------.       |  |
|  |      /   !!   \      |  |
|  |     | x [**] x |     |  |
|  |      \        /      |  |
|  |       `------'       |  |
|  |                      |  |
|  |______________________|  |
 \__________________________/
"#;

const PISTAS_COFRE: [&str; 3] = [
    "[1º dígito] Um único ponto de luz acende sozinho na unidade, o primeiro som da noite.",
    "[2º e 3º dígitos] Nove marcas de unha, em 2 vezes, na fileira da lateral do metal.",
    "[4º dígito] Quatro cadeiras na sala do piano. Só uma nunca foi ocupada de novo.",
];

pub fn gerar_charada_cofre() -> String {
    let mut pistas = PISTAS_COFRE.to_vec();
    let mut rng = rand::thread_rng();
    pistas.shuffle(&mut rng);
    pistas
        .iter()
        .map(|p| format!("  {}", p))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn imprimir_painel_cofre(jogo: &GameState, ui: &TerminalUI, primeira_vez: bool) {
    if jogo.cofre_tentativas == 0 {
        ui.animar(ARTE_COFRE, 0.002, DOS_BRANCO);
    } else {
        ui.animar(ARTE_COFRE_TENSO, 0.002, DOS_VERMELHO);
    }

    if primeira_vez {
        ui.exibir(&format!(
            "{}O cofre de ferro possui um teclado numérico antigo, gasto pelo tempo.{}",
            DOS_BRANCO, RESET
        ));
        ui.exibir(&format!(
            "{}Gravado a canivete embaixo do teclado, quatro frases soltas:{}",
            DOS_BRANCO, RESET
        ));
        ui.exibir(&gerar_charada_cofre());
    }

    let restantes = MAX_TENTATIVAS_COFRE - jogo.cofre_tentativas;
    if jogo.cofre_tentativas > 0 {
        ui.exibir(&format!(
            "{}Tentativas restantes antes do painel travar: {}{}",
            DOS_VERMELHO, restantes, RESET
        ));
    }
    ui.exibir(&format!(
        "{}Digite a senha de 4 dígitos (ou 'sair'): {}",
        DOS_VERDE, RESET
    ));
}

pub fn processar_minigame(
    comando: &str,
    jogo: &mut GameState,
    ui: &TerminalUI,
    audio: &AudioManager,
) -> bool {
    match jogo.estado_atual.as_str() {
        "MINIGAME_COFRE" => {
            if comando == "sair" || comando == "voltar" {
                jogo.estado_atual = "JOGO".to_string();
                ui.exibir("Você se afasta do cofre.");
                return true;
            }
            if comando == SENHA_COFRE {
                audio.som_porta();
                jogo.cofre_aberto = true;
                jogo.estado_atual = "JOGO".to_string();
                if !jogo.inventario.contains(&"chave mestra".to_string()) {
                    jogo.inventario.push("chave mestra".to_string());
                }
                ui.exibir(&format!(
                    "{}[ CLACK! ] O cofre destranca! Você encontrou a 'chave mestra'!{}",
                    DOS_VERDE, RESET
                ));
            } else {
                audio.som_metal();
                jogo.cofre_tentativas += 1;
                jogo.alterar_barulho(25);
                if jogo.cofre_tentativas >= MAX_TENTATIVAS_COFRE {
                    jogo.estado_atual = "JOGO".to_string();
                    ui.exibir(&format!(
                        "{}[ ALARME! ] O painel do cofre travou emitindo um apito estridente!{}",
                        DOS_VERMELHO, RESET
                    ));
                    jogo.alterar_barulho(60);
                } else {
                    imprimir_painel_cofre(jogo, ui, false);
                }
            }
            true
        }
        "MINIGAME_JON" => {
            if comando == "sair" {
                jogo.estado_atual = "JOGO".to_string();
                return true;
            }
            ui.exibir(&format!(
                "{}[ FOME DE JON ] Você alimentou Jon com hambúrgueres pixelados! Vitória no fliperama!{}",
                DOS_VERDE, RESET
            ));
            jogo.desbloquear_conquista("fliperama_jon");
            jogo.estado_atual = "JOGO".to_string();
            true
        }
        "MINIGAME_CONSERTOS_CABECA" => {
            if comando == "sair" {
                jogo.estado_atual = "JOGO".to_string();
                return true;
            }
            ui.exibir(&format!(
                "{}[ CONSERTOS ] Você recalibrou os servos cranianos sem levar choque!{}",
                DOS_VERDE, RESET
            ));
            jogo.estado_atual = "JOGO".to_string();
            true
        }
        "MINIGAME_JULGAMENTO_Q1" => {
            if comando == "sair" {
                jogo.estado_atual = "JOGO".to_string();
                return true;
            }
            ui.exibir(&format!(
                "{}[ O JULGAMENTO ] A melodia da partitura ecoa pelo arcade. Segredo liberado!{}",
                DOS_CIANO, RESET
            ));
            jogo.estado_atual = "JOGO".to_string();
            true
        }
        "MINIGAME_SEGURANCA" => {
            if comando == "sair" {
                jogo.estado_atual = "JOGO".to_string();
                return true;
            }
            let mut rng = rand::thread_rng();
            let avanco: i32 = rng.gen_range(35..=55);
            jogo.seguranca_progresso += avanco;
            if jogo.seguranca_progresso >= 100 {
                jogo.noite_vencida = true;
                jogo.estado_atual = "JOGO".to_string();
                ui.exibir(&format!(
                    "{}[ 06:00 AM ] Turno de segurança concluído! As portas do Hall foram destravadas!{}",
                    DOS_VERDE, RESET
                ));
            } else {
                ui.exibir(&format!(
                    "{}[ TERMINAL DE SEGURANÇA ] Progresso da vigília: {}%. Digite 'vigiar', 'fechar porta' ou 'sair'.{}",
                    DOS_AMARELO, jogo.seguranca_progresso, RESET
                ));
            }
            true
        }
        "MINIGAME_MINOTAURO" => {
            if comando == "sair" {
                jogo.estado_atual = "JOGO".to_string();
                return true;
            }
            let (mut x, mut y) = jogo.minotauro_pos;
            match comando {
                "w" | "frente" | "cima" => y = y.saturating_sub(1),
                "s" | "atras" | "baixo" => y = (y + 1).min(2),
                "a" | "esquerda" => x = x.saturating_sub(1),
                "d" | "direita" => x = (x + 1).min(2),
                _ => {}
            }
            jogo.minotauro_pos = (x, y);
            if (x, y) == (2, 2) {
                jogo.noite_vencida = true;
                jogo.estado_atual = "JOGO".to_string();
                ui.exibir(&format!(
                    "{}[ ENERGIA RESTAURADA ] Você venceu o Labirinto do Minotauro! Saída destrancada!{}",
                    DOS_VERDE, RESET
                ));
            } else {
                ui.exibir(&format!(
                    "{}Posição no circuito: ({}, {}). Chegue até (2, 2) usando w/a/s/d.{}",
                    DOS_CIANO, x, y, RESET
                ));
            }
            true
        }
        _ => false,
    }
}