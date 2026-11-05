use crate::audio::AudioManager;
use crate::data::{descricoes_itens, TURNOS_BATERIA};
use crate::media::MediaManager;
use crate::parser::{expandir_atalho_direcao, normalizar};
use crate::state::GameState;
use crate::ui::{
    TerminalUI, DOS_AMARELO, DOS_BRANCO, DOS_CIANO, DOS_CINZA, DOS_VERDE, DOS_VERMELHO, RESET,
};

pub fn imprimir_contexto_sala(jogo: &GameState, ui: &TerminalUI) {
    ui.exibir(&format!(
        "\n{}============================================================{}",
        DOS_CINZA, RESET
    ));
    ui.exibir(&format!(
        "{}LOCAL: [ {} ] | HP: {} | LUZ: {} turnos | RUÍDO: {}%{}",
        DOS_VERDE,
        jogo.sala_atual.to_uppercase(),
        jogo.hp,
        jogo.turnos_luz,
        jogo.nivel_barulho,
        RESET
    ));
    ui.exibir(&format!(
        "{}============================================================{}",
        DOS_CINZA, RESET
    ));

    if jogo.turnos_luz <= 0 && !jogo.god_mode {
        ui.exibir(&format!(
            "{}Está um breu total! Sua lanterna está sem bateria.{}",
            DOS_VERMELHO, RESET
        ));
        return;
    }

    if let Some(sala) = jogo.mapa.get(&jogo.sala_atual) {
        ui.exibir(&format!("{}{}{}", DOS_BRANCO, sala.descricao, RESET));

        if !sala.itens.is_empty() {
            ui.exibir(&format!(
                "\n{}Itens visíveis no local:{} {:?}",
                DOS_AMARELO, RESET, sala.itens
            ));
        }

        let saidas = sala.saidas_validas(&jogo.mapa);
        if !saidas.is_empty() {
            let formatadas: Vec<String> = saidas
                .iter()
                .map(|(dir, dest)| format!("{} ({})", dir, dest))
                .collect();
            ui.exibir(&format!(
                "{}Saídas:{} {}",
                DOS_CIANO,
                RESET,
                formatadas.join(" | ")
            ));
        }
    }
}

