use crate::audio::AudioManager;
use crate::data::{salas_seguras_patrulha, TURNOS_BATERIA, VIDA_NORMAL};
use crate::media::MediaManager;
use crate::parser::normalizar;
use crate::state::{calcular_caminho_bfs, GameState};
use crate::ui::{DOS_AMARELO, DOS_BRANCO, DOS_CIANO, DOS_VERDE, DOS_VERMELHO, RESET};
use crate::utils::encontrar_melhor_match;
use crate::views;
use crate::TerminalBuffer;

pub const CODIGO_PADRAO: &str = "sudo admin 1982";

#[derive(Clone)]
pub struct DevSnapshot {
    pub inventario: Vec<String>,
}

pub fn eh_codigo_de_ativacao(comando_normalizado: &str) -> bool {
    comando_normalizado == normalizar(CODIGO_PADRAO)
}

pub fn ativar_modo_dev(
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    media: &mut MediaManager,
) {
    let era_menu = !["JOGO", "COMBATE_ANIMATRONICO"].contains(&jogo.estado_atual.as_str())
        && !jogo.estado_atual.starts_with("MINIGAME_");

    jogo.dev_mode = true;
    jogo.god_mode = true;
    jogo.dev_usado = true;
    jogo.dev_ia_congelada = true;
    jogo.dev_ia = false;
    jogo.hp = 9999;
    jogo.turnos_luz = 9999;

    if era_menu {
        jogo.estado_atual = "JOGO".to_string();
        if jogo.mapa.contains_key("entrada") {
            jogo.sala_atual = "entrada".to_string();
        }
    }

    ui.exibir(&format!(
        "\n{}=================================================={}",
        DOS_VERDE, RESET
    ));
    ui.exibir(&format!(
        "{}[ ROOT ACCESS GRANTED - MODO DESENVOLVEDOR ATIVO ]{}",
        DOS_VERDE, RESET
    ));
    ui.exibir(&format!(
        "{}• Vida/Luz infinitas | IA congelada | Digite 'dev' para comandos.{}",
        DOS_AMARELO, RESET
    ));
    ui.exibir(&format!(
        "{}=================================================={}",
        DOS_VERDE, RESET
    ));

    if era_menu {
        views::imprimir_contexto_sala(jogo, ui, media);
    }
}

pub fn tirar_snapshot(jogo: &GameState) -> DevSnapshot {
    DevSnapshot {
        inventario: jogo.inventario.clone(),
    }
}

pub fn interceptar_perseguidor(jogo: &mut GameState, ui: &mut TerminalBuffer) {
    let seguras = salas_seguras_patrulha();
    let mut mais_longe = jogo.ai_sala.clone();
    let mut maior_dist = 0;

    for nome_sala in jogo.mapa.keys() {
        if seguras.contains(&nome_sala.as_str()) {
            continue;
        }
        if let Some(caminho) = calcular_caminho_bfs(&jogo.mapa, &jogo.sala_atual, nome_sala) {
            if caminho.len() > maior_dist {
                maior_dist = caminho.len();
                mais_longe = nome_sala.clone();
            }
        }
    }

    jogo.ai_sala = mais_longe;
    jogo.posicao_perseguidor = "longe".to_string();
    jogo.estado_atual = "JOGO".to_string();
    ui.exibir(&format!(
        "{}[DEV] O perseguidor te alcançou, mas o Modo Dev o afastou para '{}'.{}",
        DOS_AMARELO, jogo.ai_sala, RESET
    ));
}

pub fn exibir_radar(jogo: &GameState, ui: &mut TerminalBuffer) {
    if let Some(caminho) = calcular_caminho_bfs(&jogo.mapa, &jogo.sala_atual, &jogo.ai_sala) {
        let dist = caminho.len().saturating_sub(1);
        let rota = caminho.join(" -> ");
        ui.exibir(&format!(
            "{}[RADAR DEV] IA em '{}' (Distância: {} salas | Estado: {}) | Rota: {}{}",
            DOS_CIANO, jogo.ai_sala, dist, jogo.ai_estado, rota, RESET
        ));
    } else {
        ui.exibir(&format!(
            "{}[RADAR DEV] IA em '{}' (sem rota direta a partir de '{}'){}",
            DOS_CIANO, jogo.ai_sala, jogo.sala_atual, RESET
        ));
    }
}

