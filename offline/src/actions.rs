use crate::audio::AudioManager;
use crate::data::TURNOS_BATERIA;
use crate::media::MediaManager;
use crate::parser::{expandir_atalho_direcao, normalizar};
use crate::state::GameState;
use crate::TerminalBuffer;
use egui::Color32;

pub const COR_VERDE: Color32 = Color32::from_rgb(0, 255, 65);
pub const COR_VERMELHO: Color32 = Color32::from_rgb(255, 55, 55);
pub const COR_AMARELO: Color32 = Color32::from_rgb(255, 215, 0);
pub const COR_BRANCO: Color32 = Color32::from_rgb(235, 235, 235);
pub const COR_CIANO: Color32 = Color32::from_rgb(0, 225, 255);
pub const COR_CINZA: Color32 = Color32::from_rgb(130, 130, 130);

pub fn imprimir_contexto_sala(
    jogo: &GameState,
    term: &mut TerminalBuffer,
    media: &mut MediaManager,
) {
    term.push(
        "------------------------------------------------------------",
        COR_CINZA,
    );
    term.push(
        &format!("[ LOCAL: {} ]", jogo.sala_atual.to_uppercase()),
        COR_VERDE,
    );

    if jogo.turnos_luz <= 0 && !jogo.god_mode {
        term.push(
            "Está tudo completamente escuro! Sua lanterna está sem bateria.",
            COR_VERMELHO,
        );
        return;
    }

    if let Some(sala) = jogo.mapa.get(&jogo.sala_atual) {
        if let Some(ref img_sala) = sala.imagem {
            let _ = media.acionar_imagem(img_sala, false, false);
        } else {
            let _ = media.acionar_imagem(&jogo.sala_atual, false, false);
        }

        term.push(&sala.descricao, COR_BRANCO);

        if !sala.itens.is_empty() {
            term.push(
                &format!("Itens visíveis: {}", sala.itens.join(", ")),
                COR_AMARELO,
            );
        }

        if !sala.inspecionaveis.is_empty() {
            let mut objs: Vec<&String> = sala.inspecionaveis.keys().collect();
            objs.sort();
            let lista: Vec<String> = objs.into_iter().cloned().collect();
            term.push(
                &format!("Pistas/Objetos ('inspecionar <nome>'): {}", lista.join(", ")),
                COR_CINZA,
            );
        }

        let saidas = sala.saidas_validas(&jogo.mapa);
        if !saidas.is_empty() {
            let formatadas: Vec<String> = saidas
                .iter()
                .map(|(dir, dest)| format!("{} -> {}", dir, dest))
                .collect();
            term.push(&format!("Saídas: {}", formatadas.join(" | ")), COR_CIANO);
        }
    }
}

