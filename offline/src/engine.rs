use crate::actions::{processar_comando_acao, COR_AMARELO, COR_BRANCO, COR_VERDE, COR_VERMELHO};
use crate::audio::AudioManager;
use crate::data::{salas_terminais, BARULHO_ERROS_CONSECUTIVOS, LIMITE_ERROS_DICA};
use crate::devmode;
use crate::media::MediaManager;
use crate::minigames::{imprimir_painel_cofre, processar_minigame};
use crate::parser::normalizar;
use crate::state::GameState;
use crate::utils::{atualizar_eventos_de_tempo, CONQUISTAS_DB};
use crate::views::{
    dar_tela_de_morte, dar_tela_kernel_panic, imprimir_contexto_sala, imprimir_menu_dificuldade,
    imprimir_tela_boot, imprimir_tutorial, rodar_final,
};
use crate::TerminalBuffer;

fn destravar_conquista(
    id: &str,
    jogo: &mut GameState,
    audio: &mut AudioManager,
    term: &mut TerminalBuffer,
) {
    let nova = jogo.desbloquear_conquista(id);
    if !audio.config.conquistas_globais.contains(&id.to_string()) {
        audio.config.conquistas_globais.push(id.to_string());
        audio.config.salvar();
    }
    if nova {
        if let Some(info) = CONQUISTAS_DB
            .iter()
            .find(|c| c.id == id || c.chave_db == id)
        {
            term.push(
                &format!(
                    "\n[ {} CONQUISTA DESBLOQUEADA: {} — {} ]",
                    info.icone, info.nome, info.desc
                ),
                COR_AMARELO,
            );
        }
    }
}

pub fn verificar_final_de_jogo(
    jogo: &mut GameState,
    term: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) -> bool {
    if jogo.sala_atual == "hall de entrada" && jogo.noite_vencida && jogo.fios_cortados_inventario {
        destravar_conquista("final_verdadeiro", jogo, audio, term);
        if jogo.dificuldade_escolhida == "PESADELO" {
            destravar_conquista("pesadelo_verdadeiro", jogo, audio, term);
        }
        rodar_final("verdadeiro", jogo, term, audio, media);
        return true;
    }

    if jogo.sala_atual == "final_bom" {
        destravar_conquista("final_bom", jogo, audio, term);
        if jogo.dificuldade_escolhida == "PESADELO" {
            destravar_conquista("pesadelo_neutro", jogo, audio, term);
        }
        rodar_final("final_bom", jogo, term, audio, media);
        return true;
    }

    if jogo.sala_atual == "cama" {
        rodar_final("cama", jogo, term, audio, media);
        return true;
    }

    if jogo.sala_atual == "saida" {
        if jogo.dificuldade_escolhida == "NORMAL" {
            destravar_conquista("normal_zerado", jogo, audio, term);
        }
        rodar_final("saida", jogo, term, audio, media);
        return true;
    }

    if jogo.sala_atual == "morte" || jogo.hp <= 0 {
        destravar_conquista("primeira_morte", jogo, audio, term);
        dar_tela_de_morte(jogo, term, audio, media);
        return true;
    }

    false
}

pub fn iniciar_partida_com_tutorial(
    dificuldade: &str,
    texto_rapido: bool,
    modo_desafio: Option<&str>,
    jogo: &mut GameState,
    term: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) {
    jogo.reiniciar_partida(dificuldade);
    jogo.texto_rapido = texto_rapido;
    jogo.modo_desafio = modo_desafio.map(|s| s.to_string());
    if modo_desafio == Some("breu_total") {
        jogo.turnos_luz = 0;
    }
    audio.iniciar_ambiente();
    term.limpar();
    imprimir_tutorial(term, jogo);
    imprimir_contexto_sala(jogo, term, media);
}

