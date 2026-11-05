use crate::actions::{
    imprimir_contexto_sala, processar_comando_acao, COR_AMARELO, COR_BRANCO, COR_CIANO, COR_CINZA,
    COR_VERDE, COR_VERMELHO,
};
use crate::audio::AudioManager;
use crate::data::{salas_terminais, BARULHO_ERROS_CONSECUTIVOS, LIMITE_ERROS_DICA};
use crate::devmode;
use crate::media::MediaManager;
use crate::minigames::{imprimir_painel_cofre, processar_minigame};
use crate::parser::normalizar;
use crate::state::GameState;
use crate::TerminalBuffer;
use rand::Rng;

pub fn imprimir_boot(term: &mut TerminalBuffer) {
    term.clear();
    term.push("VILAS BOAS OS [SISTEMA v20.08 - TERMINAL LOCAL]", COR_VERDE);
    term.push(
        "(C) Copyright 1982 Vilas Boas Corp. Todos os direitos reservados.\n",
        COR_CINZA,
    );
    term.push("Digite 'dir' para listar os diretórios do sistema:", COR_VERDE);
}

pub fn imprimir_menu(term: &mut TerminalBuffer, tem_save: bool) {
    term.push(
        "\n================[ DIRETÓRIO C:\\VILAS_BOAS ]================",
        COR_VERDE,
    );
    term.push("  [1] INICIAR NOVA PARTIDA", COR_BRANCO);
    if tem_save {
        term.push("  [2] CONTINUAR PARTIDA SALVA", COR_AMARELO);
    } else {
        term.push("  [2] CONTINUAR (Nenhum save encontrado)", COR_CINZA);
    }
    term.push("  [3] CONQUISTAS E DESAFIOS", COR_BRANCO);
    term.push("  [4] ARQUIVOS DE IMAGEM E SECRET IMAGES", COR_CIANO);
    term.push("  [5] CONFIGURAÇÕES DE ÁUDIO E JUMPSCARE", COR_BRANCO);
    term.push("  [6] ENCERRAR SISTEMA", COR_BRANCO);
    term.push(
        "===========================================================",
        COR_VERDE,
    );
}

fn registrar_conquista(
    id: &str,
    jogo: &mut GameState,
    audio: &mut AudioManager,
    term: &mut TerminalBuffer,
) {
    if jogo.dev_usado {
        return;
    }
    let nova = jogo.desbloquear_conquista(id);
    if !audio.config.conquistas_globais.contains(&id.to_string()) {
        audio.config.conquistas_globais.push(id.to_string());
        audio.config.salvar();
    }
    if nova {
        term.push(
            &format!("\n[ 👑 CONQUISTA DESBLOQUEADA: {} ]", id.to_uppercase()),
            COR_AMARELO,
        );
    }
}

pub fn verificar_final_de_jogo(
    jogo: &mut GameState,
    term: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) -> bool {
    if jogo.sala_atual == "hall de entrada" && jogo.noite_vencida && jogo.fios_cortados_inventario {
        registrar_conquista("final_verdadeiro", jogo, audio, term);
        jogo.estado_atual = "FIM".to_string();
        audio.parar_ambiente();
        let _ = media.acionar_imagem("final", true, false);
        term.push(
            "\n[ FINAL VERDADEIRO ] Você destruiu a fonte de energia e escapou de Vilas Boas!",
            COR_VERDE,
        );
        return true;
    }

    if jogo.sala_atual == "saida" || jogo.sala_atual == "final_bom" {
        if jogo.dificuldade_escolhida == "PESADELO" {
            registrar_conquista("pesadelo_verdadeiro", jogo, audio, term);
        } else {
            registrar_conquista("normal_zerado", jogo, audio, term);
        }
        jogo.estado_atual = "FIM".to_string();
        audio.parar_ambiente();
        term.push(
            "\n[ VITÓRIA ] Você escapou com vida da unidade Vilas Boas!",
            COR_VERDE,
        );
        return true;
    }

    if jogo.sala_atual == "morte" || jogo.hp <= 0 {
        registrar_conquista("primeira_morte", jogo, audio, term);
        jogo.estado_atual = "FIM".to_string();
        audio.parar_ambiente();
        audio.som_grito();
        let _ = media.acionar_imagem("jumpscare", false, true);
        term.push(
            "\n[ SINAL PERDIDO - VOCÊ MORREU ] Digite qualquer tecla para reiniciar o terminal.",
            COR_VERMELHO,
        );
        return true;
    }

    false
}

