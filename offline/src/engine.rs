use crate::actions::{cmd_ir, processar_comando};
use crate::audio::AudioManager;
use crate::devmode;
use crate::media::MediaManager;
use crate::minigames::{
    imprimir_painel_cofre, iniciar_minigame_consertos, iniciar_minigame_jon,
    iniciar_minigame_julgamento, iniciar_minigame_minotauro, iniciar_minigame_seguranca,
    processar_minigame,
};
use crate::parser::{expandir_atalho_direcao, normalizar};
use crate::state::GameState;
use crate::ui::{DOS_AMARELO, DOS_BRANCO, DOS_VERDE, DOS_VERMELHO, RESET};
use crate::utils::{atualizar_eventos_de_tempo, desbloquear_conquista};
use crate::views::{
    dar_tela_de_morte, dar_tela_kernel_panic, imprimir_contexto_sala, imprimir_menu_dificuldade,
    imprimir_tela_boot, imprimir_tutorial, rodar_final,
};
use crate::TerminalBuffer;
use rand::seq::SliceRandom;
use rand::Rng;

fn checar_conquistas_vitoria(
    tipo_final: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
) {
    let dif = jogo.dificuldade.clone();

    if dif == "NORMAL" {
        desbloquear_conquista("normal_zerado", jogo, ui, audio);
    }
    if tipo_final == "final_bom" {
        desbloquear_conquista("final_bom", jogo, ui, audio);
        if dif == "PESADELO" {
            desbloquear_conquista("pesadelo_neutro", jogo, ui, audio);
        }
    }
    if tipo_final == "verdadeiro" {
        desbloquear_conquista("final_verdadeiro", jogo, ui, audio);
        if dif == "PESADELO" {
            desbloquear_conquista("pesadelo_verdadeiro", jogo, ui, audio);
        }
    }

    if dif == "DESAFIO_SPEEDRUN" || jogo.tempo_decorrido_segundos() <= 300 {
        desbloquear_conquista("speedrun", jogo, ui, audio);
    }
    if dif == "DESAFIO_FANTASMA" {
        desbloquear_conquista("fantasma", jogo, ui, audio);
    }
    if dif == "DESAFIO_BREU" {
        desbloquear_conquista("breu_total", jogo, ui, audio);
    }
}

