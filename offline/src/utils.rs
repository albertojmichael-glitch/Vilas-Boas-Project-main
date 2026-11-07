use crate::state::GameState;
use crate::ui::{DOS_AMARELO, DOS_VERMELHO, RESET};
use crate::TerminalBuffer;
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct ConquistaInfo {
    pub id: &'static str,
    pub chave_db: &'static str,
    pub nome: &'static str,
    pub desc: &'static str,
    pub icone: &'static str,
}

/// CONQUISTAS_DB 100% idêntico ao /server/villas_boas/utils.py
pub const CONQUISTAS_DB: [ConquistaInfo; 14] = [
    ConquistaInfo {
        chave_db: "primeira_morte",
        id: "primeira_morte",
        nome: "Sangue no Carpete",
        desc: "Bem-vindo ao Vilas Boas.",
        icone: "☠",
    },
    ConquistaInfo {
        chave_db: "mente_brilhante",
        id: "mente_brilhante",
        nome: "Mente Brilhante",
        desc: "Abra o cofre na primeira tentativa.",
        icone: "★",
    },
    ConquistaInfo {
        chave_db: "toureiro",
        id: "labirinto",
        nome: "Labirinto",
        desc: "Sobreviva à Sala de Energia.",
        icone: "☄",
    },
    ConquistaInfo {
        chave_db: "acumulador",
        id: "acumulador",
        nome: "Acumulador",
        desc: "Encha todos os 9 espaços do inventário.",
        icone: "☑",
    },
    ConquistaInfo {
        chave_db: "glicose",
        id: "glicose",
        nome: "Glicose Duvidosa",
        desc: "Coma o doce velho encontrado no chão.",
        icone: "✴",
    },
    ConquistaInfo {
        chave_db: "trapaceiro",
        id: "trapaceiro",
        nome: "Hacker",
        desc: "Ative o God Mode.",
        icone: "☣",
    },
    ConquistaInfo {
        chave_db: "speedrun",
        id: "speedrun",
        nome: "Corredor Noturno",
        desc: "Zere o jogo no Modo Desafio: Speedrun (Menos de 5 minutos reais).",
        icone: "⏱",
    },
    ConquistaInfo {
        chave_db: "fantasma",
        id: "fantasma",
        nome: "Fantasma",
        desc: "Zere no Modo Desafio: Fantasma (Sem deixar o barulho passar de 50%).",
        icone: "☹",
    },
    ConquistaInfo {
        chave_db: "breu_total",
        id: "breu_total",
        nome: "Breu Total",
        desc: "Zere no Modo Desafio: Breu Total (Sem luz desde o início).",
        icone: "👁",
    },
    ConquistaInfo {
        chave_db: "normal_zerado",
        id: "normal_zerado",
        nome: "Fim do Expediente",
        desc: "Sobreviva à noite na dificuldade Normal.",
        icone: "♨",
    },
    ConquistaInfo {
        chave_db: "final_bom",
        id: "final_bom",
        nome: "Juntos Novamente",
        desc: "Alcance o Final Neutro.",
        icone: "☀",
    },
    ConquistaInfo {
        chave_db: "final_verdadeiro",
        id: "final_verdadeiro",
        nome: "A Verdade",
        desc: "Alcance o Final Verdadeiro.",
        icone: "✦",
    },
    ConquistaInfo {
        chave_db: "pesadelo_neutro",
        id: "pesadelo_neutro",
        nome: "Pesadelo Superado",
        desc: "Alcance o Final Neutro no modo PESADELO.",
        icone: "♛",
    },
    ConquistaInfo {
        chave_db: "pesadelo_verdadeiro",
        id: "pesadelo_verdadeiro",
        nome: "Sonhador",
        desc: "Alcance o Final Verdadeiro no modo PESADELO.",
        icone: "♚",
    },
];

pub fn remover_acentos(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' | 'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' | 'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => 'u',
            'ç' | 'Ç' => 'c',
            _ => c,
        })
        .collect()
}