pub fn aplicar_efeitos_pos_turno(
    jogo: &mut GameState,
    comando_correu: bool,
    term: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) {
    if !jogo.god_mode {
        if jogo.turnos_luz > 0 {
            jogo.turnos_luz -= 1;
            if jogo.turnos_luz == 0 {
                term.push(
                    "[!] A bateria da lanterna acabou! Use uma bateria reserva!",
                    COR_VERMELHO,
                );
            }
        } else {
            jogo.turnos_no_escuro += 1;
            if rand::thread_rng().gen_bool(0.4) {
                audio.som_randomico();
            }
            if jogo.turnos_no_escuro >= 3 {
                jogo.hp = 0;
                jogo.sala_atual = "morte".to_string();
                let _ = verificar_final_de_jogo(jogo, term, audio, media);
                return;
            }
        }
    }

    if comando_correu {
        jogo.nivel_barulho = 100;
        term.push(
            "Seus passos correndo ecoaram pelo corredor! (SOM: 100%)",
            COR_AMARELO,
        );
    }

    let relatorio = jogo.avancar_ia_perseguidor();
    if relatorio.combate_iniciado {
        if jogo.dev_mode {
            devmode::interceptar_perseguidor(jogo, term);
        } else {
            audio.som_grito();
            let _ = media.acionar_imagem("animatronico", false, true);
            term.push(
                "[ PERIGO IMINENTE ] O animatrônico está na sala! Digite 'correr <direção>'!",
                COR_VERMELHO,
            );
        }
    } else if relatorio.ativa && relatorio.distancia <= 2 {
        audio.som_metal();
        term.push(
            "Passos pesados de metal rangendo se aproximam nas sombras...",
            COR_VERMELHO,
        );
    }
}

