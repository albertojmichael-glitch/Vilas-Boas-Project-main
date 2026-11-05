use crate::actions::{COR_AMARELO, COR_BRANCO, COR_CIANO, COR_VERDE, COR_VERMELHO};
use crate::audio::AudioManager;
use crate::data::{MAX_TENTATIVAS_COFRE, SENHA_COFRE};
use crate::media::MediaManager;
use crate::state::GameState;
use crate::TerminalBuffer;
use rand::seq::SliceRandom;
use rand::Rng;

const PISTAS_COFRE: [&str; 3] = [
    "[1º dígito] Um único ponto de luz acende sozinho na unidade, o primeiro som da noite.",
    "[2º e 3º dígitos] Nove marcas de unha, em 2 vezes, na fileira da lateral do metal.",
    "[4º dígito] Quatro cadeiras na sala do piano. Só uma nunca foi ocupada de novo.",
];

pub fn imprimir_painel_cofre(
    jogo: &GameState,
    term: &mut TerminalBuffer,
    media: &mut MediaManager,
    primeira_vez: bool,
) {
    let _ = media.acionar_imagem("cofre", false, false);
    if primeira_vez {
        term.push(
            "O cofre de ferro possui um teclado numérico antigo, gasto pelo tempo.",
            COR_BRANCO,
        );
        term.push(
            "Gravado a canivete embaixo do teclado, pistas para a senha:",
            COR_AMARELO,
        );
        let mut pistas = PISTAS_COFRE.to_vec();
        pistas.shuffle(&mut rand::thread_rng());
        for p in pistas {
            term.push(&format!("  {}", p), COR_CIANO);
        }
    }

    let restantes = MAX_TENTATIVAS_COFRE - jogo.cofre_tentativas;
    if jogo.cofre_tentativas > 0 {
        term.push(
            &format!("Tentativas restantes antes do bloqueio: {}", restantes),
            COR_VERMELHO,
        );
    }
    term.push("Digite a senha de 4 dígitos (ou 'sair'):", COR_VERDE);
}

pub fn processar_minigame(
    comando: &str,
    jogo: &mut GameState,
    term: &mut TerminalBuffer,
    audio: &AudioManager,
    media: &mut MediaManager,
) -> bool {
    match jogo.estado_atual.as_str() {
        "MINIGAME_COFRE" => {
            if comando == "sair" || comando == "voltar" {
                jogo.estado_atual = "JOGO".to_string();
                term.push("Você se afasta do cofre.", COR_BRANCO);
                return true;
            }
            if comando == SENHA_COFRE {
                audio.som_porta();
                jogo.cofre_aberto = true;
                jogo.estado_atual = "JOGO".to_string();
                if !jogo.inventario.contains(&"chave mestra".to_string()) {
                    jogo.inventario.push("chave mestra".to_string());
                }
                term.push(
                    "[ CLACK! ] O cofre abriu! Você obteve a 'chave mestra'!",
                    COR_VERDE,
                );
            } else {
                audio.som_metal();
                jogo.cofre_tentativas += 1;
                jogo.alterar_barulho(25);
                if jogo.cofre_tentativas >= MAX_TENTATIVAS_COFRE {
                    jogo.estado_atual = "JOGO".to_string();
                    term.push(
                        "[ ALARME! ] O cofre travou disparando um alerta sonoro!",
                        COR_VERMELHO,
                    );
                    jogo.alterar_barulho(60);
                } else {
                    imprimir_painel_cofre(jogo, term, media, false);
                }
            }
            true
        }
        "MINIGAME_JON" => {
            if comando == "sair" {
                jogo.estado_atual = "JOGO".to_string();
                return true;
            }
            let _ = media.acionar_imagem("jon", true, false);
            term.push(
                "[ FOME DE JON ] Você completou o fliperama! Um registro secreto piscou na tela!",
                COR_VERDE,
            );
            jogo.desbloquear_conquista("fliperama_jon");
            jogo.estado_atual = "JOGO".to_string();
            true
        }
        "MINIGAME_CONSERTOS_CABECA" => {
            if comando == "sair" {
                jogo.estado_atual = "JOGO".to_string();
                return true;
            }
            term.push(
                "[ CONSERTOS ] Você reparou os mecanismos endosqueléticos com sucesso!",
                COR_VERDE,
            );
            jogo.estado_atual = "JOGO".to_string();
            true
        }
        "MINIGAME_JULGAMENTO_Q1" => {
            if comando == "sair" {
                jogo.estado_atual = "JOGO".to_string();
                return true;
            }
            let _ = media.acionar_imagem("secret", true, false);
            term.push(
                "[ O JULGAMENTO ] As notas da partitura revelam uma imagem oculta nos arquivos!",
                COR_CIANO,
            );
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
                term.push(
                    "[ 06:00 AM ] Turno encerrado! As travas magnéticas da saída abriram!",
                    COR_VERDE,
                );
            } else {
                term.push(
                    &format!(
                        "[ MONITORAMENTO ] Vigília em {}%. Digite 'vigiar', 'porta' ou 'sair'.",
                        jogo.seguranca_progresso
                    ),
                    COR_AMARELO,
                );
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
                term.push(
                    "[ ENERGIA RESTABELECIDA ] Labirinto concluído! Saída destrancada!",
                    COR_VERDE,
                );
            } else {
                term.push(
                    &format!("Posição no circuito: ({}, {}). Destino: (2, 2).", x, y),
                    COR_CIANO,
                );
            }
            true
        }
        _ => false,
    }
}