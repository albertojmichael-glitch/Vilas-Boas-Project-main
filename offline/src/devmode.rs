use crate::actions::{
    imprimir_contexto_sala, COR_AMARELO, COR_BRANCO, COR_CIANO, COR_VERDE, COR_VERMELHO,
};
use crate::data::{salas_seguras_patrulha, TURNOS_BATERIA, VIDA_NORMAL};
use crate::media::MediaManager;
use crate::parser::normalizar;
use crate::state::{calcular_caminho_bfs, GameState};
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
    term: &mut TerminalBuffer,
    media: &mut MediaManager,
) {
    let era_menu = jogo.estado_atual != "JOGO" && jogo.estado_atual != "COMBATE_ANIMATRONICO";
    jogo.dev_mode = true;
    jogo.god_mode = true;
    jogo.dev_usado = true;
    jogo.hp = 9999;
    jogo.turnos_luz = 9999;

    if era_menu {
        jogo.estado_atual = "JOGO".to_string();
        if jogo.mapa.contains_key("entrada") {
            jogo.sala_atual = "entrada".to_string();
        }
    }

    term.push(
        "[ MODO DESENVOLVEDOR ATIVADO - GOD MODE ON ]",
        COR_AMARELO,
    );
    imprimir_contexto_sala(jogo, term, media);
}

pub fn tirar_snapshot(jogo: &GameState) -> DevSnapshot {
    DevSnapshot {
        inventario: jogo.inventario.clone(),
    }
}

pub fn interceptar_perseguidor(jogo: &mut GameState, term: &mut TerminalBuffer) {
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
    term.push(
        &format!(
            "[DEV] O perseguidor te alcançou, mas o Modo Dev o afastou para '{}'.",
            jogo.ai_sala
        ),
        COR_AMARELO,
    );
}

pub fn exibir_radar(jogo: &GameState, term: &mut TerminalBuffer) {
    if let Some(caminho) = calcular_caminho_bfs(&jogo.mapa, &jogo.sala_atual, &jogo.ai_sala) {
        let dist = caminho.len().saturating_sub(1);
        let rota = caminho.join(" > ");
        term.push(
            &format!(
                "[RADAR] Perseguidor em '{}' ({} salas) | Rota: {}",
                jogo.ai_sala, dist, rota
            ),
            COR_CIANO,
        );
    } else {
        term.push(
            &format!("[RADAR] Perseguidor em '{}' (sem rota direta)", jogo.ai_sala),
            COR_CIANO,
        );
    }
}

pub fn pos_turno(
    jogo: &mut GameState,
    snapshot: DevSnapshot,
    foi_largar: bool,
    term: &mut TerminalBuffer,
) {
    if !jogo.dev_mode {
        return;
    }
    if jogo.estado_atual == "COMBATE_ANIMATRONICO" || jogo.sala_atual == "morte" {
        interceptar_perseguidor(jogo, term);
    }
    jogo.hp = 9999;
    jogo.turnos_luz = 9999;

    if !foi_largar {
        for item in snapshot.inventario {
            if !jogo.inventario.contains(&item) {
                jogo.inventario.push(item);
            }
        }
    }

    if jogo.dev_radar && jogo.estado_atual == "JOGO" {
        exibir_radar(jogo, term);
    }
}

