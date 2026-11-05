use crate::actions::imprimir_contexto_sala;
use crate::data::{descricoes_itens, salas_seguras_patrulha, TURNOS_BATERIA, VIDA_NORMAL};
use crate::media::MediaManager;
use crate::parser::normalizar;
use crate::state::{calcular_caminho_bfs, GameState};
use crate::ui::{
    TerminalUI, DOS_AMARELO, DOS_BRANCO, DOS_CIANO, DOS_VERDE, DOS_VERMELHO, RESET,
};

pub const CODIGO_PADRAO: &str = "sudo admin 1982";

#[derive(Clone)]
pub struct DevSnapshot {
    pub inventario: Vec<String>,
}

pub fn eh_codigo_de_ativacao(comando_normalizado: &str) -> bool {
    let codigo_env = std::env::var("DEV_MODE_CODE")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| CODIGO_PADRAO.to_string());
    comando_normalizado == normalizar(&codigo_env)
}

pub fn ativar_modo_dev(jogo: &mut GameState, ui: &TerminalUI) {
    let era_menu = jogo.estado_atual != "JOGO" && jogo.estado_atual != "COMBATE_ANIMATRONICO";
    jogo.dev_mode = true;
    jogo.god_mode = true;
    jogo.dev_usado = true;
    jogo.hp = 9999;
    jogo.turnos_luz = 9999;

    if era_menu {
        jogo.estado_atual = "JOGO".to_string();
        jogo.sala_atual = "entrada".to_string();
    }

    ui.exibir(&format!(
        "{}[ MODO DESENVOLVEDOR ATIVADO - GOD MODE ON ]{}",
        DOS_AMARELO, RESET
    ));
    imprimir_contexto_sala(jogo, ui);
}

pub fn tirar_snapshot(jogo: &GameState) -> DevSnapshot {
    DevSnapshot {
        inventario: jogo.inventario.clone(),
    }
}

pub fn interceptar_perseguidor(jogo: &mut GameState, ui: &TerminalUI) {
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
    jogo.estado_atual = "JOGO".to_string();
    ui.exibir(&format!(
        "{}[DEV] O perseguidor te alcançou, mas o modo dev te protegeu! Ele foi recuado para '{}'.{}",
        DOS_AMARELO, jogo.ai_sala, RESET
    ));
}

pub fn exibir_radar(jogo: &GameState, ui: &TerminalUI) {
    if let Some(caminho) = calcular_caminho_bfs(&jogo.mapa, &jogo.sala_atual, &jogo.ai_sala) {
        let dist = caminho.len().saturating_sub(1);
        let rota = caminho.join(" > ");
        let sufixo = if dist == 1 {
            "1 sala de distância".to_string()
        } else {
            format!("{} salas de distância", dist)
        };
        ui.exibir(&format!(
            "{}[RADAR] Perseguidor em '{}' ({}) | Rota: {}{}",
            DOS_CIANO, jogo.ai_sala, sufixo, rota, RESET
        ));
    } else {
        ui.exibir(&format!(
            "{}[RADAR] Perseguidor em '{}' (sem rota direta){}",
            DOS_CIANO, jogo.ai_sala, RESET
        ));
    }
}

pub fn pos_turno(
    jogo: &mut GameState,
    snapshot: DevSnapshot,
    foi_largar: bool,
    ui: &TerminalUI,
) {
    if !jogo.dev_mode {
        return;
    }
    if jogo.estado_atual == "COMBATE_ANIMATRONICO" || jogo.sala_atual == "morte" {
        interceptar_perseguidor(jogo, ui);
    }
    jogo.hp = 9999;
    jogo.turnos_luz = 9999;

    // Restaura itens consumidos caso não tenham sido largados de propósito
    if !foi_largar {
        for item in snapshot.inventario {
            if !jogo.inventario.contains(&item) {
                jogo.inventario.push(item);
            }
        }
    }

    if jogo.dev_radar && jogo.estado_atual == "JOGO" {
        exibir_radar(jogo, ui);
    }
}

