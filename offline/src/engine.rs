use crate::actions::{cmd_ir, processar_comando};
use crate::audio::AudioManager;
use crate::data::{
    salas_terminais, BARULHO_ACAO_COMUM, BARULHO_CORRER, BARULHO_ERROS_CONSECUTIVOS,
    BARULHO_IR_ABRIR, CHANCE_PERDER_ITEM_CORRER, LIMITE_BARULHO_FANTASMA, LIMITE_ERROS_DICA,
    REDUCAO_OLHAR_ESPERAR, TEMPO_SPEEDRUN_NEUTRO, TEMPO_SPEEDRUN_VERDADEIRO, TURNOS_ENJOADO_CORRER,
};
use crate::devmode;
use crate::media::MediaManager;
use crate::minigames::{
    imprimir_painel_cofre, iniciar_minigame_consertos, iniciar_minigame_jon,
    iniciar_minigame_julgamento, iniciar_minigame_minotauro, processar_minigame,
};
use crate::parser::{expandir_atalho_direcao, normalizar};
use crate::state::{agora_segundos, GameState};
use crate::ui::{DOS_AMARELO, DOS_BRANCO, DOS_VERDE, DOS_VERMELHO, RESET};
use crate::utils::{atualizar_eventos_de_tempo, desbloquear_conquista};
use crate::views::{
    dar_tela_de_morte, dar_tela_kernel_panic, imprimir_contexto_sala, imprimir_menu_dificuldade,
    imprimir_tela_boot, imprimir_tutorial, rodar_final,
};
use crate::TerminalBuffer;
use rand::Rng;