pub fn processar_comando_dev(
    comando: &str,
    jogo: &mut GameState,
    term: &mut TerminalBuffer,
    media: &mut MediaManager,
) -> bool {
    if !jogo.dev_mode {
        return false;
    }

    if comando == "dev" || comando == "dev ajuda" {
        term.push(
            "=== COMANDOS DO MODO DEV ===\n\
            - tp <sala>          : Teleporta você para qualquer sala do JSON\n\
            - tp ia <sala>       : Teleporta o perseguidor\n\
            - radar              : Exibe rota BFS até o perseguidor\n\
            - dev radar on/off   : Ativa/desativa radar automático\n\
            - dev ia on/off      : Solta ou congela a IA\n\
            - gerar <item|tudo>  : Adiciona itens ao inventário\n\
            - dev ascii          : Lista e exibe todos os desenhos ASCII do data.py\n\
            - dev imagem <nome>  : Abre qualquer imagem das 4 pastas de assets\n\
            - dev mapa           : Lista todas as salas carregadas da pasta /data\n\
            - dev off            : Desativa o Modo Desenvolvedor",
            COR_AMARELO,
        );
        return true;
    }

    if comando == "dev ascii" {
        term.push(
            &format!("=== DESENHOS ASCII DO DATA.PY ({}) ===", jogo.ascii_arts.len()),
            COR_AMARELO,
        );
        for (nome, arte) in &jogo.ascii_arts {
            term.push(&format!("\n[ {} ]", nome), COR_CIANO);
            term.push(arte, COR_BRANCO);
        }
        return true;
    }

    if comando == "dev mapa" {
        let mut nomes: Vec<&String> = jogo.mapa.keys().collect();
        nomes.sort();
        term.push(
            &format!("=== SALAS CARREGADAS DO JSON ({}) ===", nomes.len()),
            COR_VERDE,
        );
        for nome in nomes {
            if let Some(sala) = jogo.mapa.get(nome) {
                let saidas: Vec<String> = sala
                    .saidas_validas(&jogo.mapa)
                    .into_iter()
                    .map(|(d, s)| format!("{}: {}", d, s))
                    .collect();
                term.push(&format!(" - {} -> [{}]", nome, saidas.join(", ")), COR_CIANO);
            }
        }
        return true;
    }

    if comando == "radar" {
        exibir_radar(jogo, term);
        return true;
    }
    if comando == "dev radar on" {
        jogo.dev_radar = true;
        term.push("[DEV] Radar automático ativado.", COR_VERDE);
        return true;
    }
    if comando == "dev radar off" {
        jogo.dev_radar = false;
        term.push("[DEV] Radar automático desativado.", COR_AMARELO);
        return true;
    }
    if comando == "dev ia on" {
        jogo.dev_ia = true;
        term.push("[DEV] IA do perseguidor liberada.", COR_VERDE);
        return true;
    }
    if comando == "dev ia off" {
        jogo.dev_ia = false;
        term.push("[DEV] IA do perseguidor congelada.", COR_AMARELO);
        return true;
    }
    if comando == "dev off" {
        jogo.dev_mode = false;
        jogo.god_mode = false;
        jogo.dev_ia = false;
        jogo.hp = VIDA_NORMAL;
        jogo.turnos_luz = TURNOS_BATERIA;
        term.push("[DEV] Modo Desenvolvedor desativado.", COR_AMARELO);
        return true;
    }

    if let Some(resto) = comando.strip_prefix("tp ia ") {
        let alvo = resto.trim();
        if let Some(nome_real) = jogo.mapa.keys().find(|k| normalizar(k) == alvo).cloned() {
            jogo.ai_sala = nome_real.clone();
            term.push(
                &format!("[DEV] Perseguidor teleportado para '{}'.", nome_real),
                COR_VERDE,
            );
        } else {
            term.push(
                &format!("[DEV] Sala '{}' não encontrada.", alvo),
                COR_VERMELHO,
            );
        }
        return true;
    }

    if let Some(resto) = comando.strip_prefix("tp ") {
        let alvo = resto.trim();
        if let Some(nome_real) = jogo.mapa.keys().find(|k| normalizar(k) == alvo).cloned() {
            jogo.sala_atual = nome_real;
            imprimir_contexto_sala(jogo, term, media);
        } else {
            term.push(
                &format!("[DEV] Sala '{}' não encontrada.", alvo),
                COR_VERMELHO,
            );
        }
        return true;
    }

    if let Some(resto) = comando.strip_prefix("gerar ") {
        let alvo = resto.trim();
        if alvo == "tudo" {
            let mut todos: Vec<String> = jogo.descricoes_itens.keys().cloned().collect();
            for sala in jogo.mapa.values() {
                for it in &sala.itens {
                    if !todos.contains(it) {
                        todos.push(it.clone());
                    }
                }
            }
            for item in todos {
                if !jogo.inventario.contains(&item) {
                    jogo.inventario.push(item);
                }
            }
            term.push("[DEV] Todos os itens do JSON adicionados!", COR_VERDE);
            return true;
        }
        jogo.inventario.push(alvo.to_string());
        term.push(
            &format!("[DEV] Item '{}' adicionado ao inventário.", alvo),
            COR_VERDE,
        );
        return true;
    }

    if let Some(img) = comando.strip_prefix("dev imagem ") {
        if !media.acionar_imagem(img.trim(), true, false) {
            term.push("Imagem não encontrada nas 4 pastas de assets.", COR_VERMELHO);
        }
        return true;
    }

    false
}