pub fn processar_comando_dev(
    comando: &str,
    jogo: &mut GameState,
    ui: &TerminalUI,
    media: &MediaManager,
) -> bool {
    if !jogo.dev_mode {
        return false;
    }

    if comando == "dev" || comando == "dev ajuda" {
        ui.exibir(&format!(
            "{}=== COMANDOS DO MODO DEV ===\n\
            - tp <sala>          : Teleporta você para a sala\n\
            - tp ia <sala>       : Teleporta o perseguidor\n\
            - radar              : Exibe a posição e rota do perseguidor\n\
            - dev radar on/off   : Liga/desliga o radar automático pós-turno\n\
            - dev ia on/off      : Solta ou congela a IA do perseguidor\n\
            - gerar <item|tudo>  : Adiciona item (ou todos) na mochila\n\
            - dev imagem <nome>  : Renderiza qualquer imagem/secret image\n\
            - dev status         : Mostra variáveis internas da engine\n\
            - dev mapa           : Lista todas as salas e conexões\n\
            - dev morrer         : Força tela de fim de jogo\n\
            - dev off            : Desativa o modo desenvolvedor{}",
            DOS_AMARELO, RESET
        ));
        return true;
    }

    if comando == "dev status" {
        ui.exibir(&format!(
            "{}=== STATUS DEV ===\nSala: {} | IA Sala: {} ({}) | Barulho: {} | IA Solta: {}{}",
            DOS_CIANO,
            jogo.sala_atual,
            jogo.ai_sala,
            jogo.ai_estado,
            jogo.nivel_barulho,
            jogo.dev_ia,
            RESET
        ));
        return true;
    }

    if comando == "dev mapa" {
        ui.exibir(&format!("{}=== MAPA COMPLETO ==={}", DOS_VERDE, RESET));
        let mut nomes: Vec<&String> = jogo.mapa.keys().collect();
        nomes.sort();
        for nome in nomes {
            if let Some(sala) = jogo.mapa.get(nome) {
                let saidas: Vec<String> = sala
                    .saidas_validas(&jogo.mapa)
                    .into_iter()
                    .map(|(d, s)| format!("{}: {}", d, s))
                    .collect();
                ui.exibir(&format!(" - {} -> [{}]", nome, saidas.join(", ")));
            }
        }
        return true;
    }

    if comando == "radar" {
        exibir_radar(jogo, ui);
        return true;
    }

    if comando == "dev radar on" {
        jogo.dev_radar = true;
        ui.exibir("[DEV] Radar automático ativado.");
        return true;
    }
    if comando == "dev radar off" {
        jogo.dev_radar = false;
        ui.exibir("[DEV] Radar automático desativado.");
        return true;
    }

    if comando == "dev ia on" {
        jogo.dev_ia = true;
        ui.exibir("[DEV] IA do perseguidor liberada.");
        return true;
    }
    if comando == "dev ia off" {
        jogo.dev_ia = false;
        ui.exibir("[DEV] IA do perseguidor congelada.");
        return true;
    }

    if comando == "dev morrer" {
        jogo.estado_atual = "FIM".to_string();
        jogo.sala_atual = "morte".to_string();
        ui.exibir(&format!("{}[DEV] Morte forçada acionada.{}", DOS_VERMELHO, RESET));
        return true;
    }

    if comando == "dev off" {
        jogo.dev_mode = false;
        jogo.god_mode = false;
        jogo.dev_ia = false;
        jogo.hp = VIDA_NORMAL;
        jogo.turnos_luz = TURNOS_BATERIA;
        ui.exibir(&format!(
            "{}[DEV] Modo desenvolvedor desativado. Valores restaurados.{}",
            DOS_AMARELO, RESET
        ));
        return true;
    }

    if let Some(resto) = comando.strip_prefix("tp ia ") {
        let alvo = resto.trim();
        if let Some(nome_real) = jogo.mapa.keys().find(|k| normalizar(k) == alvo).cloned() {
            jogo.ai_sala = nome_real.clone();
            ui.exibir(&format!("[DEV] Perseguidor teleportado para '{}'.", nome_real));
        } else {
            ui.exibir(&format!("{}[DEV] A sala '{}' não existe.{}", DOS_VERMELHO, alvo, RESET));
        }
        return true;
    }

    if let Some(resto) = comando.strip_prefix("tp ") {
        let alvo = resto.trim();
        if let Some(nome_real) = jogo.mapa.keys().find(|k| normalizar(k) == alvo).cloned() {
            jogo.sala_atual = nome_real;
            imprimir_contexto_sala(jogo, ui);
        } else {
            ui.exibir(&format!("{}[DEV] A sala '{}' não existe.{}", DOS_VERMELHO, alvo, RESET));
        }
        return true;
    }

    if let Some(resto) = comando.strip_prefix("gerar ") {
        let alvo = resto.trim();
        let catalogo = descricoes_itens();
        if alvo == "tudo" {
            for item in catalogo.keys() {
                if !jogo.inventario.contains(item) {
                    jogo.inventario.push(item.clone());
                }
            }
            ui.exibir(&format!("{}[DEV] Todos os itens gerados na mochila!{}", DOS_VERDE, RESET));
            return true;
        }
        let nome_item = catalogo
            .keys()
            .find(|k| normalizar(k) == alvo)
            .cloned()
            .unwrap_or_else(|| alvo.to_string());
        if !jogo.inventario.contains(&nome_item) {
            jogo.inventario.push(nome_item.clone());
        }
        ui.exibir(&format!("{}[DEV] Item '{}' adicionado ao inventário.{}", DOS_VERDE, nome_item, RESET));
        return true;
    }

    if let Some(img) = comando.strip_prefix("dev imagem ") {
        if !media.exibir_imagem(img.trim(), true, ui) {
            ui.exibir("Imagem não encontrada nas pastas de assets.");
        }
        return true;
    }

    false
}