pub fn processar_fluxo_jogo(
    comando_bruto: &str,
    jogo: &mut GameState,
    term: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) {
    let comando = normalizar(comando_bruto);
    if comando.is_empty() {
        return;
    }

    if devmode::eh_codigo_de_ativacao(&comando) {
        audio.iniciar_ambiente();
        devmode::ativar_modo_dev(jogo, term, media);
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
                jogo.estado_atual = "MENU".to_string();
                imprimir_menu(term, GameState::save_existe());
            } else {
                term.push(
                    "Comando inválido. Digite 'dir' para acessar os diretórios.",
                    COR_VERMELHO,
                );
            }
        }
        "MENU" => match comando.as_str() {
            "1" | "iniciar" | "novo" => {
                jogo.estado_atual = "MENU_DIFICULDADE".to_string();
                term.push(
                    "\nSelecione a dificuldade:\n  [1] NORMAL (3 HP)\n  [2] PESADELO (1 HP)\n  [voltar] Retornar",
                    COR_BRANCO,
                );
            }
            "2" | "continuar" | "carregar" => {
                if let Some(salvo) = GameState::carregar_local() {
                    *jogo = salvo;
                    audio.iniciar_ambiente();
                    term.push("Partida salva carregada!", COR_VERDE);
                    imprimir_contexto_sala(jogo, term, media);
                } else {
                    term.push("Nenhum save encontrado.", COR_VERMELHO);
                }
            }
            "3" | "conquistas" | "desafios" => {
                term.push("\n=== CONQUISTAS REGISTRADAS ===", COR_AMARELO);
                if audio.config.conquistas_globais.is_empty() {
                    term.push("Nenhuma conquista desbloqueada ainda.", COR_CINZA);
                } else {
                    for c in &audio.config.conquistas_globais {
                        term.push(&format!("  👑 {}", c), COR_VERDE);
                    }
                }
            }
            "4" | "galeria" | "imagens" | "secret" => {
                jogo.estado_atual = "MENU_GALERIA".to_string();
                let imgs: Vec<String> = MediaManager::listar_arquivos(false)
                    .into_iter()
                    .map(|(n, _)| n)
                    .collect();
                let secrets: Vec<String> = MediaManager::listar_arquivos(true)
                    .into_iter()
                    .map(|(n, _)| n)
                    .collect();
                term.push("\n=== ARQUIVOS VISUAIS DISPONÍVEIS ===", COR_CIANO);
                term.push(&format!("Imagens: {:?}", imgs), COR_BRANCO);
                term.push(&format!("Secret Images: {:?}", secrets), COR_AMARELO);
                term.push(
                    "Digite o nome do arquivo para abrir na tela ou 'voltar' para o menu.",
                    COR_VERDE,
                );
            }
            "5" | "config" => {
                term.push(
                    "Dica: Você também pode clicar no ícone de Engrenagem [⚙] no canto inferior direito!",
                    COR_CIANO,
                );
            }
            "6" | "sair" => std::process::exit(0),
            _ => imprimir_menu(term, GameState::save_existe()),
        },
        "MENU_DIFICULDADE" => match comando.as_str() {
            "1" | "normal" => {
                jogo.reiniciar_partida("NORMAL");
                audio.iniciar_ambiente();
                imprimir_contexto_sala(jogo, term, media);
            }
            "2" | "pesadelo" => {
                jogo.reiniciar_partida("PESADELO");
                audio.iniciar_ambiente();
                imprimir_contexto_sala(jogo, term, media);
            }
            "voltar" => {
                jogo.estado_atual = "MENU".to_string();
                imprimir_menu(term, GameState::save_existe());
            }
            _ => term.push("Escolha 1 (Normal), 2 (Pesadelo) ou 'voltar'.", COR_AMARELO),
        },
        "MENU_GALERIA" => {
            if comando == "voltar" || comando == "sair" {
                jogo.estado_atual = "MENU".to_string();
                imprimir_menu(term, GameState::save_existe());
            } else if !media.acionar_imagem(&comando, true, false) {
                term.push("Imagem não encontrada. Digite outro nome ou 'voltar'.", COR_VERMELHO);
            }
        }
        "FIM" => {
            media.imagem_atual = None;
            imprimir_boot(term);
            jogo.estado_atual = "AGUARDANDO_DIR".to_string();
        }
        "JOGO" | "COMBATE_ANIMATRONICO" => {
            if ["cls", "limpar", "clear", "clean"].contains(&comando.as_str()) {
                term.clear();
                imprimir_contexto_sala(jogo, term, media);
                return;
            }
            if comando == "salvar" {
                if jogo.salvar_local() {
                    term.push("Jogo salvo localmente em 'savegame.json'.", COR_VERDE);
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
                term.push("[ FLIPERAMA: FOME DE JON ] Digite 'comer' ou 'sair'.", COR_AMARELO);
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
                        COR_CIANO,
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

            let resultado_acao =
                processar_comando_acao(&comando_exec, jogo, term, audio, media);

            let mut foi_largar = false;
            if let Some(tipo) = resultado_acao {
                jogo.erros_consecutivos = 0;
                foi_largar = tipo == "largar";
                if jogo.estado_atual == "JOGO" && tipo != "olhar" {
                    aplicar_efeitos_pos_turno(jogo, jogador_correu, term, audio, media);
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
                        "O barulho dos seus tropeços no escuro atraiu algo...",
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

            if jogo.estado_atual == "FIM" && jogo.incendio {
                registrar_conquista("final_verdadeiro", jogo, audio, term);
                audio.parar_ambiente();
                let _ = media.acionar_imagem("incendio", true, false);
                term.push(
                    "\n[ FINAL VERDADEIRO ] O fogo consumiu os horrores de Vilas Boas!",
                    COR_VERDE,
                );
                return;
            }

            if jogo.atingiu_meta_acumulador() {
                registrar_conquista("acumulador", jogo, audio, term);
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
                    registrar_conquista(&ultima, jogo, audio, term);
                }
            }
            if jogo.estado_atual == "JOGO" {
                imprimir_contexto_sala(jogo, term, media);
            }
        }
    }
}