pub fn processar_comando_acao(
    comando_bruto: &str,
    jogo: &mut GameState,
    term: &mut TerminalBuffer,
    audio: &AudioManager,
    media: &mut MediaManager,
) -> Option<&'static str> {
    let cmd_norm = normalizar(comando_bruto);
    let cmd = expandir_atalho_direcao(&cmd_norm, &jogo.sinonimos);

    if cmd == "olhar" || cmd == "examinar sala" {
        if jogo.sala_atual == "sala de energia" && !jogo.noite_vencida {
            jogo.estado_atual = "MINIGAME_MINOTAURO".to_string();
            jogo.minotauro_pos = (0, 0);
            term.push(
                "[ TERMINAL DE ENERGIA - LABIRINTO DO MINOTAURO ]",
                COR_AMARELO,
            );
            term.push(
                "Use w/a/s/d para guiar a carga até a saída (2,2) ou digite 'sair'.",
                COR_BRANCO,
            );
            return Some("olhar");
        }
        imprimir_contexto_sala(jogo, term, media);
        return Some("olhar");
    }

    if cmd == "inventario" || cmd == "i" || cmd == "mochila" {
        term.push(
            &format!(
                "Mochila ({}/{}): {:?}",
                jogo.inventario.len(),
                jogo.obter_limite_inventario(),
                jogo.inventario
            ),
            COR_AMARELO,
        );
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
            if nova_sala == "saida"
                && !jogo.noite_vencida
                && !jogo.inventario.iter().any(|i| normalizar(i) == "chave mestra")
                && !jogo.dev_mode
            {
                term.push(
                    "A porta principal está trancada! Vença o turno ou use a Chave Mestra.",
                    COR_VERMELHO,
                );
                return None;
            }
            jogo.sala_atual = nova_sala;
            jogo.alterar_barulho(10);
            audio.som_porta();
            return Some("turno");
        } else {
            term.push(
                &format!("Não há saída para '{}' nesta sala.", direcao),
                COR_VERMELHO,
            );
            return None;
        }
    }

    if let Some(item_alvo) = cmd.strip_prefix("pegar ") {
        let item_busca = item_alvo.trim();
        let item_encontrado = jogo.mapa.get(&jogo.sala_atual).and_then(|s| {
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
                term.push(
                    &format!(
                        "Você coletou uma Bolsa! Capacidade expandida para {}.",
                        jogo.obter_limite_inventario()
                    ),
                    COR_VERDE,
                );
                return Some("turno");
            }

            if jogo.inventario.len() >= jogo.obter_limite_inventario() {
                term.push(
                    &format!(
                        "Sua mochila está cheia! Limite: {}.",
                        jogo.obter_limite_inventario()
                    ),
                    COR_VERMELHO,
                );
                return None;
            }

            if let Some(sala) = jogo.mapa.get_mut(&jogo.sala_atual) {
                sala.itens.retain(|i| i != &item_real);
            }
            let _ = media.acionar_imagem(&item_real, false, false);
            jogo.inventario.push(item_real.clone());
            term.push(&format!("Você pegou: {}.", item_real), COR_VERDE);
            return Some("turno");
        } else {
            term.push("Esse item não está nesta sala.", COR_VERMELHO);
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
            term.push(&format!("Você largou {} no chão.", removido), COR_AMARELO);
            return Some("largar");
        } else {
            term.push("Você não possui esse item na mochila.", COR_VERMELHO);
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
            term.push(
                &format!("Você não tem '{}' no inventário.", item_busca),
                COR_VERMELHO,
            );
            return None;
        };

        let nome_item = jogo.inventario[idx].clone();
        let norm_item = normalizar(&nome_item);

        if norm_item.contains("bateria") || norm_item.contains("pilha") {
            jogo.inventario.remove(idx);
            jogo.turnos_luz += TURNOS_BATERIA;
            jogo.turnos_no_escuro = 0;
            term.push(
                &format!(
                    "Bateria recarregada! (+{} turnos de luz).",
                    TURNOS_BATERIA
                ),
                COR_VERDE,
            );
            return Some("turno");
        } else if norm_item == "alicate" && jogo.sala_atual == "sala de energia" {
            jogo.fios_cortados_inventario = true;
            audio.som_metal();
            term.push(
                "Você cortou os cabos de alta tensão do gerador central!",
                COR_VERDE,
            );
            return Some("turno");
        } else if norm_item.contains("gasolina") {
            jogo.inventario.remove(idx);
            jogo.gasolina_derramada = true;
            term.push("Você despejou gasolina pelo chão.", COR_AMARELO);
            return Some("turno");
        } else if norm_item == "isqueiro" {
            if jogo.gasolina_derramada || jogo.fios_cortados_inventario {
                jogo.incendio = true;
                jogo.noite_vencida = true;
                jogo.estado_atual = "FIM".to_string();
                term.push("O fogo se espalha rapidamente pela unidade!", COR_VERMELHO);
                return Some("turno");
            } else {
                jogo.turnos_luz = jogo.turnos_luz.max(2);
                term.push(
                    "A chama do isqueiro ilumina brevemente os arredores.",
                    COR_AMARELO,
                );
                return Some("turno");
            }
        } else {
            term.push(
                &format!("Não é possível usar '{}' desta forma aqui.", nome_item),
                COR_AMARELO,
            );
            return None;
        }
    }

    if let Some(alvo) = cmd
        .strip_prefix("inspecionar ")
        .or_else(|| cmd.strip_prefix("examinar "))
        .or_else(|| cmd.strip_prefix("ver "))
    {
        let busca = alvo.trim();
        let _ = media.acionar_imagem(busca, false, false);

        for (k, desc) in &jogo.descricoes_itens {
            if normalizar(k) == busca {
                term.push(&format!("{}: {}", k, desc), COR_AMARELO);
                return None;
            }
        }

        if let Some(sala) = jogo.mapa.get(&jogo.sala_atual) {
            for (obj, texto) in &sala.inspecionaveis {
                if normalizar(obj) == busca {
                    term.push(&format!("{}: {}", obj, texto), COR_CIANO);
                    return None;
                }
            }
        }
        term.push("Você examina com cuidado, mas não nota nada além do óbvio.", COR_CINZA);
        return None;
    }

    None
}