fn checar_conquistas_vitoria(
    tipo_final: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
) {
    let (conquista, pesadelo, limite) = match tipo_final {
        "final_bom" => ("final_bom", "pesadelo_neutro", TEMPO_SPEEDRUN_NEUTRO),
        "verdadeiro" => (
            "final_verdadeiro",
            "pesadelo_verdadeiro",
            TEMPO_SPEEDRUN_VERDADEIRO,
        ),
        _ => return,
    };
    desbloquear_conquista(conquista, jogo, ui, audio);
    match jogo.dificuldade_escolhida.as_str() {
        "NORMAL" => desbloquear_conquista("normal_zerado", jogo, ui, audio),
        "PESADELO" => desbloquear_conquista(pesadelo, jogo, ui, audio),
        "DESAFIO_SPEEDRUN" if (jogo.tempo_decorrido_segundos() as f64) < limite => {
            desbloquear_conquista("speedrun", jogo, ui, audio);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::MAX_TENTATIVAS_COFRE;
    use crate::ItemFilaTerminal;

    fn texto_da_tela(ui: &TerminalBuffer) -> String {
        ui.fila
            .iter()
            .filter_map(|item| match item {
                ItemFilaTerminal::Linha { linha, .. } => Some(
                    linha
                        .segmentos
                        .iter()
                        .map(|segmento| segmento.texto.as_str())
                        .collect::<String>(),
                ),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn inicia_explora_salva_e_restaura_pelo_menu() {
        let mut jogo = GameState::new();
        let mut ui = TerminalBuffer::default();
        let mut audio = AudioManager::new();
        let mut media = MediaManager::new();
        assert_eq!(jogo.estado_atual, "AGUARDANDO_DIR");

        processar_fluxo_jogo("dir", &mut jogo, &mut ui, &mut audio, &mut media);
        assert_eq!(jogo.estado_atual, "MENU");
        assert!(texto_da_tela(&ui).contains("68.097 bytes"));

        processar_fluxo_jogo("2", &mut jogo, &mut ui, &mut audio, &mut media);
        assert_eq!(jogo.estado_atual, "JOGO");
        assert!(jogo.fast_mode);
        assert_eq!(jogo.sala_atual, "entrada");
        jogo.dev_ia_congelada = true;
        jogo.eventos_aleatorios.clear();

        processar_fluxo_jogo("pegar tábua", &mut jogo, &mut ui, &mut audio, &mut media);
        assert_eq!(jogo.inventario.len(), 2);
        assert!(jogo.mapa["entrada"].itens.is_empty());
        processar_fluxo_jogo("norte", &mut jogo, &mut ui, &mut audio, &mut media);
        assert_eq!(jogo.sala_atual, "sala de jantar");

        let mut nova_sessao = GameState::new();
        processar_fluxo_jogo("dir", &mut nova_sessao, &mut ui, &mut audio, &mut media);
        processar_fluxo_jogo("7", &mut nova_sessao, &mut ui, &mut audio, &mut media);
        assert_eq!(nova_sessao.sala_atual, "sala de jantar");
        assert_eq!(nova_sessao.inventario, jogo.inventario);
    }

    #[test]
    fn entrada_do_minotauro_e_cofre_retentavel() {
        let mut jogo = GameState::new();
        let mut ui = TerminalBuffer::default();
        let mut audio = AudioManager::new();
        let mut media = MediaManager::new();
        jogo.estado_atual = "JOGO".to_string();
        jogo.sala_atual = "sala dos fundos".to_string();
        jogo.eventos_aleatorios.clear();
        jogo.dev_ia_congelada = true;

        processar_fluxo_jogo("ir energia", &mut jogo, &mut ui, &mut audio, &mut media);
        assert_eq!(jogo.sala_atual, "sala de energia");
        processar_fluxo_jogo("olhar", &mut jogo, &mut ui, &mut audio, &mut media);
        assert_eq!(jogo.estado_atual, "MINIGAME_MINOTAURO");

        jogo.estado_atual = "JOGO".to_string();
        jogo.sala_atual = "01".to_string();
        processar_fluxo_jogo("abrir cofre", &mut jogo, &mut ui, &mut audio, &mut media);
        for _ in 0..MAX_TENTATIVAS_COFRE {
            processar_fluxo_jogo("1111", &mut jogo, &mut ui, &mut audio, &mut media);
        }
        assert_eq!(jogo.estado_atual, "JOGO");
        assert!(texto_da_tela(&ui).contains("aceite tentar de novo"));
        processar_fluxo_jogo("abrir cofre", &mut jogo, &mut ui, &mut audio, &mut media);
        processar_fluxo_jogo("1994", &mut jogo, &mut ui, &mut audio, &mut media);
        assert!(jogo
            .inventario
            .iter()
            .any(|item| item == "chave dos fundos"));
    }

    #[test]
    fn volta_ao_hall_depois_da_noite_sem_comando_extra() {
        let mut jogo = GameState::new();
        let mut ui = TerminalBuffer::default();
        let mut audio = AudioManager::new();
        let mut media = MediaManager::new();
        jogo.sala_atual = "hall de entrada".to_string();
        jogo.estado_atual = "JOGO".to_string();
        jogo.noite_vencida = true;
        jogo.fios_cortados_inventario = true;

        processar_fluxo_jogo("olhar", &mut jogo, &mut ui, &mut audio, &mut media);
        assert_eq!(jogo.estado_atual, "FIM");
        assert!(jogo.conquistas.iter().any(|id| id == "final_verdadeiro"));
        assert!(texto_da_tela(&ui).contains("[ FINAL VERDADEIRO: LIBERTAÇÃO ]"));
    }

    #[test]
    fn fliperamas_entregam_o_cartao_que_desbloqueia_a_seguranca() {
        let mut jogo = GameState::new();
        let mut ui = TerminalBuffer::default();
        let mut audio = AudioManager::new();
        let mut media = MediaManager::new();
        jogo.estado_atual = "JOGO".to_string();
        jogo.sala_atual = "sala de fliperamas".to_string();

        processar_fluxo_jogo("jogar jon", &mut jogo, &mut ui, &mut audio, &mut media);
        for passo in jogo.jon_caminho_certo.clone() {
            processar_fluxo_jogo(&passo, &mut jogo, &mut ui, &mut audio, &mut media);
        }
        assert!(jogo.inventario.iter().any(|item| item == "moeda velha"));

        for comando in [
            "jogar consertos",
            "2",
            "1",
            "2",
            "jogar julgamento",
            "1995",
            "Caroline",
            "1982",
            "Rogério",
            "Angela",
            "João",
            "Renato",
        ] {
            processar_fluxo_jogo(comando, &mut jogo, &mut ui, &mut audio, &mut media);
        }
        assert!(jogo
            .inventario
            .iter()
            .any(|item| item == "cartao de seguranca"));

        jogo.sala_atual = "01".to_string();
        processar_fluxo_jogo("sentar", &mut jogo, &mut ui, &mut audio, &mut media);
        assert_eq!(jogo.estado_atual, "MINIGAME_SEGURANCA");
        assert_eq!(jogo.seguranca_dados.unwrap().turno, 0);
    }
}

fn verificar_final_de_jogo(
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) -> bool {
    let final_alcancado = match jogo.sala_atual.as_str() {
        "morte" => {
            dar_tela_de_morte(jogo, ui, audio, media);
            desbloquear_conquista("primeira_morte", jogo, ui, audio);
            return true;
        }
        "saida" => "saida",
        "cama" => "cama",
        "hall de entrada" if jogo.noite_vencida && jogo.fios_cortados_inventario => "verdadeiro",
        "hall de entrada" if jogo.noite_vencida => "final_bom",
        _ => return false,
    };
    jogo.tempo_total_segundos = jogo.tempo_decorrido_segundos() as f64;
    rodar_final(final_alcancado, jogo, ui, audio, media);
    checar_conquistas_vitoria(final_alcancado, jogo, ui, audio);
    true
}

fn aplicar_efeitos_pos_turno(
    comando: &str,
    correu: bool,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
) {
    if correu {
        jogo.nivel_barulho = BARULHO_CORRER;
        jogo.ai_alvo = Some(jogo.sala_atual.clone());
        jogo.turnos_enjoado = TURNOS_ENJOADO_CORRER;
        let mut rng = rand::thread_rng();
        if rng.gen::<f64>() <= CHANCE_PERDER_ITEM_CORRER && !jogo.inventario.is_empty() {
            let indice = rng.gen_range(0..jogo.inventario.len());
            let item = jogo.inventario.remove(indice);
            if let Some(sala) = jogo.mapa.get_mut(&jogo.sala_atual) {
                sala.itens.push(item.clone());
            }
            ui.animar(
                &format!(
                    "Você corre desesperado. No pânico, você tropeça e deixa cair seu(sua) {}!",
                    item.to_uppercase()
                ),
                DOS_VERMELHO,
            );
        } else {
            ui.animar(
                "Você foge correndo! Você deixa tudo para trás num piscar de olhos",
                DOS_VERMELHO,
            );
        }
        ui.animar(
            "O animatrônico te perde de vista enquanto corre.",
            DOS_AMARELO,
        );
    } else {
        let delta = if comando.starts_with("ir ") || comando.starts_with("abrir ") {
            BARULHO_IR_ABRIR
        } else if ["olhar", "inventario", "esperar", "aguardar", "z"].contains(&comando)
            || crate::actions::COMANDOS_ESCONDER.contains(&comando)
        {
            -REDUCAO_OLHAR_ESPERAR
        } else {
            BARULHO_ACAO_COMUM
        };
        jogo.alterar_barulho(delta);

        let ia = jogo.avancar_ia_perseguidor();
        if ia.ativa {
            if ia.passou_escondido {
                ui.animar(
                    "O animatrônico atravessa a sala, fareja o ar... e segue adiante sem te ver.",
                    DOS_AMARELO,
                );
            } else if ia.combate_iniciado {
                ui.animar(
                    "O animatrônico irrompe na sala! Ele encontrou você!",
                    DOS_VERMELHO,
                );
            } else if ia.distancia == 1 {
                ui.buffer_efeito("@@PASSO@@");
                ui.exibir("Você ouve passos metálicos pesados na sala ao lado...");
            } else if ia.distancia == 2 {
                ui.animar(
                    "Um baque surdo ecoa pelos corredores distantes.",
                    DOS_AMARELO,
                );
            }
        }
    }

    if jogo.dificuldade_escolhida == "DESAFIO_FANTASMA"
        && jogo.nivel_barulho > LIMITE_BARULHO_FANTASMA
    {
        ui.exibir(&format!(
            "\n{DOS_VERMELHO}Você quebrou o silêncio. Eles ouviram.{RESET}"
        ));
        ui.pausa(1.5);
        jogo.sala_atual = "morte".to_string();
        jogo.causa_morte = "fantasma".to_string();
    }
}

/// Fuga bem-sucedida da "Ela": custa HP, mas a deixa atordoada.
fn fugir_da_ela(jogo: &mut GameState, ui: &mut TerminalBuffer) {
    let dano = jogo.perfil_ela().dano_fuga;
    if jogo.aplicar_dano(dano) {
        jogo.causa_morte = "ela".to_string();
        ui.exibir(&format!(
            "\n{DOS_VERMELHO}As garras dela rasgam suas costas enquanto você foge. Você não chega longe...{RESET}"
        ));
        return;
    }
    crate::ela::atordoar_apos_fuga(jogo);
    ui.exibir(&format!(
        "\n{DOS_VERMELHO}As garras dela rasgam suas costas, mas você escapa! (-{dano} HP){RESET}"
    ));
    ui.exibir(&format!(
        "{DOS_AMARELO}Ela bate contra o batente da porta e fica desorientada por alguns instantes.{RESET}"
    ));
}

fn imprimir_diretorio(jogo: &mut GameState, ui: &mut TerminalBuffer, audio: &AudioManager) {
    ui.limpar();
    ui.exibir(&format!(
        "{DOS_BRANCO} Volume in drive A is VILLASBOAS{RESET}"
    ));
    ui.exibir(&format!("{DOS_BRANCO} Directory of A:\\{RESET}\n\n"));
    ui.exibir(&format!(
        "{DOS_VERDE}COMMAND  COM        47.845  02-11-1982  6:00a{RESET}\n"
    ));
    ui.exibir(&format!(
        "{DOS_VERDE}SEGURA   SYS         2.048  02-11-1982  6:00a{RESET}\n"
    ));
    ui.exibir(&format!(
        "{DOS_VERDE}NOTURNO  EXE        18.204  02-11-1982  6:00a{RESET}\n"
    ));
    ui.exibir(&format!(
        "{DOS_AMARELO}         3 file(s)       68.097 bytes{RESET}"
    ));
    ui.exibir(&format!(
        "{DOS_VERDE}=================================================={RESET}"
    ));
    jogo.estado_atual = "MENU".to_string();
    imprimir_menu_dificuldade(
        ui,
        GameState::save_existe(),
        &audio.config.finais_desbloqueados,
    );
}

pub fn iniciar_partida_com_tutorial(
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) {
    jogo.estado_atual = "JOGO".to_string();
    jogo.tempo_inicio = agora_segundos();
    jogo.tempo_total_segundos = 0.0;
    audio.iniciar_ambiente();
    ui.limpar();
    ui.animar(
        &format!(
            "MODO {} SELECIONADO. BOA SORTE.",
            jogo.dificuldade_escolhida
        ),
        DOS_VERDE,
    );
    imprimir_tutorial(ui, jogo);
    imprimir_contexto_sala(jogo, ui, media);
    let _ = jogo.salvar_local();
}

pub fn processar_fluxo_jogo(
    comando_bruto: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) {
    let comando_norm = normalizar(comando_bruto);

    if jogo.estado_atual == "FIM" {
        match comando_norm.as_str() {
            "f5" | "reiniciar" | "restart" | "reset" | "dir" => {
                media.imagem_atual = None;
                jogo.resetar_para_novo_jogo();
                if comando_norm == "dir" {
                    imprimir_diretorio(jogo, ui, audio);
                } else {
                    imprimir_tela_boot(ui);
                }
            }
            _ => ui.exibir(&format!(
                "{DOS_VERMELHO}[SISTEMA BLOQUEADO] - Aperte a tecla F5 no teclado para jogar novamente.{RESET}"
            )),
        }
        return;
    }

    if !jogo.mapa.contains_key(&jogo.sala_atual)
        && !salas_terminais().contains(&jogo.sala_atual.as_str())
    {
        jogo.sala_atual = "01".to_string();
    }

    if comando_norm.is_empty() {
        return;
    }

    jogo.registrar_comando(comando_bruto.trim());

    // Ativação global do Modo Desenvolvedor ("sudo admin 1982")
    if devmode::eh_codigo_de_ativacao(&comando_norm) {
        audio.iniciar_ambiente();
        devmode::ativar_modo_dev(jogo, ui, media);
        desbloquear_conquista("trapaceiro", jogo, ui, audio);
        return;
    }

    if (jogo.dev_mode || jogo.god_mode)
        && devmode::processar_comando_dev(&comando_norm, jogo, ui, audio, media)
    {
        return;
    }

    match jogo.estado_atual.as_str() {
        "AGUARDANDO_DIR" => {
            if comando_norm == "dir" {
                imprimir_diretorio(jogo, ui, audio);
            } else if ["cls", "limpar", "clear", "clean"].contains(&comando_norm.as_str()) {
                imprimir_tela_boot(ui);
            } else if comando_norm == "help" {
                ui.exibir(&format!(
                    "{}[ERRO DO SISTEMA]: O arquivo AJUDA.COM foi corrompido.{}",
                    DOS_VERMELHO, RESET
                ));
            } else if [
                "del caroline.sys",
                "delete caroline.sys",
                "erase caroline.sys",
                "format a:",
            ]
            .contains(&comando_norm.as_str())
            {
                ui.exibir(&format!(
                    "{DOS_VERMELHO}Acesso Negado. O arquivo CAROLINE.SYS está enraizado na sua mente.{RESET}"
                ));
                ui.pausa(2.0);
                dar_tela_kernel_panic(jogo, ui);
            } else if comando_norm == "exit" {
                ui.buffer_efeito("@@EXIT@@");
            } else if comando_norm == "shutdown" {
                ui.buffer_efeito("@@RELOAD@@");
            } else {
                ui.exibir(&format!(
                    "{DOS_VERMELHO}Bad command or file name{RESET}\n{DOS_VERDE}Digite {DOS_BRANCO}dir{DOS_VERDE} para acessar os diretórios:{RESET}"
                ));
            }
        }

        "MENU" | "MENU_DIFICULDADE" => match comando_norm.as_str() {
            "cls" | "limpar" | "clear" | "clean" => {
                ui.limpar();
                imprimir_menu_dificuldade(
                    ui,
                    GameState::save_existe(),
                    &audio.config.finais_desbloqueados,
                );
            }
            "1" => {
                jogo.configurar_dificuldade("NORMAL", 1.0);
                iniciar_partida_com_tutorial(jogo, ui, audio, media);
            }
            "2" => {
                jogo.configurar_dificuldade("NORMAL", 0.0);
                iniciar_partida_com_tutorial(jogo, ui, audio, media);
            }
            "3" => {
                jogo.configurar_dificuldade("PESADELO", 1.0);
                iniciar_partida_com_tutorial(jogo, ui, audio, media);
            }
            "4" => {
                jogo.configurar_dificuldade("PESADELO", 0.0);
                iniciar_partida_com_tutorial(jogo, ui, audio, media);
            }
            "5" => {
                jogo.estado_atual = "MENU_DESAFIOS".to_string();
                ui.limpar();
                ui.animar(
                    "==================================================",
                    DOS_AMARELO,
                );
                ui.animar(
                    "             [ MODOS DE DESAFIO ]                 ",
                    DOS_AMARELO,
                );
                ui.animar(
                    "==================================================",
                    DOS_AMARELO,
                );
                ui.exibir(&format!(
                    "{}[1] SPEEDRUN (Vença em menos de 5 minutos)\n\
                    [2] MODO FANTASMA (Morte instantânea se o SOM passar de 50%)\n\
                    [3] BREU TOTAL (Lanterna quebrada desde o início)\n\
                    [4] Voltar ao Menu Principal{}",
                    DOS_BRANCO, RESET
                ));
            }
            "6" => {
                jogo.estado_atual = "MENU_CUSTOM_BAT".to_string();
                ui.limpar();
                ui.animar("=== MODO PERSONALIZADO (1/3) ===", DOS_AMARELO);
                ui.exibir(&format!(
                    "{}Digite a duração de turnos de cada bateria (1 a 99, padrão 12):{}",
                    DOS_VERDE, RESET
                ));
            }
            "7" if GameState::save_existe() => {
                if let Some(salvo) = GameState::carregar_local() {
                    *jogo = salvo;
                    audio.iniciar_ambiente();
                    ui.limpar();
                    ui.exibir(&format!(
                        "{}[SISTEMA]: Autosave carregado com sucesso.{}",
                        DOS_VERDE, RESET
                    ));
                    imprimir_contexto_sala(jogo, ui, media);
                }
            }
            "2007" => {
                jogo.configurar_dificuldade("GOD MODE", 1.0);
                desbloquear_conquista("trapaceiro", jogo, ui, audio);
                iniciar_partida_com_tutorial(jogo, ui, audio, media);
            }
            "1980" => {
                ui.buffer_efeito("@@PLAY_1980_SLIDESHOW@@");
            }
            _ => {
                ui.exibir(&format!(
                    "{}Opção inválida. Escolha entre as opções listadas.{}",
                    DOS_VERMELHO, RESET
                ));
            }
        },

        "MENU_DESAFIOS" => match comando_norm.as_str() {
            "1" => {
                jogo.configurar_dificuldade("DESAFIO_SPEEDRUN", 0.0);
                iniciar_partida_com_tutorial(jogo, ui, audio, media);
            }
            "2" => {
                jogo.configurar_dificuldade("DESAFIO_FANTASMA", 1.0);
                iniciar_partida_com_tutorial(jogo, ui, audio, media);
            }
            "3" => {
                jogo.configurar_dificuldade("DESAFIO_BREU", 1.0);
                iniciar_partida_com_tutorial(jogo, ui, audio, media);
            }
            "4" | "voltar" => {
                jogo.estado_atual = "MENU".to_string();
                ui.limpar();
                imprimir_menu_dificuldade(
                    ui,
                    GameState::save_existe(),
                    &audio.config.finais_desbloqueados,
                );
            }
            _ => ui.exibir(&format!("{DOS_VERMELHO}Opção inválida.{RESET}")),
        },

        "MENU_CUSTOM_BAT" => {
            if let Ok(val) = comando_norm.parse::<i32>() {
                jogo.custom_bat = val.clamp(1, 99);
                jogo.estado_atual = "MENU_CUSTOM_INV".to_string();
                ui.animar("\n=== MODO PERSONALIZADO (2/3) ===", DOS_AMARELO);
                ui.exibir(&format!(
                    "{}Digite o limite inicial de espaços na mochila (1 a 10, padrão 3):{}",
                    DOS_VERDE, RESET
                ));
            } else {
                ui.exibir(&format!(
                    "{}Por favor, digite um número válido entre 1 e 99.{}",
                    DOS_VERMELHO, RESET
                ));
            }
        }

        "MENU_CUSTOM_INV" => {
            if let Ok(val) = comando_norm.parse::<usize>() {
                jogo.custom_inv = val.clamp(1, 10);
                jogo.estado_atual = "MENU_CUSTOM_HP".to_string();
                ui.animar("\n=== MODO PERSONALIZADO (3/3) ===", DOS_AMARELO);
                ui.exibir(&format!(
                    "{}Digite os pontos de vida iniciais (1 a 10, padrão 3):{}",
                    DOS_VERDE, RESET
                ));
            } else {
                ui.exibir(&format!(
                    "{}Por favor, digite um número válido entre 1 e 10.{}",
                    DOS_VERMELHO, RESET
                ));
            }
        }

        "MENU_CUSTOM_HP" => {
            if let Ok(val) = comando_norm.parse::<i32>() {
                jogo.custom_hp = val.clamp(1, 10);
                jogo.configurar_dificuldade("CUSTOM", 0.0);
                iniciar_partida_com_tutorial(jogo, ui, audio, media);
            } else {
                ui.exibir(&format!(
                    "{}Por favor, digite um número válido entre 1 e 10.{}",
                    DOS_VERMELHO, RESET
                ));
            }
        }

        "JOGO" | "COMBATE_ANIMATRONICO" => {
            let comando = expandir_atalho_direcao(&comando_norm, &jogo.sinonimos);

            if ["cls", "limpar", "clear", "clean"].contains(&comando.as_str()) {
                ui.limpar();
                imprimir_contexto_sala(jogo, ui, media);
                return;
            }
            if ["ajuda", "help", "manual"].contains(&comando.as_str()) {
                imprimir_tutorial(ui, jogo);
                return;
            }
            if comando == "salvar" {
                if jogo.salvar_local() {
                    ui.exibir(&format!(
                        "{}[SISTEMA]: Progresso salvo localmente com sucesso.{}",
                        DOS_VERDE, RESET
                    ));
                }
                return;
            }

            let snapshot = devmode::tirar_snapshot(jogo);
            let estado_antes = jogo.estado_atual.clone();

            // 1. Gatilhos especiais de salas e minigames (antes do parser padrão)
            if jogo.estado_atual == "JOGO" {
                if jogo.sala_atual == "sala de energia" && !jogo.fios_cortados_inventario {
                    iniciar_minigame_minotauro(jogo, ui);
                    return;
                }

                if jogo.sala_atual != "01"
                    && ["cadeira", "sentar", "sentar na cadeira", "usar cadeira"]
                        .contains(&comando.as_str())
                {
                    ui.exibir("Não há nenhuma cadeira de segurança aqui.");
                    return;
                }

                // Sala 01: Cofre e Cadeira de Segurança
                if jogo.sala_atual == "01" {
                    if [
                        "abrir cofre",
                        "usar cofre",
                        "cofre",
                        "examinar cofre",
                        "olhar cofre",
                    ]
                    .contains(&comando.as_str())
                    {
                        jogo.estado_atual = "MINIGAME_COFRE".to_string();
                        jogo.cofre_tentativas = 0;
                        imprimir_painel_cofre(jogo, ui, media, true);
                        return;
                    }

                    if [
                        "sentar",
                        "sentar na cadeira",
                        "usar cadeira",
                        "cadeira",
                        "iniciar turno",
                        "usar cartao de seguranca",
                    ]
                    .contains(&comando.as_str())
                    {
                        if jogo.noite_vencida {
                            ui.exibir(&format!(
                                "{}O turno das 06:00 AM já terminou. Vá para o 'hall' decidir o seu destino.{}",
                                DOS_VERDE, RESET
                            ));
                            return;
                        }
                        if jogo.inventario.iter().any(|i| i == "cartao de seguranca")
                            || jogo.god_mode
                        {
                            crate::seguranca::iniciar(jogo, ui);
                        } else {
                            ui.exibir(&format!(
                                "{}O terminal de segurança exige o 'cartao de seguranca' para iniciar o turno da noite.{}",
                                DOS_VERMELHO, RESET
                            ));
                        }
                        return;
                    }
                }

                // Salão de Jogos: 3 Minigames (Jon, Consertos, Julgamento)
                if jogo.sala_atual == "salao de jogos" || jogo.sala_atual == "sala de fliperamas" {
                    if [
                        "jogar jon",
                        "jogar fome de jon",
                        "fome de jon",
                        "usar fliperama",
                        "jogar fliperama",
                    ]
                    .contains(&comando.as_str())
                    {
                        iniciar_minigame_jon(jogo, ui);
                        return;
                    }

                    if [
                        "jogar consertos",
                        "consertos",
                        "consertos e sorrisos",
                        "usar moeda",
                        "usar moeda velha",
                    ]
                    .contains(&comando.as_str())
                    {
                        if jogo.inventario.iter().any(|i| i == "moeda velha") {
                            iniciar_minigame_consertos(jogo, ui);
                        } else {
                            ui.exibir(&format!(
                                "{}O fliperama 'Consertos & Sorrisos' pede uma 'moeda velha' para funcionar.{}",
                                DOS_VERMELHO, RESET
                            ));
                        }
                        return;
                    }

                    if [
                        "tocar piano",
                        "usar piano",
                        "jogar piano",
                        "piano",
                        "jogar adivinha",
                        "jogar julgamento",
                        "usar partitura",
                        "usar partitura rasgada",
                    ]
                    .contains(&comando.as_str())
                    {
                        if jogo.inventario.iter().any(|i| i == "partitura rasgada") {
                            iniciar_minigame_julgamento(jogo, ui);
                        } else {
                            ui.exibir(&format!(
                                "{}As teclas estão mudas. Parece faltar uma 'partitura rasgada' no suporte.{}",
                                DOS_VERMELHO, RESET
                            ));
                        }
                        return;
                    }
                }
            }

            let correu = comando.starts_with("correr ");
            let mut fugindo_de = String::new();
            jogo.escondido = false;
            let acao = if let Some(direcao) = comando.strip_prefix("correr ") {
                let entrada = format!("ir {direcao}");
                let movimento = expandir_atalho_direcao(&entrada, &jogo.sinonimos);
                if jogo.estado_atual == "COMBATE_ANIMATRONICO" {
                    fugindo_de = if jogo.atacante_combate.is_empty() {
                        "animatronico".to_string()
                    } else {
                        jogo.atacante_combate.clone()
                    };
                }
                jogo.estado_atual = "JOGO".to_string();
                if cmd_ir(
                    movimento.strip_prefix("ir ").unwrap_or(direcao),
                    jogo,
                    ui,
                    audio,
                    media,
                ) {
                    Some("ir".to_string())
                } else {
                    jogo.estado_atual = estado_antes;
                    None
                }
            } else {
                processar_comando(&comando, jogo, ui, audio, media)
            };

            if acao.is_some() && !fugindo_de.is_empty() {
                jogo.atacante_combate.clear();
                if fugindo_de == "ela" {
                    fugir_da_ela(jogo, ui);
                }
            }

            if let Some(acao) = &acao {
                if jogo.estado_atual != "FIM" {
                    atualizar_eventos_de_tempo(jogo, ui, audio, media);
                }
                jogo.erros_consecutivos = 0;
                if jogo.estado_atual == "JOGO" && acao != "olhar" {
                    aplicar_efeitos_pos_turno(&comando, correu, jogo, ui);
                }
            } else {
                jogo.erros_consecutivos += 1;
                if jogo.erros_consecutivos >= LIMITE_ERROS_DICA {
                    jogo.alterar_barulho(BARULHO_ERROS_CONSECUTIVOS);
                    jogo.ai_alvo = Some(jogo.sala_atual.clone());
                    ui.animar(
                        "Seus tropeços no escuro estão chamando muita atenção...",
                        DOS_AMARELO,
                    );
                    jogo.erros_consecutivos = 0;
                }
            }

            if jogo.atingiu_meta_acumulador() {
                desbloquear_conquista("acumulador", jogo, ui, audio);
            }
            if jogo.estado_atual != "FIM"
                && !verificar_final_de_jogo(jogo, ui, audio, media)
                && jogo.estado_atual == "JOGO"
            {
                imprimir_contexto_sala(jogo, ui, media);
            }
            if jogo.dev_mode || jogo.god_mode {
                devmode::pos_turno(jogo, snapshot, acao.as_deref() == Some("largar"), ui);
            }
            let _ = jogo.salvar_local();
        }

        // Qualquer estado de MINIGAME_*
        _ => {
            if processar_minigame(&comando_norm, jogo, ui, audio, media) {
                if jogo.sala_atual == "morte" {
                    desbloquear_conquista("primeira_morte", jogo, ui, audio);
                }
                let _ = jogo.salvar_local();
            }
        }
    }
}
