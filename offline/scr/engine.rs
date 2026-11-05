use crate::actions::{imprimir_contexto_sala, processar_comando_acao};
use crate::audio::AudioManager;
use crate::data::{salas_terminais, BARULHO_ERROS_CONSECUTIVOS, LIMITE_ERROS_DICA};
use crate::devmode;
use crate::media::MediaManager;
use crate::minigames::{imprimir_painel_cofre, processar_minigame};
use crate::parser::normalizar;
use crate::state::GameState;
use crate::ui::{
    imprimir_menu_configuracoes, imprimir_menu_principal, imprimir_tela_boot, TerminalUI,
    DOS_AMARELO, DOS_BRANCO, DOS_CIANO, DOS_VERDE, DOS_VERMELHO, RESET,
};
use rand::Rng;

fn registrar_conquista_offline(
    id: &str,
    jogo: &mut GameState,
    audio: &mut AudioManager,
    ui: &TerminalUI,
) {
    if jogo.dev_usado {
        return;
    }
    let nova_no_jogo = jogo.desbloquear_conquista(id);
    let mut nova_global = false;
    if !audio.config.conquistas_globais.contains(&id.to_string()) {
        audio.config.conquistas_globais.push(id.to_string());
        audio.config.salvar();
        nova_global = true;
    }
    if nova_no_jogo || nova_global {
        ui.exibir(&format!(
            "\n{}======================================================\n\
            [ CONQUISTA DESBLOQUEADA: {} ]\n\
            ======================================================{}",
            DOS_AMARELO,
            id.to_uppercase(),
            RESET
        ));
    }
}

pub fn verificar_final_de_jogo(
    jogo: &mut GameState,
    ui: &TerminalUI,
    audio: &mut AudioManager,
    media: &MediaManager,
) -> bool {
    if jogo.sala_atual == "hall de entrada" && jogo.noite_vencida && jogo.fios_cortados_inventario {
        registrar_conquista_offline("final_verdadeiro", jogo, audio, ui);
        jogo.estado_atual = "FIM".to_string();
        audio.parar_ambiente();
        let _ = media.exibir_imagem("final_verdadeiro", true, ui);
        ui.animar(
            "\n[ FINAL VERDADEIRO ] Você desativou a rede elétrica e escapou de Vilas Boas para sempre!",
            0.02,
            DOS_VERDE,
        );
        return true;
    }

    if jogo.sala_atual == "saida" || jogo.sala_atual == "final_bom" {
        if jogo.dificuldade_escolhida == "PESADELO" {
            registrar_conquista_offline("pesadelo_verdadeiro", jogo, audio, ui);
        } else {
            registrar_conquista_offline("normal_zerado", jogo, audio, ui);
        }
        jogo.estado_atual = "FIM".to_string();
        audio.parar_ambiente();
        ui.animar(
            "\n[ VITÓRIA ] As portas se abriram e você sobreviveu à noite em Vilas Boas!",
            0.02,
            DOS_VERDE,
        );
        return true;
    }

    if jogo.sala_atual == "morte" || jogo.hp <= 0 {
        registrar_conquista_offline("primeira_morte", jogo, audio, ui);
        jogo.estado_atual = "FIM".to_string();
        audio.parar_ambiente();
        ui.efeito_jumpscare(audio.config.jumpscare_visual);
        audio.som_grito();
        let _ = media.exibir_imagem("jumpscare", false, ui);
        ui.animar(
            "\n[ VOCÊ MORREU ] O som metálico foi a última coisa que você ouviu no escuro...",
            0.03,
            DOS_VERMELHO,
        );
        return true;
    }

    false
}