pub fn processar_fluxo_jogo(
    comando_bruto: &str,
    jogo: &mut GameState,
    term: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) {
    let comando = normalizar(comando_bruto);

    if jogo.estado_atual == "FIM" {
        media.imagem_atual = None;
        imprimir_tela_boot(term);
        jogo.estado_atual = "AGUARDANDO_DIR".to_string();
        return;
    }

    if comando.is_empty() {
        return;
    }

    if devmode::eh_codigo_de_ativacao(&comando) {
        audio.iniciar_ambiente();
        devmode::ativar_modo_dev(jogo, term, media);
        destravar_conquista("trapaceiro", jogo, audio, term);
        return;
    }

    if jogo.dev_mode && devmode::processar_comando_dev(&comando, jogo, term, media) {
        return;
    }

    let snapshot = devmode::tirar_snapshot(jogo);
    let estado_antes = jogo.estado_atual.clone();

    match jogo.estado_atual.as_str() {
        "AGUARDANDO_DIR" => {
            if comando == "dir" {
                jogo.estado_atual = "MENU_DIFICULDADE".to_string();
                term.limpar();
                imprimir_menu_dificuldade(
                    term,
                    GameState::save_existe(),
                    &audio.config.conquistas_globais,
                );
            } else {
                term.push(
                    "Bad command or file name. Digite 'dir' para acessar os diretórios.",
                    COR_VERMELHO,
                );
            }
        }
        "MENU_DIFICULDADE" => match comando.as_str() {
            "1" => iniciar_partida_com_tutorial("NORMAL", false, None, jogo, term, audio, media),
            "2" => iniciar_partida_com_tutorial("NORMAL", true, None, jogo, term, audio, media),
            "3" => iniciar_partida_com_tutorial("PESADELO", false, None, jogo, term, audio, media),
            "4" => iniciar_partida_com_tutorial("PESADELO", true, None, jogo, term, audio, media),
            "5" => {
                jogo.estado_atual = "MENU_DESAFIOS".to_string();
                term.push(
                    "\n=== MODOS DE DESAFIO ===\n\
                    [1] SPEEDRUN (Menos de 5 minutos)\n\
                    [2] FANTASMA (Barulho nunca pode passar de 50%)\n\
                    [3] BREU TOTAL (Sem lanterna desde o início)\n\
                    [voltar] Retornar ao menu",
                    COR_AMARELO,
                );
            }
            "6" => {
                jogo.estado_atual = "MENU_PERSONALIZADO".to_string();
                term.push(
                    "\n=== MODO PERSONALIZADO ===\n\
                    Digite 'iniciar' para começar com parâmetros customizados ou 'voltar'.",
                    COR_AMARELO,
                );
            }
            "7" if GameState::save_existe() => {
                if let Some(salvo) = GameState::carregar_local() {
                    *jogo = salvo;
                    audio.iniciar_ambiente();
                    term.limpar();
                    term.push("[ AUTOSAVE CARREGADO COM SUCESSO ]", COR_VERDE);
                    imprimir_contexto_sala(jogo, term, media);
                }
            }
            "2007" => {
                dar_tela_kernel_panic(jogo, term);
            }
            _ => {
                imprimir_menu_dificuldade(
                    term,
                    GameState::save_existe(),
                    &audio.config.conquistas_globais,
                );
            }
        },
        "MENU_DESAFIOS" => match comando.as_str() {
            "1" => iniciar_partida_com_tutorial(
                "NORMAL",
                true,
                Some("speedrun"),
                jogo,
                term,
                audio,
                media,
            ),
            "2" => iniciar_partida_com_tutorial(
                "NORMAL",
                true,
                Some("fantasma"),
                jogo,
                term,
                audio,
                media,
            ),
            "3" => iniciar_partida_com_tutorial(
                "NORMAL",
                true,
                Some("breu_total"),
                jogo,
                term,
                audio,
                media,
            ),
            _ => {
                jogo.estado_atual = "MENU_DIFICULDADE".to_string();
                imprimir_menu_dificuldade(
                    term,
                    GameState::save_existe(),
                    &audio.config.conquistas_globais,
                );
            }
        },
        "MENU_PERSONALIZADO" => {
            if comando == "voltar" {
                jogo.estado_atual = "MENU_DIFICULDADE".to_string();
                imprimir_menu_dificuldade(
                    term,
                    GameState::save_existe(),
                    &audio.config.conquistas_globais,
                );
            } else {
                iniciar_partida_com_tutorial("PESADELO", false, None, jogo, term, audio, media);
            }
        }
        "JOGO" | "COMBATE_ANIMATRONICO" => {
            if ["cls", "limpar", "clear", "clean"].contains(&comando.as_str()) {
                term.limpar();
                imprimir_contexto_sala(jogo, term, media);
                return;
            }
            if comando == "ajuda" || comando == "help" {
                imprimir_tutorial(term, jogo);
                return;
            }
            if comando == "salvar" {
                if jogo.salvar_local() {
                    term.push("Jogo salvo em 'savegame.json'.", COR_VERDE);
                }
                return;
            }

            if comando == "abrir cofre" && jogo.sala_atual == "01" {
                jogo.estado_atual = "MINIGAME_COFRE".to_string();
                jogo.cofre_tentativas = 0;
                imprimir_painel_cofre(jogo, term, media, true);
                return;
            }
            if comando == "cadeira" && jogo.sala_atual == "01" {
                if jogo
                    .inventario
                    .iter()
                    .any(|i| normalizar(i).contains("cartao"))
                {
                    jogo.estado_atual = "MINIGAME_SEGURANCA".to_string();
                    term.push(
                        "[ TERMINAL DE SEGURANÇA ] Digite 'vigiar' para avançar o turno até as 6AM.",
                        COR_VERDE,
                    );
                } else {
                    term.push("Acesso negado: exige o Cartão de Segurança.", COR_VERMELHO);
                }
                return;
            }
            if (comando == "jogar jon" || comando == "jogar fome de jon")
                && jogo.sala_atual == "sala de fliperamas"
            {
                jogo.estado_atual = "MINIGAME_JON".to_string();
                term.push(
                    "[ FLIPERAMA: FOME DE JON ] Digite 'f', 'e', 'd' ou 'sair'.",
                    COR_AMARELO,
                );
                return;
            }
            if comando == "jogar consertos" && jogo.sala_atual == "sala de fliperamas" {
                if let Some(pos) = jogo
                    .inventario
                    .iter()
                    .position(|i| normalizar(i).contains("moeda"))
                {
                    jogo.inventario.remove(pos);
                    jogo.estado_atual = "MINIGAME_CONSERTOS_CABECA".to_string();
                    term.push(
                        "[ FLIPERAMA: CONSERTOS ] Moeda inserida! Digite 'consertar'.",
                        COR_AMARELO,
                    );
                } else {
                    term.push("Você precisa da 'moeda velha' para jogar.", COR_VERMELHO);
                }
                return;
            }
            if comando == "jogar adivinha" && jogo.sala_atual == "sala de fliperamas" {
                if jogo
                    .inventario
                    .iter()
                    .any(|i| normalizar(i).contains("partitura"))
                {
                    jogo.estado_atual = "MINIGAME_JULGAMENTO_Q1".to_string();
                    term.push(
                        "[ FLIPERAMA: O JULGAMENTO ] Digite as notas ou 'tocar'.",
                        COR_BRANCO,
                    );
                } else {
                    term.push("Exige a 'partitura rasgada'.", COR_VERMELHO);
                }
                return;
            }

            let mut jogador_correu = false;
            let mut comando_exec = comando.clone();
            if let Some(dir) = comando.strip_prefix("correr ") {
                let d = match dir.trim() {
                    "f" => "frente",
                    "t" | "atras" => "atrás",
                    "e" => "esquerda",
                    "d" => "direita",
                    outro => outro,
                };
                comando_exec = format!("ir {}", d);
                jogador_correu = true;
                if jogo.estado_atual == "COMBATE_ANIMATRONICO" {
                    jogo.estado_atual = "JOGO".to_string();
                }
            }

            if estado_antes == "COMBATE_ANIMATRONICO" && !jogador_correu {
                if jogo.dev_mode {
                    devmode::interceptar_perseguidor(jogo, term);
                } else {
                    jogo.hp = 0;
                    jogo.sala_atual = "morte".to_string();
                    let _ = verificar_final_de_jogo(jogo, term, audio, media);
                }
                return;
            }

            let sala_anterior = jogo.sala_atual.clone();
            let resultado_acao =
                processar_comando_acao(&comando_exec, jogo, term, audio, media);

            if jogo.sala_atual != sala_anterior {
                jogo.turnos_mesma_sala = 0;
            }

            let mut foi_largar = false;
            if let Some(tipo) = resultado_acao {
                jogo.erros_consecutivos = 0;
                foi_largar = tipo == "largar";
                if jogo.estado_atual == "JOGO" && tipo != "olhar" {
                    if jogador_correu {
                        jogo.nivel_barulho = 100;
                    }
                    atualizar_eventos_de_tempo(jogo, term);
                    let rel = jogo.avancar_ia_perseguidor();
                    if rel.combate_iniciado {
                        if jogo.dev_mode {
                            devmode::interceptar_perseguidor(jogo, term);
                        } else {
                            audio.som_grito();
                            let _ = media.acionar_imagem("animatronico", false, true);
                            term.push(
                                "[ PERIGO IMINENTE ] O animatrônico entrou na sala! Digite 'correr <direção>'!",
                                COR_VERMELHO,
                            );
                        }
                    } else if rel.ativa && rel.distancia <= 2 {
                        audio.som_metal();
                    }
                    let _ = jogo.salvar_local();
                }
            } else {
                if jogador_correu && estado_antes == "COMBATE_ANIMATRONICO" {
                    jogo.estado_atual = "COMBATE_ANIMATRONICO".to_string();
                }
                jogo.erros_consecutivos += 1;
                if jogo.erros_consecutivos >= LIMITE_ERROS_DICA {
                    jogo.alterar_barulho(BARULHO_ERROS_CONSECUTIVOS);
                    jogo.ai_alvo = Some(jogo.sala_atual.clone());
                    term.push(
                        "Seus tropeços no escuro estão fazendo barulho...",
                        COR_AMARELO,
                    );
                    jogo.erros_consecutivos = 0;
                }
            }

            let terminais = salas_terminais();
            if !jogo.mapa.contains_key(&jogo.sala_atual)
                && !terminais.contains(&jogo.sala_atual.as_str())
            {
                jogo.sala_atual = "01".to_string();
            }

            if jogo.estado_atual == "DIALOGO_FINAL" && jogo.incendio {
                destravar_conquista("final_verdadeiro", jogo, audio, term);
                if jogo.dificuldade_escolhida == "PESADELO" {
                    destravar_conquista("pesadelo_verdadeiro", jogo, audio, term);
                }
                rodar_final("verdadeiro", jogo, term, audio, media);
                return;
            }

            if jogo.atingiu_meta_acumulador() {
                destravar_conquista("acumulador", jogo, audio, term);
            }

            if !verificar_final_de_jogo(jogo, term, audio, media) && jogo.estado_atual == "JOGO" {
                if resultado_acao.is_some() && resultado_acao != Some("olhar") {
                    imprimir_contexto_sala(jogo, term, media);
                }
            }

            if jogo.dev_mode && estado_antes != "FIM" {
                devmode::pos_turno(jogo, snapshot, foi_largar, term);
            }
        }
        _ => {
            let antes = jogo.conquistas.len();
            let _ = processar_minigame(&comando, jogo, term, audio, media);
            if jogo.conquistas.len() > antes {
                if let Some(ultima) = jogo.conquistas.last().cloned() {
                    if ultima == "mestre_cofre" && jogo.cofre_tentativas == 0 {
                        destravar_conquista("mente_brilhante", jogo, audio, term);
                    } else if ultima == "engenheiro_eletrico" {
                        destravar_conquista("labirinto", jogo, audio, term);
                    } else {
                        destravar_conquista(&ultima, jogo, audio, term);
                    }
                }
            }
            if jogo.estado_atual == "JOGO" {
                imprimir_contexto_sala(jogo, term, media);
            }
        }
    }
}