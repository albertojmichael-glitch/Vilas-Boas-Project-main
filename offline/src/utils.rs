use crate::audio::AudioManager;
use crate::media::MediaManager;
use crate::parser::normalizar as normalizar_parser;
use crate::state::GameState;
use crate::ui::{DOS_AMARELO, DOS_BRANCO, DOS_VERDE, DOS_VERMELHO, RESET};
use crate::views;
use crate::TerminalBuffer;
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

pub fn desbloquear_conquista(
    id: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
) {
    let mut chave_canonica = id.to_string();
    let mut info_achada = None;
    for info in &CONQUISTAS_DB {
        if info.id == id || info.chave_db == id {
            chave_canonica = info.chave_db.to_string();
            info_achada = Some(info.clone());
            break;
        }
    }

    let nova_sessao = jogo.desbloquear_conquista(&chave_canonica);
    let mut nova_global = false;
    if !audio.config.conquistas_globais.contains(&chave_canonica) {
        audio.config.conquistas_globais.push(chave_canonica.clone());
        audio.config.salvar();
        nova_global = true;
    }

    if nova_sessao || nova_global {
        if let Some(info) = info_achada {
            ui.exibir(&format!(
                "\n{}🏆 CONQUISTA DESBLOQUEADA: {} — {}{}",
                DOS_AMARELO, info.nome, info.desc, RESET
            ));
        } else {
            ui.exibir(&format!(
                "\n{}🏆 CONQUISTA DESBLOQUEADA: {}{}",
                DOS_AMARELO, chave_canonica, RESET
            ));
        }
        audio.tocar_bipe(880.0, 120, 0.08);
    }
}

pub fn normalizar(texto: &str) -> String {
    normalizar_parser(texto)
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

/// Port fiel de `texto_cintilante` do utils.py
pub fn texto_cintilante(texto: &str) -> String {
    let cores = [DOS_VERMELHO, DOS_AMARELO, DOS_VERDE, DOS_BRANCO];
    let mut resultado = String::new();
    for (i, ch) in texto.chars().enumerate() {
        if !ch.is_whitespace() {
            resultado.push_str(cores[i % cores.len()]);
            resultado.push(ch);
        } else {
            resultado.push(ch);
        }
    }
    resultado.push_str(RESET);
    resultado
}

/// Port fiel de `processar_perseguidores_por_tempo` do utils.py: o cronômetro de
/// ficar parado (NORMAL) e a "Ela" (NORMAL e PESADELO, com IA por dificuldade).
pub fn processar_perseguidores_por_tempo(jogo: &mut GameState, ui: &mut TerminalBuffer) {
    use crate::data::{LIMIAR_ALERTA, LIMIAR_CACA};
    use crate::ela::{ela_ativa_no_modo, processar_ela, SALAS_SEGURAS_ELA};

    if !ela_ativa_no_modo(jogo) {
        return;
    }
    let mut rng = rand::thread_rng();
    if SALAS_SEGURAS_ELA.contains(&jogo.sala_atual.as_str()) {
        // Ela não entra aqui e perde o rastro, mas continua andando (e audível).
        jogo.turnos_mesma_sala = 0;
        processar_ela(jogo, ui, false, &mut rng);
        return;
    }

    if jogo.dificuldade_escolhida == "NORMAL" {
        jogo.turnos_mesma_sala += 1;
        if jogo.turnos_mesma_sala == jogo.turnos_perseguidor_aviso {
            ui.exibir(&format!(
                "\n{}Você escuta ruídos metálicos pesados ecoando no corredor próximo...{}",
                DOS_AMARELO, RESET
            ));
        } else if jogo.turnos_mesma_sala >= jogo.turnos_perseguidor_morte {
            ui.exibir(&format!(
                "\n{}Você ficou tempo demais parado. Algo ouviu você e está vindo...{}",
                DOS_VERMELHO, RESET
            ));
            jogo.turnos_mesma_sala = 0;
            jogo.ela_dormencia = 0;
            jogo.alterar_barulho(LIMIAR_CACA);
            jogo.ai_alvo = Some(jogo.sala_atual.clone());
        }
    }

    let barulho_alto = jogo.nivel_barulho >= LIMIAR_ALERTA;
    processar_ela(jogo, ui, barulho_alto, &mut rng);
}

/// Port fiel de `atualizar_eventos_de_tempo` do utils.py
pub fn atualizar_eventos_de_tempo(
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) {
    if jogo.god_mode {
        jogo.hp = 9999;
        jogo.turnos_luz = 9999;
        if jogo.dev_ia {
            processar_perseguidores_por_tempo(jogo, ui);
        }
        return;
    }

    let salas_iluminadas = crate::data::SALAS_ILUMINADAS;

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
            ui.pausa(1.5);
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
            jogo.turnos_no_escuro = 0;
            if !jogo.aplicar_dano(1) {
                ui.exibir(&format!(
                    "\n{}Você tenta andar no escuro, tropeça violentamente e se machuca! (-1 HP){}",
                    DOS_VERMELHO, RESET
                ));
            } else {
                ui.exibir(&format!(
                    "\n{}Você cai de mau jeito no escuro e bate a cabeça. Você não consegue mais levantar...{}",
                    DOS_VERMELHO, RESET
                ));
                jogo.causa_morte = "escuro".to_string();
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
            jogo.causa_morte = "incendio".to_string();
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

    if jogo.sala_atual == "morte" || jogo.hp <= 0 {
        views::dar_tela_de_morte(jogo, ui, audio, media);
    }
}