pub fn normalizar(texto: &str) -> String {
    remover_acentos(texto)
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn extrair_argumentos(comando_bruto: &str) -> Vec<String> {
    comando_bruto
        .split_whitespace()
        .map(normalizar)
        .filter(|s| !s.is_empty())
        .collect()
}

fn similaridade(a: &str, b: &str) -> f64 {
    if a == b {
        return 1.0;
    }
    let va: Vec<char> = a.chars().collect();
    let vb: Vec<char> = b.chars().collect();
    if va.is_empty() || vb.is_empty() {
        return 0.0;
    }
    let mut dp = vec![vec![0usize; vb.len() + 1]; va.len() + 1];
    for i in 0..=va.len() {
        dp[i][0] = i;
    }
    for j in 0..=vb.len() {
        dp[0][j] = j;
    }
    for i in 1..=va.len() {
        for j in 1..=vb.len() {
            let cost = usize::from(va[i - 1] != vb[j - 1]);
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    let dist = dp[va.len()][vb.len()];
    let max_len = va.len().max(vb.len());
    1.0 - (dist as f64 / max_len as f64)
}

/// Port fiel de `encontrar_melhor_match` do utils.py
pub fn encontrar_melhor_match(termo: &str, lista_opcoes: &[String], cutoff: f64) -> Option<String> {
    let t = normalizar(termo);
    if t.is_empty() || lista_opcoes.is_empty() {
        return None;
    }
    for op in lista_opcoes {
        if normalizar(op) == t {
            return Some(op.clone());
        }
    }
    for op in lista_opcoes {
        let nop = normalizar(op);
        if nop.starts_with(&t) || t.starts_with(&nop) {
            return Some(op.clone());
        }
    }
    for op in lista_opcoes {
        let nop = normalizar(op);
        if nop.contains(&t) || t.contains(&nop) {
            return Some(op.clone());
        }
    }
    let mut melhor: Option<(f64, String)> = None;
    for op in lista_opcoes {
        let sim = similaridade(&t, &normalizar(op));
        if sim >= cutoff && melhor.as_ref().map_or(true, |(m, _)| sim > *m) {
            melhor = Some((sim, op.clone()));
        }
    }
    melhor.map(|(_, s)| s)
}

/// Port fiel de `corromper_texto` do utils.py (caracteres Zalgo 0x0300..0x036F)
pub fn corromper_texto(texto: &str, intensidade: f64) -> String {
    if intensidade <= 0.0 {
        return texto.to_string();
    }
    let mut rng = rand::thread_rng();
    let mut resultado = String::with_capacity(texto.len() * 2);
    for ch in texto.chars() {
        resultado.push(ch);
        if ch.is_alphabetic() && rng.gen::<f64>() < intensidade {
            let num_zalgos = rng.gen_range(1..=3);
            for _ in 0..num_zalgos {
                if let Some(z) = char::from_u32(rng.gen_range(0x0300..=0x036F)) {
                    resultado.push(z);
                }
            }
        }
    }
    resultado
}

/// Port fiel de `processar_perseguidores_por_tempo` do utils.py
pub fn processar_perseguidores_por_tempo(jogo: &mut GameState, ui: &mut TerminalBuffer) {
    let salas_seguras_ia = [
        "saida",
        "cama",
        "final_bom",
        "morte",
        "tubo de ventilação",
        "01",
    ];

    if jogo.dificuldade_escolhida == "NORMAL" {
        if !salas_seguras_ia.contains(&jogo.sala_atual.as_str()) {
            jogo.turnos_mesma_sala += 1;
            if jogo.turnos_mesma_sala == jogo.turnos_perseguidor_aviso {
                ui.exibir(&format!(
                    "\n{}Você escuta ruídos metálicos pesados ecoando no corredor próximo...{}",
                    DOS_AMARELO, RESET
                ));
            } else if jogo.turnos_mesma_sala >= jogo.turnos_perseguidor_morte {
                ui.exibir(&format!(
                    "\n{}\n{}Você ficou muito tempo parado. Algo entrou na sua sala...\n{}{}",
                    "=".repeat(50),
                    DOS_VERMELHO,
                    RESET,
                    "=".repeat(50)
                ));
                if jogo.dev_mode {
                    ui.exibir(&format!(
                        "{}[DEV] Cronômetro do corredor zerado: você seria morto aqui. Reiniciado.{}",
                        DOS_AMARELO, RESET
                    ));
                    jogo.turnos_mesma_sala = 0;
                } else {
                    jogo.sala_atual = "morte".to_string();
                }
            }
        } else {
            jogo.turnos_mesma_sala = 0;
        }
    } else if jogo.dificuldade_escolhida == "PESADELO"
        && jogo.posicao_perseguidor != "morte"
        && !salas_seguras_ia.contains(&jogo.sala_atual.as_str())
    {
        let mut conexoes = Vec::new();
        if let Some(sala_monstro) = jogo.mapa.get(&jogo.posicao_perseguidor) {
            for (d, dest) in sala_monstro.saidas_validas(&jogo.mapa) {
                let _ = d;
                if !salas_seguras_ia.contains(&dest.as_str()) {
                    conexoes.push(dest);
                }
            }
        }
        let mut rng = rand::thread_rng();
        if !conexoes.is_empty() && rng.gen::<f64>() < 0.40 {
            if let Some(escolhida) = conexoes.choose(&mut rng) {
                jogo.posicao_perseguidor = escolhida.clone();
            }
        }

        if jogo.posicao_perseguidor == jogo.sala_atual && jogo.dev_mode {
            ui.exibir(&format!(
                "{}[DEV] 'Ela' te encontrou em {}: você seria morto. Ela recuou.{}",
                DOS_AMARELO, jogo.sala_atual, RESET
            ));
            jogo.posicao_perseguidor = "palco".to_string();
        } else if jogo.posicao_perseguidor == jogo.sala_atual {
            ui.exibir(&format!("\n{}", "=".repeat(50)));
            ui.exibir(&format!(
                "{}A porta quebra. Ela te encontrou{}",
                DOS_VERMELHO, RESET
            ));
            jogo.sala_atual = "morte".to_string();
        } else if let Some(sala_jog) = jogo.mapa.get(&jogo.sala_atual) {
            let vizinhos: Vec<String> = sala_jog
                .saidas_validas(&jogo.mapa)
                .into_iter()
                .map(|(_, dest)| dest)
                .collect();
            if vizinhos.contains(&jogo.posicao_perseguidor) {
                ui.exibir(&format!(
                    "\n{}O chão vibra. Você ouve passos de metal maciço na sala ao lado...{}",
                    DOS_AMARELO, RESET
                ));
            }
        }
    }
}

/// Port fiel de `atualizar_eventos_de_tempo` do utils.py
pub fn atualizar_eventos_de_tempo(jogo: &mut GameState, ui: &mut TerminalBuffer) {
    if jogo.god_mode {
        jogo.hp = 9999;
        jogo.turnos_luz = 9999;
        if jogo.dev_ia {
            processar_perseguidores_por_tempo(jogo, ui);
        }
        return;
    }

    let salas_iluminadas = [
        "sala de fliperamas",
        "hall de entrada",
        "sala de refrigeração",
        "quarto de refrigeração",
        "01",
        "balcão",
        "mesas de jantar",
        "sala de jantar",
    ];

    if jogo.amanheceu || salas_iluminadas.contains(&jogo.sala_atual.as_str()) {
        jogo.turnos_no_escuro = 0;
    } else if jogo.turnos_luz > 0 {
        jogo.turnos_luz -= 1;
        jogo.turnos_no_escuro = 0;
        if jogo.turnos_luz == 2 {
            ui.buffer_efeito("@@GLITCH_LUZ@@");
            ui.exibir(&format!(
                "\n{}Sua lanterna dá uma leve piscada. A luz está enfraquecendo...{}",
                DOS_AMARELO, RESET
            ));
        } else if jogo.turnos_luz == 1 {
            ui.buffer_efeito("@@GLITCH_LUZ@@");
            ui.exibir(&format!(
                "\n{}A luz da lanterna falha quase que totalmente. Vai apagar a qualquer momento.{}",
                DOS_VERMELHO, RESET
            ));
        } else if jogo.turnos_luz == 0 {
            ui.exibir(&format!(
                "\n{}A escuridão volta a dominar... Sua fonte de luz se apagou{}",
                DOS_VERMELHO, RESET
            ));
        }
    } else {
        jogo.turnos_no_escuro += 1;
        if jogo.turnos_no_escuro == 1 {
            ui.exibir(&format!(
                "\n{}As sombras parecem se mexer nos cantos da sua visão...{}",
                DOS_AMARELO, RESET
            ));
        } else if jogo.turnos_no_escuro == 3 {
            jogo.nivel_barulho = 100;
            jogo.ai_alvo = Some(jogo.sala_atual.clone());
            ui.exibir(&format!(
                "\n{}No escuro absoluto, você entra em pânico e esbarra nos móveis. O barulho ecoa pelo corredor!{}",
                DOS_VERMELHO, RESET
            ));
        } else if jogo.turnos_no_escuro >= 5 {
            jogo.hp -= 1;
            jogo.turnos_no_escuro = 0;
            if jogo.hp > 0 {
                ui.exibir(&format!(
                    "\n{}Você tenta andar no escuro, tropeça violentamente e se machuca! (-1 HP){}",
                    DOS_VERMELHO, RESET
                ));
            } else {
                ui.exibir(&format!(
                    "\n{}Você cai de mau jeito no escuro e bate a cabeça. Você não consegue mais levantar...{}",
                    DOS_VERMELHO, RESET
                ));
                jogo.sala_atual = "morte".to_string();
            }
        }
    }

    if jogo.incendio {
        jogo.turnos_fuga -= 1;
        ui.exibir(&format!(
            "\n{}O RESTAURANTE ESTÁ DESMORONANDO ({} turnos para fugir){}",
            DOS_VERMELHO, jogo.turnos_fuga, RESET
        ));
        if jogo.turnos_fuga <= 0 {
            ui.exibir(&format!(
                "\n{}O teto desaba sobre você. O fogo consome o que restou.{}",
                DOS_VERMELHO, RESET
            ));
            jogo.sala_atual = "morte".to_string();
        }
    }

    if jogo.turnos_enjoado > 0 {
        ui.exibir(&format!(
            "\n{}Você está enjoado e com tontura... Seus olhos embaçam.{}",
            DOS_AMARELO, RESET
        ));
        if jogo.turnos_luz > 0
            && !jogo.amanheceu
            && !salas_iluminadas.contains(&jogo.sala_atual.as_str())
        {
            jogo.turnos_luz -= 1;
        }
        jogo.turnos_enjoado -= 1;
    }

    processar_perseguidores_por_tempo(jogo, ui);
}