pub fn iniciar_partida_com_tutorial(
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) {
    audio.iniciar_ambiente();
    ui.limpar();
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

    // Se estiver na tela de FIM, qualquer comando/Enter reinicia para o boot
    if jogo.estado_atual == "FIM" {
        media.imagem_atual = None;
        let conquistas_mantidas = jogo.conquistas.clone();
        *jogo = GameState::novo();
        jogo.conquistas = conquistas_mantidas;
        imprimir_tela_boot(ui);
        return;
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
                ui.limpar();
                ui.animar(" Volume in drive A is VILLAS-BOAS", DOS_BRANCO);
                ui.animar(" Directory of A:\\\n", DOS_BRANCO);
                ui.animar("COMMAND  COM     25,276 02-11-82   2:00a", DOS_VERDE);
                ui.animar("SEGURA   SYS     64,102 02-11-82   2:00a", DOS_VERDE);
                ui.animar("NOTURNO  EXE    128,512 14-10-95  11:59p", DOS_VERDE);
                ui.animar("        3 file(s)        217,890 bytes\n", DOS_BRANCO);

                jogo.estado_atual = "MENU".to_string();
                imprimir_menu_dificuldade(
                    ui,
                    GameState::save_existe(),
                    &audio.config.finais_desbloqueados,
                );
            } else if ["cls", "clear"].contains(&comando_norm.as_str()) {
                imprimir_tela_boot(ui);
            } else if comando_norm == "help" {
                ui.exibir(&format!(
                    "{}O arquivo AJUDA.COM foi corrompido. Digite 'dir' para iniciar.{}",
                    DOS_VERMELHO, RESET
                ));
            } else if [
                "del caroline.sys",
                "delete caroline.sys",
                "erase caroline.sys",
                "format a:",
                "format a",
            ]
            .contains(&comando_norm.as_str())
            {
                dar_tela_kernel_panic(jogo, ui);
            } else if comando_norm == "exit" {
                ui.buffer_efeito("@@EXIT@@");
            } else if comando_norm == "shutdown" {
                ui.buffer_efeito("@@RELOAD@@");
            } else {
                ui.exibir(&format!(
                    "{}Bad command or file name. Digite 'dir' para acessar os diretórios.{}",
                    DOS_VERMELHO, RESET
                ));
            }
        }

        "MENU" | "MENU_DIFICULDADE" => match comando_norm.as_str() {
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
                ui.animar(
                    "=== MODO PERSONALIZADO (1/3) ===",
                    DOS_AMARELO,
                );
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
                jogo.configurar_dificuldade("DEV", 0.0);
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
                jogo.configurar_dificuldade("DESAFIO_FANTASMA", 0.0);
                iniciar_partida_com_tutorial(jogo, ui, audio, media);
            }
            "3" => {
                jogo.configurar_dificuldade("DESAFIO_BREU", 0.0);
                iniciar_partida_com_tutorial(jogo, ui, audio, media);
            }
            _ => {
                jogo.estado_atual = "MENU".to_string();
                ui.limpar();
                imprimir_menu_dificuldade(
                    ui,
                    GameState::save_existe(),
                    &audio.config.finais_desbloqueados,
                );
            }
        },

        "MENU_CUSTOM_BAT" => {
            if let Ok(val) = comando_norm.parse::<i32>() {
                jogo.custom_bat = val.clamp(1, 99);
                jogo.estado_atual = "MENU_CUSTOM_INV".to_string();
                ui.animar(
                    "\n=== MODO PERSONALIZADO (2/3) ===",
                    DOS_AMARELO,
                );
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
                ui.animar(
                    "\n=== MODO PERSONALIZADO (3/3) ===",
                    DOS_AMARELO,
                );
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
                // Sala dos Fundos -> Sala de Energia (Labirinto do Minotauro)
                if jogo.sala_atual == "sala dos fundos"
                    && [
                        "ir frente",
                        "frente",
                        "ir sala de energia",
                        "sala de energia",
                    ]
                    .contains(&comando.as_str())
                {
                    if !jogo.fios_cortados_inventario {
                        iniciar_minigame_minotauro(jogo, ui);
                        return;
                    } else {
                        ui.exibir(&format!(
                            "{}Você já cortou os cabos de alta tensão lá dentro. Não há motivo para voltar àquela escuridão.{}",
                            DOS_AMARELO, RESET
                        ));
                        return;
                    }
                }

                // Cozinha -> Despensa (destrancar com chave da cozinha)
                if jogo.sala_atual == "cozinha"
                    && [
                        "ir frente",
                        "frente",
                        "ir despensa",
                        "despensa",
                        "usar chave da cozinha",
                        "abrir porta",
                    ]
                    .contains(&comando.as_str())
                {
                    let trancada = jogo
                        .mapa
                        .get("cozinha")
                        .and_then(|s| s.conexoes.get("frente"))
                        .map(|opt| opt.is_none())
                        .unwrap_or(false);

                    if trancada {
                        if jogo.inventario.iter().any(|i| i == "chave da cozinha") || jogo.god_mode
                        {
                            jogo.inventario.retain(|i| i != "chave da cozinha");
                            if let Some(cozinha) = jogo.mapa.get_mut("cozinha") {
                                cozinha
                                    .conexoes
                                    .insert("frente".to_string(), Some("despensa".to_string()));
                            }
                            ui.limpar();
                            ui.animar(
                                "Você usa a 'chave da cozinha' para destrancar o cadeado enferrujado da Despensa!",
                                DOS_VERDE,
                            );
                            jogo.sala_atual = "despensa".to_string();
                            atualizar_eventos_de_tempo(jogo, ui, audio, media);
                            imprimir_contexto_sala(jogo, ui, media);
                            return;
                        }
                    }
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
                        if jogo.cofre_aberto {
                            ui.exibir(&format!(
                                "{}O cofre já está aberto e vazio.{}",
                                DOS_AMARELO, RESET
                            ));
                        } else if jogo.cofre_tentativas >= 3 {
                            ui.exibir(&format!(
                                "{}O mecanismo eletrônico do cofre travou permanentemente.{}",
                                DOS_VERMELHO, RESET
                            ));
                        } else {
                            jogo.estado_atual = "MINIGAME_COFRE".to_string();
                            ui.limpar();
                            imprimir_painel_cofre(jogo, ui, media, true);
                        }
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
                        if jogo
                            .inventario
                            .iter()
                            .any(|i| i == "cartao de seguranca")
                            || jogo.god_mode
                        {
                            iniciar_minigame_seguranca(jogo, ui);
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
                        if jogo.jon_vencido {
                            ui.exibir(&format!(
                                "{}A tela do arcade 'A Fome de Jon' está desligada. Você já venceu este jogo.{}",
                                DOS_AMARELO, RESET
                            ));
                        } else {
                            iniciar_minigame_jon(jogo, ui);
                        }
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
                        if jogo.consertos_vencido {
                            ui.exibir(&format!(
                                "{}A máquina 'Consertos & Sorrisos' já entregou todos os prêmios.{}",
                                DOS_AMARELO, RESET
                            ));
                        } else if jogo.inventario.iter().any(|i| i == "moeda velha")
                            || jogo.god_mode
                        {
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
                        "usar partitura",
                        "usar partitura rasgada",
                    ]
                    .contains(&comando.as_str())
                    {
                        if jogo.julgamento_vencido {
                            ui.exibir(&format!(
                                "{}A tampa do piano permanece trancada em silêncio absoluto.{}",
                                DOS_AMARELO, RESET
                            ));
                        } else if jogo.inventario.iter().any(|i| i == "partitura rasgada")
                            || jogo.god_mode
                        {
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

                // Hall após 06:00 AM: Usar isqueiro / queimar cortinas -> Final Neutro ou Final Verdadeiro
                if (jogo.sala_atual == "hall" || jogo.sala_atual == "hall de entrada")
                    && jogo.noite_vencida
                    && [
                        "usar isqueiro",
                        "acender isqueiro",
                        "queimar cortinas",
                        "queimar hall",
                        "incendiar",
                        "usar fios cortados",
                    ]
                    .contains(&comando.as_str())
                {
                    if jogo.inventario.iter().any(|i| i == "isqueiro") || jogo.god_mode {
                        let tem_fios = jogo.fios_cortados_inventario
                            || jogo.inventario.iter().any(|i| i == "fios cortados");
                        if tem_fios {
                            checar_conquistas_vitoria("verdadeiro", jogo, ui, audio);
                            rodar_final("verdadeiro", jogo, ui, audio, media);
                        } else {
                            checar_conquistas_vitoria("final_bom", jogo, ui, audio);
                            rodar_final("final_bom", jogo, ui, audio, media);
                        }
                        return;
                    } else {
                        ui.exibir(&format!(
                            "{}Você precisa de um 'isqueiro' para atear fogo!{}",
                            DOS_VERMELHO, RESET
                        ));
                        return;
                    }
                }
            }

            // 2. Mecânica de Correr ("correr <direção>")
            if let Some(dir_raw) = comando.strip_prefix("correr ") {
                if jogo.turnos_enjoado > 0 && !jogo.god_mode {
                    ui.exibir(&format!(
                        "{}Suas pernas tremem e sua cabeça gira... Você está exausto demais para correr agora!{}",
                        DOS_VERMELHO, RESET
                    ));
                    if jogo.estado_atual == "COMBATE_ANIMATRONICO" {
                        dar_tela_de_morte(jogo, ui, audio, media);
                    }
                    return;
                }

                let dir = match dir_raw.trim() {
                    "f" => "frente",
                    "a" | "t" | "atras" => "atrás",
                    "e" => "esquerda",
                    "d" => "direita",
                    outro => outro,
                };

                let sala_origem = jogo.sala_atual.clone();
                jogo.estado_atual = "JOGO".to_string();

                if cmd_ir(dir, jogo, ui, audio, media) {
                    jogo.posicao_perseguidor = "longe".to_string();
                    jogo.nivel_barulho = 100;
                    jogo.turnos_enjoado = 2;
                    ui.animar(
                        "Você corre desesperadamente batendo os pés no chão! Você despistou o perigo imediato, mas fez um barulho ensurdecedor (100% SOM)!",
                        DOS_AMARELO,
                    );

                    // 25% de chance de derrubar um item da mochila na sala anterior
                    let mut rng = rand::thread_rng();
                    let candidatos: Vec<String> = jogo
                        .inventario
                        .iter()
                        .filter(|i| *i != "lanterna")
                        .cloned()
                        .collect();
                    if !jogo.god_mode && !candidatos.is_empty() && rng.gen::<f64>() < 0.25 {
                        if let Some(item_caido) = candidatos.choose(&mut rng).cloned() {
                            jogo.inventario.retain(|i| i != &item_caido);
                            if let Some(s_ant) = jogo.mapa.get_mut(&sala_origem) {
                                s_ant.itens.push(item_caido.clone());
                            }
                            ui.animar(
                                &format!(
                                    "Na correria desesperada, '{}' caiu do seu bolso lá em '{}'!",
                                    item_caido, sala_origem
                                ),
                                DOS_VERMELHO,
                            );
                        }
                    }

                    if jogo.dificuldade == "DESAFIO_FANTASMA" && !jogo.god_mode {
                        ui.animar(
                            "[FALHA NO DESAFIO FANTASMA]: Seu barulho ultrapassou 50%! Ela te encontrou instantaneamente!",
                            DOS_VERMELHO,
                        );
                        dar_tela_de_morte(jogo, ui, audio, media);
                        return;
                    }

                    atualizar_eventos_de_tempo(jogo, ui, audio, media);
                    if jogo.estado_atual == "JOGO" {
                        imprimir_contexto_sala(jogo, ui, media);
                    }
                    let _ = jogo.salvar_local();
                } else if estado_antes == "COMBATE_ANIMATRONICO" {
                    jogo.estado_atual = "COMBATE_ANIMATRONICO".to_string();
                    ui.exibir(&format!(
                        "{}Não há saída para '{}' ! Corra para uma saída válida!{}",
                        DOS_VERMELHO, dir, RESET
                    ));
                }
                return;
            }

            // 3. Execução do comando padrão
            let acao_executada = processar_comando(&comando, jogo, ui, audio, media);

            let Some(acao) = acao_executada else {
                return;
            };

            if jogo.estado_atual == "FIM" {
                return;
            }

            // Se apenas olhou a sala atual
            if acao == "olhar" {
                jogo.alterar_barulho(-10);
                imprimir_contexto_sala(jogo, ui, media);
                return;
            }

            // Checa se entrou nas salas finais ("saida" ou "cama")
            if jogo.sala_atual == "saida" {
                checar_conquistas_vitoria("saida", jogo, ui, audio);
                rodar_final("saida", jogo, ui, audio, media);
                return;
            }
            if jogo.sala_atual == "cama" {
                checar_conquistas_vitoria("cama", jogo, ui, audio);
                rodar_final("cama", jogo, ui, audio, media);
                return;
            }

            // Atualiza nível de barulho por ação
            match acao.as_str() {
                "ir" => {
                    jogo.alterar_barulho(15);
                }
                "inventario" => {
                    jogo.alterar_barulho(-10);
                }
                _ => {
                    jogo.alterar_barulho(5);
                }
            }

            if jogo.dificuldade == "DESAFIO_FANTASMA"
                && jogo.nivel_barulho > 50
                && !jogo.god_mode
            {
                ui.animar(
                    "[FALHA NO DESAFIO FANTASMA]: Seu barulho ultrapassou 50%! Ela te encontrou instantaneamente!",
                    DOS_VERMELHO,
                );
                dar_tela_de_morte(jogo, ui, audio, media);
                return;
            }

            // Avança tempo (lanterna, enjoo, perseguidor) exceto se apenas abriu o inventário
            if acao != "inventario" {
                atualizar_eventos_de_tempo(jogo, ui, audio, media);
            }

            if jogo.estado_atual == "FIM" {
                return;
            }

            if acao == "ir" && jogo.estado_atual == "JOGO" {
                imprimir_contexto_sala(jogo, ui, media);
            }

            if jogo.dev_mode || jogo.god_mode {
                devmode::pos_turno(jogo, snapshot, acao == "largar", ui);
            }

            let _ = jogo.salvar_local();
        }

        // Qualquer estado de MINIGAME_*
        _ => {
            let _ = processar_minigame(&comando_norm, jogo, ui, audio, media);
            let _ = jogo.salvar_local();
        }
    }
}