pub fn pos_turno(
    jogo: &mut GameState,
    snapshot: DevSnapshot,
    foi_largar: bool,
    ui: &mut TerminalBuffer,
) {
    if !jogo.dev_mode && !jogo.god_mode {
        return;
    }
    if jogo.estado_atual == "COMBATE_ANIMATRONICO" || jogo.sala_atual == "morte" {
        interceptar_perseguidor(jogo, ui);
    }
    jogo.hp = 9999;
    jogo.turnos_luz = 9999;

    if jogo.dev_mode && !foi_largar {
        for item in snapshot.inventario {
            if !jogo.inventario.contains(&item) {
                if item == "tesoura" && jogo.inventario.contains(&"tesoura quebrada".to_string())
                {
                    continue;
                }
                jogo.inventario.push(item);
            }
        }
    }

    if (jogo.dev_radar || jogo.dev_radar_ativo) && jogo.estado_atual == "JOGO" {
        exibir_radar(jogo, ui);
    }
}

pub fn processar_comando_dev(
    comando: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) -> bool {
    if !jogo.dev_mode && !jogo.god_mode {
        return false;
    }

    if comando == "dev" || comando == "dev ajuda" {
        ui.exibir(&format!(
            "{}=== COMANDOS DO MODO DESENVOLVEDOR ===\n\
            • tp <sala>                        : Teleporta você para a sala informada\n\
            • tp ia                            : Teleporta você até a sala da IA\n\
            • tp                               : Lista todas as salas disponíveis\n\
            • gerar <item> / gerar tudo        : Gera um item ou todos os itens\n\
            • radar                            : Mostra distância e rota BFS da IA\n\
            • dev radar on / off               : Liga/desliga radar automático a cada turno\n\
            • dev ia on / off / <sala>         : Descongela, congela ou move a IA\n\
            • dev mapa                         : Mostra todas as salas (@ = você, X = IA)\n\
            • dev status                       : Exibe variáveis internas da partida\n\
            • dev dificuldade <normal|pesadelo|dev> : Altera a dificuldade atual\n\
            • dev morrer                       : Testa a tela de Game Over\n\
            • dev ascii                        : Lista todas as artes ASCII\n\
            • dev imagem <nome>                : Abre uma imagem das pastas de assets\n\
            • dev off                          : Desativa o Modo Desenvolvedor{}",
            DOS_AMARELO, RESET
        ));
        return true;
    }

    if comando == "tp" {
        let mut nomes: Vec<String> = jogo.mapa.keys().cloned().collect();
        nomes.sort();
        ui.exibir(&format!(
            "{}[DEV] Salas disponíveis ({}): {}{}",
            DOS_CIANO,
            nomes.len(),
            nomes.join(", "),
            RESET
        ));
        return true;
    }

    if comando == "tp ia" {
        jogo.sala_atual = jogo.ai_sala.clone();
        ui.exibir(&format!(
            "{}[DEV] Você se teleportou para a sala da IA: '{}'.{}",
            DOS_VERDE, jogo.sala_atual, RESET
        ));
        views::imprimir_contexto_sala(jogo, ui, media);
        return true;
    }

    if let Some(resto) = comando
        .strip_prefix("tp ia ")
        .or_else(|| comando.strip_prefix("dev ia "))
    {
        let alvo = resto.trim();
        if alvo == "on" {
            jogo.dev_ia = true;
            jogo.dev_ia_congelada = false;
            ui.exibir(&format!(
                "{}[DEV] IA do perseguidor DESCONGELADA (ativa).{}",
                DOS_VERDE, RESET
            ));
            return true;
        } else if alvo == "off" {
            jogo.dev_ia = false;
            jogo.dev_ia_congelada = true;
            ui.exibir(&format!(
                "{}[DEV] IA do perseguidor CONGELADA.{}",
                DOS_AMARELO, RESET
            ));
            return true;
        }

        let salas: Vec<String> = jogo.mapa.keys().cloned().collect();
        if let Some(nome_real) = salas
            .iter()
            .find(|k| normalizar(k) == alvo)
            .cloned()
            .or_else(|| encontrar_melhor_match(alvo, &salas, 0.65))
        {
            jogo.ai_sala = nome_real.clone();
            ui.exibir(&format!(
                "{}[DEV] Perseguidor teleportado para '{}'.{}",
                DOS_VERDE, nome_real, RESET
            ));
        } else {
            ui.exibir(&format!(
                "{}[DEV] Sala '{}' não encontrada.{}",
                DOS_VERMELHO, alvo, RESET
            ));
        }
        return true;
    }

    if let Some(resto) = comando.strip_prefix("tp ") {
        let alvo = resto.trim();
        let salas: Vec<String> = jogo.mapa.keys().cloned().collect();
        if let Some(nome_real) = salas
            .iter()
            .find(|k| normalizar(k) == alvo)
            .cloned()
            .or_else(|| encontrar_melhor_match(alvo, &salas, 0.65))
        {
            jogo.sala_atual = nome_real;
            jogo.estado_atual = "JOGO".to_string();
            views::imprimir_contexto_sala(jogo, ui, media);
        } else {
            ui.exibir(&format!(
                "{}[DEV] Sala '{}' não encontrada.{}",
                DOS_VERMELHO, alvo, RESET
            ));
        }
        return true;
    }

    if let Some(resto) = comando.strip_prefix("gerar ") {
        let alvo = resto.trim();
        if alvo == "caroline" || alvo == "alma" {
            ui.animar(
                "[ERRO FATAL DE SISTEMA]: TENTATIVA DE MATERIALIZAR ENTIDADE NÃO AUTORIZADA...",
                DOS_VERMELHO,
            );
            views::dar_tela_kernel_panic(jogo, ui);
            return true;
        }
        if alvo == "tudo" {
            let mut todos: Vec<String> = jogo.descricoes_itens.keys().cloned().collect();
            for sala in jogo.mapa.values() {
                for it in &sala.itens {
                    if !todos.contains(it) {
                        todos.push(it.clone());
                    }
                }
            }
            todos.retain(|i| i != "fios cortados");
            for item in todos {
                if !jogo.inventario.contains(&item) {
                    jogo.inventario.push(item);
                }
            }
            ui.exibir(&format!(
                "{}[DEV] Todos os itens (exceto 'fios cortados') foram adicionados à mochila!{}",
                DOS_VERDE, RESET
            ));
            return true;
        }

        let opcoes_itens: Vec<String> = jogo.descricoes_itens.keys().cloned().collect();
        let item_real = encontrar_melhor_match(alvo, &opcoes_itens, 0.70)
            .unwrap_or_else(|| alvo.to_string());
        if !jogo.inventario.contains(&item_real) {
            jogo.inventario.push(item_real.clone());
        }
        if item_real == "fios cortados" {
            jogo.fios_cortados_inventario = true;
        }
        ui.exibir(&format!(
            "{}[DEV] Item '{}' adicionado ao inventário.{}",
            DOS_VERDE, item_real, RESET
        ));
        return true;
    }

    if comando == "radar" {
        exibir_radar(jogo, ui);
        return true;
    }
    if comando == "dev radar on" {
        jogo.dev_radar = true;
        jogo.dev_radar_ativo = true;
        ui.exibir(&format!(
            "{}[DEV] Radar automático ativado.{}",
            DOS_VERDE, RESET
        ));
        return true;
    }
    if comando == "dev radar off" {
        jogo.dev_radar = false;
        jogo.dev_radar_ativo = false;
        ui.exibir(&format!(
            "{}[DEV] Radar automático desativado.{}",
            DOS_AMARELO, RESET
        ));
        return true;
    }

    if comando == "dev mapa" {
        let mut nomes: Vec<&String> = jogo.mapa.keys().collect();
        nomes.sort();
        ui.exibir(&format!(
            "{}=== MAPA DO SISTEMA ({} salas | @ = Você | X = IA) ==={}",
            DOS_VERDE,
            nomes.len(),
            RESET
        ));
        for nome in nomes {
            if let Some(sala) = jogo.mapa.get(nome) {
                let mut marcador = String::new();
                if *nome == jogo.sala_atual {
                    marcador.push_str(&format!(" {}[@]{}", DOS_VERDE, RESET));
                }
                if *nome == jogo.ai_sala {
                    marcador.push_str(&format!(" {}[X]{}", DOS_VERMELHO, RESET));
                }
                let saidas: Vec<String> = sala
                    .saidas_validas(&jogo.mapa)
                    .into_iter()
                    .map(|(d, s)| format!("{}: {}", d, s))
                    .collect();
                ui.exibir(&format!(
                    "{}• {}{} -> [{}]{}",
                    DOS_CIANO,
                    nome,
                    marcador,
                    saidas.join(", "),
                    RESET
                ));
            }
        }
        return true;
    }

    if comando == "dev status" {
        ui.exibir(&format!(
            "{}=== STATUS INTERNO (DEV) ===\n\
            • Sala Atual: {} | Estado: {}\n\
            • Dificuldade: {} | HP: {} | Luz: {} | Barulho: {}%\n\
            • IA Sala: {} | IA Estado: {} | Posição Perseguidor: {} | IA Congelada: {}\n\
            • Noite Vencida: {} | Amanheceu: {} | Fios Cortados: {}\n\
            • Inventário ({}/{}): {:?}{}",
            DOS_CIANO,
            jogo.sala_atual,
            jogo.estado_atual,
            jogo.dificuldade,
            jogo.hp,
            jogo.turnos_luz,
            jogo.nivel_barulho,
            jogo.ai_sala,
            jogo.ai_estado,
            jogo.posicao_perseguidor,
            jogo.dev_ia_congelada,
            jogo.noite_vencida,
            jogo.amanheceu,
            jogo.fios_cortados_inventario,
            jogo.inventario.len(),
            jogo.obter_limite_inventario(),
            jogo.inventario,
            RESET
        ));
        return true;
    }

    if let Some(dif) = comando.strip_prefix("dev dificuldade ") {
        match dif.trim() {
            "normal" => {
                jogo.dificuldade = "NORMAL".to_string();
                ui.exibir(&format!(
                    "{}[DEV] Dificuldade alterada para NORMAL.{}",
                    DOS_VERDE, RESET
                ));
            }
            "pesadelo" => {
                jogo.dificuldade = "PESADELO".to_string();
                ui.exibir(&format!(
                    "{}[DEV] Dificuldade alterada para PESADELO.{}",
                    DOS_VERMELHO, RESET
                ));
            }
            "dev" | "deus" => {
                jogo.dificuldade = "DEV".to_string();
                jogo.god_mode = true;
                ui.exibir(&format!(
                    "{}[DEV] Dificuldade alterada para DEV (God Mode).{}",
                    DOS_AMARELO, RESET
                ));
            }
            _ => {
                ui.exibir(&format!(
                    "{}Use: dev dificuldade <normal|pesadelo|dev>{}",
                    DOS_AMARELO, RESET
                ));
            }
        }
        return true;
    }

    if comando == "dev morrer" {
        views::dar_tela_de_morte(jogo, ui, audio, media);
        return true;
    }

    if comando == "dev ascii" {
        ui.exibir(&format!(
            "{}=== DESENHOS ASCII EMBUTIDOS ({}) ==={}",
            DOS_AMARELO,
            jogo.ascii_arts.len(),
            RESET
        ));
        for (nome, arte) in &jogo.ascii_arts {
            ui.exibir(&format!("\n{}[ {} ]{}", DOS_CIANO, nome, RESET));
            ui.exibir(&format!("{}{}{}", DOS_BRANCO, arte, RESET));
        }
        return true;
    }

    if let Some(img) = comando.strip_prefix("dev imagem ") {
        if !media.acionar_imagem(img.trim(), true, false) {
            ui.exibir(&format!(
                "{}Imagem '{}' não encontrada nos assets.{}",
                DOS_VERMELHO,
                img.trim(),
                RESET
            ));
        }
        return true;
    }

    if comando == "dev off" {
        jogo.dev_mode = false;
        jogo.god_mode = false;
        jogo.dev_ia = true;
        jogo.dev_ia_congelada = false;
        jogo.hp = VIDA_NORMAL;
        jogo.turnos_luz = TURNOS_BATERIA;
        ui.exibir(&format!(
            "{}[DEV] Modo Desenvolvedor desativado.{}",
            DOS_AMARELO, RESET
        ));
        return true;
    }

    false
}