pub fn aplicar_efeitos_pos_turno(
    jogo: &mut GameState,
    comando_correu: bool,
    ui: &TerminalUI,
    audio: &mut AudioManager,
    media: &MediaManager,
) {
    if !jogo.god_mode {
        if jogo.turnos_luz > 0 {
            jogo.turnos_luz -= 1;
            if jogo.turnos_luz == 0 {
                ui.exibir(&format!(
                    "{}[!] A bateria da sua lanterna acabou! Use uma bateria reserva imediatamente!{}",
                    DOS_VERMELHO, RESET
                ));
            }
        } else {
            jogo.turnos_no_escuro += 1;
            if rand::thread_rng().gen_bool(0.4) {
                audio.som_randomico();
            }
            if jogo.turnos_no_escuro >= 3 {
                jogo.hp = 0;
                jogo.sala_atual = "morte".to_string();
                let _ = verificar_final_de_jogo(jogo, ui, audio, media);
                return;
            }
        }
    }

    if comando_correu {
        jogo.nivel_barulho = 100;
        ui.exibir(&format!(
            "{}Seus passos apressados ecoaram por toda a estrutura! (Ruído: 100%){}",
            DOS_AMARELO, RESET
        ));
    }

    let relatorio = jogo.avancar_ia_perseguidor();
    if relatorio.combate_iniciado {
        if jogo.dev_mode {
            devmode::interceptar_perseguidor(jogo, ui);
        } else {
            ui.efeito_jumpscare(audio.config.jumpscare_visual);
            audio.som_grito();
            let _ = media.exibir_imagem("animatronico", false, ui);
            ui.exibir(&format!(
                "{}[ PERIGO IMINENTE ] O animatrônico entrou na sala! Digite 'correr <direção>' para fugir!{}",
                DOS_VERMELHO, RESET
            ));
        }
    } else if relatorio.ativa && relatorio.distancia <= 2 {
        audio.som_metal();
        ui.exibir(&format!(
            "{}Você ouve passos pesados de metal rangendo bem perto daqui...{}",
            DOS_VERMELHO, RESET
        ));
    }
}

fn processar_menu_config(
    comando: &str,
    jogo: &mut GameState,
    ui: &mut TerminalUI,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) {
    if comando == "voltar" || comando == "sair" {
        audio.config.salvar();
        if jogo.sala_atual != "01" || jogo.turnos_luz != 6 {
            // Se já havia uma partida em andamento e abriu config pelo comando 'config'
            if jogo.estado_atual == "MENU_CONFIG_JOGO" {
                jogo.estado_atual = "JOGO".to_string();
                imprimir_contexto_sala(jogo, ui);
                return;
            }
        }
        jogo.estado_atual = "MENU".to_string();
        imprimir_menu_principal(ui, GameState::save_existe());
        return;
    }

    if comando == "testar" {
        ui.efeito_jumpscare(audio.config.jumpscare_visual);
        audio.som_grito();
        ui.exibir("Som e efeito de Jumpscare testados!");
        return;
    }
    if comando == "mudo" || comando == "5" {
        audio.config.mudo = !audio.config.mudo;
        audio.atualizar_volume_ambiente();
        audio.config.salvar();
        imprimir_menu_configuracoes(ui, &audio.config);
        return;
    }
    if comando == "flash" || comando == "6" {
        audio.config.jumpscare_visual = !audio.config.jumpscare_visual;
        audio.config.salvar();
        imprimir_menu_configuracoes(ui, &audio.config);
        return;
    }
    if comando == "animacao" || comando == "7" {
        audio.config.animacao_texto = !audio.config.animacao_texto;
        ui.animacao_ativa = audio.config.animacao_texto;
        audio.config.salvar();
        imprimir_menu_configuracoes(ui, &audio.config);
        return;
    }
    if comando == "janela" || comando == "8" {
        audio.config.abrir_janela_externa = !audio.config.abrir_janela_externa;
        media.abrir_janela_externa = audio.config.abrir_janela_externa;
        audio.config.salvar();
        imprimir_menu_configuracoes(ui, &audio.config);
        return;
    }

    let partes: Vec<&str> = comando.split_whitespace().collect();
    if partes.len() == 2 {
        if let Ok(valor) = partes[1].replace('%', "").parse::<f32>() {
            let prop = (valor / 100.0).clamp(0.0, 1.0);
            match partes[0] {
                "geral" | "1" => audio.config.volume_geral = prop,
                "jumpscare" | "grito" | "2" => audio.config.volume_jumpscare = prop,
                "ambiente" | "musica" | "3" => audio.config.volume_ambiente = prop,
                "efeitos" | "sfx" | "4" => audio.config.volume_efeitos = prop,
                _ => {
                    ui.exibir("Opção desconhecida. Use: geral, jumpscare, ambiente ou efeitos <0-100>.");
                    return;
                }
            }
            audio.atualizar_volume_ambiente();
            audio.config.salvar();
            imprimir_menu_configuracoes(ui, &audio.config);
            return;
        }
    }

    imprimir_menu_configuracoes(ui, &audio.config);
}