pub fn processar_comando_acao(
    comando_bruto: &str,
    jogo: &mut GameState,
    ui: &TerminalUI,
    audio: &AudioManager,
    media: &MediaManager,
) -> Option<&'static str> {
    let cmd_norm = normalizar(comando_bruto);
    let cmd = expandir_atalho_direcao(&cmd_norm);

    if cmd == "olhar" || cmd == "examinar sala" {
        if jogo.sala_atual == "sala de energia" && !jogo.noite_vencida {
            jogo.estado_atual = "MINIGAME_MINOTAURO".to_string();
            jogo.minotauro_pos = (0, 0);
            ui.exibir(&format!(
                "{}[ TERMINAL DE ENERGIA - LABIRINTO DO MINOTAURO ]{}",
                DOS_AMARELO, RESET
            ));
            ui.exibir("Use w/a/s/d para guiar a carga até a saída (2,2) ou digite 'sair'.");
            return Some("olhar");
        }
        imprimir_contexto_sala(jogo, ui);
        return Some("olhar");
    }

    if cmd == "inventario" || cmd == "i" || cmd == "mochila" {
        ui.exibir(&format!(
            "{}Mochila ({}/{}):{} {:?}",
            DOS_AMARELO,
            jogo.inventario.len(),
            jogo.obter_limite_inventario(),
            RESET,
            jogo.inventario
        ));
        return None;
    }

    if let Some(direcao_raw) = cmd.strip_prefix("ir ") {
        let direcao = match direcao_raw.trim() {
            "atras" | "t" => "atrás",
            "f" => "frente",
            "e" => "esquerda",
            "d" => "direita",
            outro => outro,
        };

        let destino = jogo
            .mapa
            .get(&jogo.sala_atual)
            .and_then(|s| s.obter_saida(direcao));

        if let Some(nova_sala) = destino {
            if nova_sala == "saida" && !jogo.noite_vencida && !jogo.inventario.contains(&"chave mestra".to_string()) && !jogo.dev_mode {
                ui.exibir(&format!(
                    "{}A porta blindada do Hall está trancada! Vença a noite na Sala 01/Energia ou use a Chave Mestra.{}",
                    DOS_VERMELHO, RESET
                ));
                return None;
            }
            jogo.sala_atual = nova_sala;
            jogo.alterar_barulho(10);
            audio.som_porta();
            return Some("turno");
        } else {
            ui.exibir(&format!(
                "{}Não há passagem para '{}' nesta sala.{}",
                DOS_VERMELHO, direcao, RESET
            ));
            return None;
        }
    }

    if let Some(item_alvo) = cmd.strip_prefix("pegar ") {
        let item_busca = item_alvo.trim();
        let item_encontrado = jogo
            .mapa
            .get(&jogo.sala_atual)
            .and_then(|s| {
                s.itens
                    .iter()
                    .find(|i| normalizar(i) == item_busca)
                    .cloned()
            });

        if let Some(item_real) = item_encontrado {
            if normalizar(&item_real) == "bolsa" {
                if let Some(sala) = jogo.mapa.get_mut(&jogo.sala_atual) {
                    sala.itens.retain(|i| i != &item_real);
                }
                jogo.bolsas_coletadas += 1;
                ui.exibir(&format!(
                    "{}Você coletou uma Bolsa! Capacidade da mochila expandida para {}.{}",
                    DOS_VERDE,
                    jogo.obter_limite_inventario(),
                    RESET
                ));
                return Some("turno");
            }

            if jogo.inventario.len() >= jogo.obter_limite_inventario() {
                ui.exibir(&format!(
                    "{}Sua mochila está cheia! Limite atual: {}.{}",
                    DOS_VERMELHO,
                    jogo.obter_limite_inventario(),
                    RESET
                ));
                return None;
            }

            if let Some(sala) = jogo.mapa.get_mut(&jogo.sala_atual) {
                sala.itens.retain(|i| i != &item_real);
            }
            jogo.inventario.push(item_real.clone());
            ui.exibir(&format!(
                "{}Você pegou: {}.{}",
                DOS_VERDE, item_real, RESET
            ));
            return Some("turno");
        } else {
            ui.exibir(&format!(
                "{}Esse item não está aqui.{}",
                DOS_VERMELHO, RESET
            ));
            return None;
        }
    }

    if let Some(item_alvo) = cmd.strip_prefix("largar ") {
        let item_busca = item_alvo.trim();
        if let Some(pos) = jogo
            .inventario
            .iter()
            .position(|i| normalizar(i) == item_busca)
        {
            let removido = jogo.inventario.remove(pos);
            if let Some(sala) = jogo.mapa.get_mut(&jogo.sala_atual) {
                sala.itens.push(removido.clone());
            }
            ui.exibir(&format!(
                "{}Você largou {} no chão.{}",
                DOS_AMARELO, removido, RESET
            ));
            return Some("largar");
        } else {
            ui.exibir(&format!(
                "{}Você não possui esse item na mochila.{}",
                DOS_VERMELHO, RESET
            ));
            return None;
        }
    }

    if let Some(item_alvo) = cmd.strip_prefix("usar ") {
        let item_busca = item_alvo.trim();
        let pos_inv = jogo
            .inventario
            .iter()
            .position(|i| normalizar(i) == item_busca);

        let Some(idx) = pos_inv else {
            ui.exibir(&format!(
                "{}Você não tem '{}' no inventário.{}",
                DOS_VERMELHO, item_busca, RESET
            ));
            return None;
        };

        let nome_item = jogo.inventario[idx].clone();
        let norm_item = normalizar(&nome_item);

        if norm_item == "bateria nova" || norm_item == "bateria" {
            jogo.inventario.remove(idx);
            jogo.turnos_luz += TURNOS_BATERIA;
            jogo.turnos_no_escuro = 0;
            ui.exibir(&format!(
                "{}Bateria substituída! Lanterna recarregada (+{} turnos).{}",
                DOS_VERDE, TURNOS_BATERIA, RESET
            ));
            return Some("turno");
        } else if norm_item == "alicate" && jogo.sala_atual == "sala de energia" {
            jogo.fios_cortados_inventario = true;
            audio.som_metal();
            ui.exibir(&format!(
                "{}Você cortou os cabos de alta tensão do painel central!{}",
                DOS_VERDE, RESET
            ));
            return Some("turno");
        } else if norm_item == "galao de gasolina" {
            jogo.inventario.remove(idx);
            jogo.gasolina_derramada = true;
            ui.exibir(&format!(
                "{}Você espalhou gasolina por todo o piso.{}",
                DOS_AMARELO, RESET
            ));
            return Some("turno");
        } else if norm_item == "isqueiro" {
            if jogo.gasolina_derramada || jogo.fios_cortados_inventario {
                jogo.incendio = true;
                jogo.noite_vencida = true;
                jogo.estado_atual = "FIM".to_string();
                ui.exibir(&format!(
                    "{}As chamas consomem a instalação rapidamente!{}",
                    DOS_VERMELHO, RESET
                ));
                return Some("turno");
            } else {
                jogo.turnos_luz = jogo.turnos_luz.max(2);
                ui.exibir("A chama fraca do isqueiro ilumina brevemente ao seu redor.");
                return Some("turno");
            }
        } else {
            ui.exibir(&format!(
                "Você tentou usar {}, mas nada aconteceu aqui.",
                nome_item
            ));
            return None;
        }
    }

    if let Some(alvo) = cmd
        .strip_prefix("inspecionar ")
        .or_else(|| cmd.strip_prefix("ver "))
    {
        let busca = alvo.trim();
        // Tenta exibir imagem ou secret image associada ao nome
        let _ = media.exibir_imagem(busca, false, ui);

        let desc_itens = descricoes_itens();
        for (k, desc) in &desc_itens {
            if normalizar(k) == busca {
                ui.exibir(&format!("{}{}:{} {}", DOS_AMARELO, k, RESET, desc));
                return None;
            }
        }

        if let Some(sala) = jogo.mapa.get(&jogo.sala_atual) {
            for (obj, texto) in &sala.inspecionaveis {
                if normalizar(obj) == busca {
                    ui.exibir(&format!("{}{}:{} {}", DOS_CIANO, obj, RESET, texto));
                    return None;
                }
            }
        }
        ui.exibir("Nada de especial encontrado ao inspecionar isso.");
        return None;
    }

    None
}