pub fn processar_fluxo_jogo(
    comando_bruto: &str,
    jogo: &mut GameState,
    ui: &mut TerminalUI,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) {
    let comando = normalizar(comando_bruto);
    if comando.is_empty() {
        return;
    }

    if devmode::eh_codigo_de_ativacao(&comando) {
        audio.iniciar_ambiente();
        devmode::ativar_modo_dev(jogo, ui);
        return;
    }

    if jogo.dev_mode && devmode::processar_comando_dev(&comando, jogo, ui, media) {
        return;
    }

    let snapshot = devmode::tirar_snapshot(jogo);
    let estado_antes = jogo.estado_atual.clone();

    match jogo.estado_atual.as_str() {
        "AGUARDANDO_DIR" => {
            if comando == "dir" {
                jogo.estado_atual = "MENU".to_string();
                imprimir_menu_principal(ui, GameState::save_existe());
            } else {
                ui.exibir(&format!(
                    "{}Bad command or file name{}\n{}Digite {}dir{} para acessar os diretórios:{}",
                    DOS_VERMELHO, RESET, DOS_VERDE, DOS_BRANCO, DOS_VERDE, RESET
                ));
            }
        }
        "MENU" => match comando.as_str() {
            "1" | "iniciar" | "novo" => {
                jogo.estado_atual = "MENU_DIFICULDADE".to_string();
                ui.exibir(&format!(
                    "\n{}Escolha a dificuldade:\n  [1] NORMAL (3 HP)\n  [2] PESADELO (1 HP)\n  [voltar] Retornar{}",
                    DOS_BRANCO, RESET
                ));
            }
            "2" | "continuar" | "carregar" => {
                if let Some(salvo) = GameState::carregar_local() {
                    *jogo = salvo;
                    audio.iniciar_ambiente();
                    ui.exibir(&format!("{}Save carregado com sucesso!{}", DOS_VERDE, RESET));
                    imprimir_contexto_sala(jogo, ui);
                } else {
                    ui.exibir(&format!(
                        "{}Nenhum arquivo de save encontrado.{}",
                        DOS_VERMELHO, RESET
                    ));
                }
            }
            "3" | "desafios" | "conquistas" => {
                jogo.estado_atual = "MENU_DESAFIOS".to_string();
                let mut todas = audio.config.conquistas_globais.clone();
                for c in &jogo.conquistas {
                    if !todas.contains(c) {
                        todas.push(c.clone());
                    }
                }
                ui.exibir(&format!(
                    "\n{}=== CONQUISTAS DESBLOQUEADAS ({}) ==={}",
                    DOS_AMARELO,
                    todas.len(),
                    RESET
                ));
                if todas.is_empty() {
                    ui.exibir("Nenhuma conquista registrada ainda.");
                } else {
                    for c in &todas {
                        ui.exibir(&format!("  * {}", c));
                    }
                }
                ui.exibir("Digite 'voltar' para retornar ao menu.");
            }
            "4" | "galeria" | "imagens" | "secret" => {
                jogo.estado_atual = "MENU_GALERIA".to_string();
                let normais = media.listar_imagens_disponiveis(false);
                let secretas = media.listar_imagens_disponiveis(true);
                ui.exibir(&format!(
                    "\n{}=== GALERIA DE ARQUIVOS DE IMAGEM ==={}",
                    DOS_CIANO, RESET
                ));
                ui.exibir(&format!("Imagens encontradas: {:?}", normais));
                ui.exibir(&format!("Secret Images encontradas: {:?}", secretas));
                ui.exibir("Digite o nome da imagem para visualizar ou 'voltar' para sair.");
            }
            "5" | "config" | "configuracoes" | "opcoes" => {
                jogo.estado_atual = "MENU_CONFIG".to_string();
                imprimir_menu_configuracoes(ui, &audio.config);
            }
            "6" | "sair" | "exit" => {
                std::process::exit(0);
            }
            _ => {
                imprimir_menu_principal(ui, GameState::save_existe());
            }
        },
        "MENU_CONFIG" | "MENU_CONFIG_JOGO" => {
            processar_menu_config(&comando, jogo, ui, audio, media);
        }
        "MENU_DIFICULDADE" => match comando.as_str() {
            "1" | "normal" => {
                jogo.reiniciar_partida("NORMAL");
                audio.iniciar_ambiente();
                imprimir_contexto_sala(jogo, ui);
            }
            "2" | "pesadelo" => {
                jogo.reiniciar_partida("PESADELO");
                audio.iniciar_ambiente();
                imprimir_contexto_sala(jogo, ui);
            }
            "voltar" => {
                jogo.estado_atual = "MENU".to_string();
                imprimir_menu_principal(ui, GameState::save_existe());
            }
            _ => ui.exibir("Digite 1 (Normal), 2 (Pesadelo) ou 'voltar'."),
        },
        "MENU_DESAFIOS" => {
            jogo.estado_atual = "MENU".to_string();
            imprimir_menu_principal(ui, GameState::save_existe());
        }
        "MENU_GALERIA" => {
            if comando == "voltar" || comando == "sair" {
                jogo.estado_atual = "MENU".to_string();
                imprimir_menu_principal(ui, GameState::save_existe());
            } else if !media.exibir_imagem(&comando, true, ui) {
                ui.exibir("Arquivo de imagem não encontrado. Digite outro nome ou 'voltar'.");
            }
        }
        "FIM" => {
            imprimir_tela_boot(ui);
            jogo.estado_atual = "AGUARDANDO_DIR".to_string();
        }
        "JOGO" | "COMBATE_ANIMATRONICO" => {
            if ["cls", "limpar", "clear", "clean"].contains(&comando.as_str()) {
                ui.limpar();
                imprimir_contexto_sala(jogo, ui);
                return;
            }
            if comando == "config" || comando == "configuracoes" || comando == "volume" {
                jogo.estado_atual = "MENU_CONFIG_JOGO".to_string();
                imprimir_menu_configuracoes(ui, &audio.config);
                return;
            }
            if comando == "salvar" {
                if jogo.salvar_local() {
                    ui.exibir(&format!(
                        "{}Progresso salvo em 'savegame.json'!{}",
                        DOS_VERDE, RESET
                    ));
                }
                return;
            }

            if comando == "abrir cofre" && jogo.sala_atual == "01" {
                jogo.estado_atual = "MINIGAME_COFRE".to_string();
                jogo.cofre_tentativas = 0;
                imprimir_painel_cofre(jogo, ui, true);
                return;
            }
            if comando == "cadeira" && jogo.sala_atual == "01" {
                if jogo
                    .inventario
                    .iter()
                    .any(|i| normalizar(i) == "cartao de seguranca")
                {
                    jogo.estado_atual = "MINIGAME_SEGURANCA".to_string();
                    ui.exibir(&format!(
                        "{}[ TERMINAL DA CADEIRA ATIVADO ] Digite 'vigiar' para avançar o turno até as 6AM.{}",
                        DOS_VERDE, RESET
                    ));
                } else {
                    ui.exibir(&format!(
                        "{}Acesso negado: exige o Cartão de Segurança.{}",
                        DOS_VERMELHO, RESET
                    ));
                }
                return;
            }
            if (comando == "jogar jon" || comando == "jogar fome de jon")
                && jogo.sala_atual == "sala de fliperamas"
            {
                jogo.estado_atual = "MINIGAME_JON".to_string();
                ui.exibir(&format!(
                    "{}[ FLIPERAMA: FOME DE JON ] Digite 'comer' para jogar ou 'sair'.{}",
                    DOS_AMARELO, RESET
                ));
                return;
            }
            if comando == "jogar consertos" && jogo.sala_atual == "sala de fliperamas" {
                if let Some(pos) = jogo
                    .inventario
                    .iter()
                    .position(|i| normalizar(i) == "moeda velha")
                {
                    jogo.inventario.remove(pos);
                    jogo.estado_atual = "MINIGAME_CONSERTOS_CABECA".to_string();
                    ui.exibir(&format!(
                        "{}[ FLIPERAMA: CONSERTOS ] Ficha consumida! Digite 'consertar' para iniciar.{}",
                        DOS_AMARELO, RESET
                    ));
                } else {
                    ui.exibir("Você precisa de uma ficha ('moeda velha') para jogar Consertos.");
                }
                return;
            }
            if comando == "jogar adivinha" && jogo.sala_atual == "sala de fliperamas" {
                if jogo
                    .inventario
                    .iter()
                    .any(|i| normalizar(i) == "partitura rasgada")
                {
                    jogo.estado_atual = "MINIGAME_JULGAMENTO_Q1".to_string();
                    ui.exibir(&format!(
                        "{}[ FLIPERAMA: O JULGAMENTO ] Digite as notas ou 'tocar' para responder.{}",
                        DOS_CIANO, RESET
                    ));
                } else {
                    ui.exibir("Falta a 'partitura rasgada' para iniciar este fliperama.");
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
                    devmode::interceptar_perseguidor(jogo, ui);
                } else {
                    jogo.hp = 0;
                    jogo.sala_atual = "morte".to_string();
                    let _ = verificar_final_de_jogo(jogo, ui, audio, media);
                }
                return;
            }

            let resultado_acao = processar_comando_acao(&comando_exec, jogo, ui, audio, media);

            let mut foi_largar = false;
            if let Some(tipo) = resultado_acao {
                jogo.erros_consecutivos = 0;
                foi_largar = tipo == "largar";
                if jogo.estado_atual == "JOGO" && tipo != "olhar" {
                    aplicar_efeitos_pos_turno(jogo, jogador_correu, ui, audio, media);
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
                    ui.animar(
                        "Seus tropeços no escuro estão chamando muita atenção...",
                        0.02,
                        DOS_AMARELO,
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
                registrar_conquista_offline("final_verdadeiro", jogo, audio, ui);
                audio.parar_ambiente();
                let _ = media.exibir_imagem("incendio", true, ui);
                ui.animar(
                    "\n[ FINAL VERDADEIRO ] O fogo purificou os segredos de Vilas Boas!",
                    0.02,
                    DOS_VERDE,
                );
                return;
            }

            if jogo.atingiu_meta_acumulador() {
                registrar_conquista_offline("acumulador", jogo, audio, ui);
            }

            if !verificar_final_de_jogo(jogo, ui, audio, media) && jogo.estado_atual == "JOGO" {
                if resultado_acao.is_some() && resultado_acao != Some("olhar") {
                    imprimir_contexto_sala(jogo, ui);
                }
            }

            if jogo.dev_mode && estado_antes != "FIM" {
                devmode::pos_turno(jogo, snapshot, foi_largar, ui);
            }
        }
        _ => {
            let antes = jogo.conquistas.len();
            let _ = processar_minigame(&comando, jogo, ui, audio);
            if jogo.conquistas.len() > antes {
                if let Some(ultima) = jogo.conquistas.last().cloned() {
                    registrar_conquista_offline(&ultima, jogo, audio, ui);
                }
            }
            if jogo.estado_atual == "JOGO" {
                imprimir_contexto_sala(jogo, ui);
            }
        }
    }
}