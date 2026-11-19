//! IA da "Ela", a perseguidora do NORMAL e do PESADELO.
//! Port 1:1 de `server/villas_boas/ela.py` (veja a documentação lá).

use crate::data::{chaves_nao_saida, PerfilEla, DUTOS_ELA, SALA_INICIAL_ELA};
use crate::state::GameState;
use crate::ui::{DOS_AMARELO, DOS_VERMELHO, RESET};
use crate::TerminalBuffer;
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::{HashMap, VecDeque};

/// Salas onde ela nunca entra.
pub const SALAS_SEGURAS_ELA: [&str; 6] = [
    "saida",
    "cama",
    "final_bom",
    "morte",
    "tubo de ventilação",
    "01",
];

pub fn ela_ativa_no_modo(jogo: &GameState) -> bool {
    matches!(jogo.dificuldade_escolhida.as_str(), "NORMAL" | "PESADELO")
}

/// Saídas (direção, sala) que ela consegue percorrer a pé a partir de `sala`.
fn saidas(jogo: &GameState, sala: &str) -> Vec<(String, String)> {
    let nao_saida = chaves_nao_saida();
    let Some(dados) = jogo.mapa.get(sala) else {
        return Vec::new();
    };
    dados
        .conexoes
        .iter()
        .filter_map(|(k, v)| {
            let dest = v.as_deref()?;
            (!nao_saida.contains(&k.as_str())
                && jogo.mapa.contains_key(dest)
                && !SALAS_SEGURAS_ELA.contains(&dest)
                && dest != sala)
                .then(|| (k.clone(), dest.to_string()))
        })
        .collect()
}

/// BFS pelas salas que ela percorre: {sala: distância}.
fn distancias(jogo: &GameState, origem: &str) -> HashMap<String, usize> {
    let mut dist = HashMap::new();
    if !jogo.mapa.contains_key(origem) {
        return dist;
    }
    dist.insert(origem.to_string(), 0);
    let mut fila = VecDeque::from([origem.to_string()]);
    while let Some(sala) = fila.pop_front() {
        let d = dist[&sala];
        for (_, viz) in saidas(jogo, &sala) {
            if !dist.contains_key(&viz) {
                dist.insert(viz.clone(), d + 1);
                fila.push_back(viz);
            }
        }
    }
    dist
}

/// Primeira sala do menor caminho dela de `origem` até `destino`.
fn proximo_passo(jogo: &GameState, origem: &str, destino: &str) -> Option<String> {
    if origem == destino {
        return None;
    }
    let mut anterior: HashMap<String, String> = HashMap::new();
    let mut fila = VecDeque::from([origem.to_string()]);
    let mut visitados = vec![origem.to_string()];
    while let Some(sala) = fila.pop_front() {
        if sala == destino {
            break;
        }
        for (_, viz) in saidas(jogo, &sala) {
            if !visitados.contains(&viz) {
                visitados.push(viz.clone());
                anterior.insert(viz.clone(), sala.clone());
                fila.push_back(viz);
            }
        }
    }
    let mut passo = destino.to_string();
    loop {
        let ant = anterior.get(&passo)?;
        if ant == origem {
            return Some(passo);
        }
        passo = ant.clone();
    }
}

/// (distância, direção) de quem está em `origem` até a sala do jogador; a direção
/// é a saída da sala do jogador por onde essa pessoa chegaria.
pub fn rastro_ate_jogador(jogo: &GameState, origem: &str) -> (Option<usize>, Option<String>) {
    if origem == jogo.sala_atual {
        return (Some(0), None);
    }
    let dist = distancias(jogo, origem);
    let nao_saida = chaves_nao_saida();
    let mut melhor: Option<(usize, String)> = None;
    if let Some(sala) = jogo.mapa.get(&jogo.sala_atual) {
        for (direcao, v) in &sala.conexoes {
            let Some(viz) = v.as_deref() else { continue };
            if nao_saida.contains(&direcao.as_str()) {
                continue;
            }
            if let Some(&d) = dist.get(viz) {
                if melhor.as_ref().is_none_or(|(m, _)| d < *m) {
                    melhor = Some((d, direcao.clone()));
                }
            }
        }
    }
    match melhor {
        Some((d, direcao)) => (Some(d + 1), Some(direcao)),
        None => (None, None),
    }
}

/// Salas com grade alcançáveis pelo duto a partir de `origem`.
pub fn saidas_de_duto(jogo: &GameState, origem: &str) -> Vec<String> {
    let alcance = jogo.perfil_ela().alcance_duto;
    let mut res: Vec<String> = distancias(jogo, origem)
        .into_iter()
        .filter(|(sala, d)| {
            (1..=alcance).contains(d) && DUTOS_ELA.contains(&sala.as_str()) && sala != origem
        })
        .map(|(sala, _)| sala)
        .collect();
    res.sort();
    res
}

fn entrar_no_duto<R: Rng>(
    jogo: &mut GameState,
    destinos: Vec<String>,
    alvo: Option<&str>,
    rng: &mut R,
) {
    let saida = match alvo {
        Some(alvo) => {
            let dist_alvo = distancias(jogo, alvo);
            destinos
                .into_iter()
                .min_by_key(|s| (dist_alvo.get(s).copied().unwrap_or(99), s.clone()))
        }
        None => destinos.choose(rng).cloned(),
    };
    if let Some(saida) = saida {
        jogo.ela_no_duto = true;
        jogo.ela_saida_duto = saida;
    }
}

fn mover<R: Rng>(jogo: &mut GameState, perfil: &PerfilEla, barulho_alto: bool, rng: &mut R) {
    let pos = jogo.posicao_perseguidor.clone();
    let dutos = if DUTOS_ELA.contains(&pos.as_str()) {
        saidas_de_duto(jogo, &pos)
    } else {
        Vec::new()
    };

    if barulho_alto && rng.gen::<f64>() < perfil.chance_rastrear {
        let alvo = jogo.sala_atual.clone();
        let dist_a_pe = distancias(jogo, &pos).get(&alvo).copied().unwrap_or(99);
        if !dutos.is_empty() && dist_a_pe > 1 && rng.gen::<f64>() < perfil.chance_usar_duto {
            entrar_no_duto(jogo, dutos, Some(&alvo), rng);
            return;
        }
        if let Some(passo) = proximo_passo(jogo, &pos, &alvo) {
            jogo.posicao_perseguidor = passo;
        }
        return;
    }

    if rng.gen::<f64>() >= perfil.chance_mover {
        return;
    }
    if !dutos.is_empty() && rng.gen::<f64>() < perfil.chance_usar_duto {
        entrar_no_duto(jogo, dutos, None, rng);
        return;
    }
    let vizinhos: Vec<String> = saidas(jogo, &pos).into_iter().map(|(_, v)| v).collect();
    if let Some(escolhida) = vizinhos.choose(rng) {
        jogo.posicao_perseguidor = escolhida.clone();
    }
}

fn avisar(jogo: &GameState, perfil: &PerfilEla, ui: &mut TerminalBuffer) {
    if jogo.ela_no_duto {
        if jogo.ela_saida_duto == jogo.sala_atual {
            ui.exibir(&format!(
                "\n{DOS_VERMELHO}A grade da ventilação range acima de você... algo se arrasta lá dentro.{RESET}"
            ));
        } else if let (Some(1), direcao) = rastro_ate_jogador(jogo, &jogo.ela_saida_duto) {
            let onde = direcao.map(|d| format!(" (à {d})")).unwrap_or_default();
            ui.exibir(&format!(
                "\n{DOS_AMARELO}Uma grade de ventilação range na sala ao lado{onde}.{RESET}"
            ));
        }
        return;
    }
    let (dist, direcao) = rastro_ate_jogador(jogo, &jogo.posicao_perseguidor);
    let Some(dist) = dist else { return };
    if dist < 1 || dist > perfil.aviso_distancia {
        return;
    }
    let vindo = direcao
        .map(|d| format!(" vindo da {d}"))
        .unwrap_or_default();
    if dist == 1 {
        ui.exibir(&format!(
            "\n{DOS_AMARELO}O chão vibra. Passos de metal maciço na sala ao lado{vindo}...{RESET}"
        ));
    } else {
        ui.exibir(&format!(
            "\n{DOS_AMARELO}Ao longe, passos pesados ecoam{vindo}...{RESET}"
        ));
    }
}

fn encontro<R: Rng>(jogo: &mut GameState, ui: &mut TerminalBuffer, rng: &mut R) {
    if jogo.dev_mode {
        ui.exibir(&format!(
            "{DOS_AMARELO}[DEV] 'Ela' te encontrou em {}: você seria atacado. Ela recuou.{RESET}",
            jogo.sala_atual
        ));
        jogo.posicao_perseguidor = SALA_INICIAL_ELA.to_string();
        return;
    }
    if jogo.escondido {
        ui.exibir(&format!(
            "\n{DOS_AMARELO}Ela entra na sala. Você prende a respiração no esconderijo... \
             os passos param, giram, e se afastam.{RESET}"
        ));
        let vizinhos: Vec<String> = saidas(jogo, &jogo.sala_atual)
            .into_iter()
            .map(|(_, v)| v)
            .collect();
        if let Some(v) = vizinhos.choose(rng) {
            jogo.posicao_perseguidor = v.clone();
        }
        return;
    }
    ui.exibir(&format!("\n{}", "=".repeat(50)));
    ui.exibir(&format!(
        "{DOS_VERMELHO}A porta quebra. Ela te encontrou! CORRA! (correr [direção]){RESET}"
    ));
    ui.pausa(2.0);
    jogo.estado_atual = "COMBATE_ANIMATRONICO".to_string();
    jogo.atacante_combate = "ela".to_string();
}

/// Um turno da IA da "Ela" (só quando o jogador gastou um turno).
pub fn processar_ela<R: Rng>(
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    barulho_alto: bool,
    rng: &mut R,
) {
    let perfil = jogo.perfil_ela();

    if jogo.ela_dormencia > 0 {
        jogo.ela_dormencia -= 1;
        if jogo.ela_dormencia == 0 {
            ui.exibir(&format!(
                "\n{DOS_AMARELO}Um rangido metálico longo vem do palco. Algo despertou.{RESET}"
            ));
        }
        return;
    }
    if jogo.ela_atordoada > 0 {
        jogo.ela_atordoada -= 1;
        return;
    }

    if jogo.ela_no_duto {
        if !jogo.ela_saida_duto.is_empty() {
            jogo.posicao_perseguidor = std::mem::take(&mut jogo.ela_saida_duto);
        }
        jogo.ela_no_duto = false;
        if jogo.posicao_perseguidor == jogo.sala_atual {
            ui.exibir(&format!(
                "\n{DOS_VERMELHO}A grade da ventilação cai no chão.{RESET}"
            ));
        }
    } else {
        mover(jogo, &perfil, barulho_alto, rng);
    }

    if !jogo.ela_no_duto && jogo.posicao_perseguidor == jogo.sala_atual {
        encontro(jogo, ui, rng);
    } else {
        avisar(jogo, &perfil, ui);
    }
}

/// Fuga bem-sucedida: ela fica atordoada e perde o rastro por alguns turnos.
pub fn atordoar_apos_fuga(jogo: &mut GameState) {
    jogo.ela_atordoada = jogo.perfil_ela().turnos_atordoada;
    jogo.ela_no_duto = false;
    jogo.ela_saida_duto.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::AudioManager;
    use crate::data::{
        PERFIL_ELA_PESADELO, PESADELO_ENERGIA_MAX_NOITE, PESADELO_ENERGIA_MIN_NOITE,
    };
    use crate::engine::processar_fluxo_jogo;
    use crate::media::MediaManager;
    use rand::{rngs::StdRng, SeedableRng};

    struct Mesa {
        jogo: GameState,
        ui: TerminalBuffer,
        audio: AudioManager,
        media: MediaManager,
    }

    impl Mesa {
        fn nova(modo: &str, sala: &str) -> Self {
            let mut jogo = GameState::new();
            jogo.configurar_dificuldade(modo, 0.0);
            jogo.estado_atual = "JOGO".to_string();
            jogo.sala_atual = sala.to_string();
            jogo.turnos_luz = 99;
            jogo.dev_ia_congelada = true; // isola a "Ela" do animatrônico do barulho
            Self {
                jogo,
                ui: TerminalBuffer::default(),
                audio: AudioManager::new(),
                media: MediaManager::new(),
            }
        }

        fn comando(&mut self, cmd: &str) {
            processar_fluxo_jogo(
                cmd,
                &mut self.jogo,
                &mut self.ui,
                &mut self.audio,
                &mut self.media,
            );
        }

        fn encontro(&mut self) {
            self.jogo.ela_dormencia = 0;
            self.jogo.posicao_perseguidor = self.jogo.sala_atual.clone();
            encontro(&mut self.jogo, &mut self.ui, &mut StdRng::seed_from_u64(1));
        }
    }

    #[test]
    fn presets_e_perfis_vem_do_json_compartilhado() {
        let mesa = Mesa::nova("PESADELO", "entrada");
        assert_eq!(mesa.jogo.energia_min_noite, PESADELO_ENERGIA_MIN_NOITE);
        assert_eq!(mesa.jogo.energia_max_noite, PESADELO_ENERGIA_MAX_NOITE);
        assert_eq!(
            mesa.jogo.ela_dormencia,
            PERFIL_ELA_PESADELO.turnos_dormencia
        );
        let (normal, pesadelo) = (
            crate::data::perfil_ela("NORMAL"),
            crate::data::perfil_ela("PESADELO"),
        );
        assert_eq!(pesadelo, PERFIL_ELA_PESADELO);
        assert!(normal.turnos_dormencia > pesadelo.turnos_dormencia);
        assert!(normal.chance_rastrear < pesadelo.chance_rastrear);
        assert!(ela_ativa_no_modo(&Mesa::nova("NORMAL", "entrada").jogo));
    }

    #[test]
    fn ela_comeca_dormindo_e_desperta() {
        let mut mesa = Mesa::nova("PESADELO", "sala 1");
        mesa.jogo.posicao_perseguidor = "sala 1".to_string();
        let mut rng = StdRng::seed_from_u64(7);
        for _ in 0..mesa.jogo.ela_dormencia {
            assert_eq!(mesa.jogo.estado_atual, "JOGO");
            processar_ela(&mut mesa.jogo, &mut mesa.ui, false, &mut rng);
        }
        assert_eq!(mesa.jogo.ela_dormencia, 0);
        assert_eq!(mesa.jogo.estado_atual, "JOGO");
    }

    #[test]
    fn dutos_so_levam_a_salas_com_grade_por_perto() {
        let mesa = Mesa::nova("PESADELO", "entrada");
        for origem in DUTOS_ELA {
            for saida in saidas_de_duto(&mesa.jogo, origem) {
                assert!(DUTOS_ELA.contains(&saida.as_str()));
                let d = distancias(&mesa.jogo, origem)[&saida];
                assert!((1..=2).contains(&d));
            }
        }
        for sem_grade in ["sala de fliperamas", "01", "04", "balcão"] {
            assert!(!DUTOS_ELA.contains(&sem_grade));
        }
    }

    #[test]
    fn duto_some_por_um_turno_e_reaparece_na_saida() {
        let mut mesa = Mesa::nova("PESADELO", "sala de jantar");
        mesa.jogo.ela_dormencia = 0;
        let mut rng = StdRng::seed_from_u64(11);
        let mut viu_duto = false;
        for _ in 0..500 {
            mesa.jogo.posicao_perseguidor = "palco".to_string();
            mesa.jogo.estado_atual = "JOGO".to_string();
            processar_ela(&mut mesa.jogo, &mut mesa.ui, true, &mut rng);
            if mesa.jogo.ela_no_duto {
                let saida = mesa.jogo.ela_saida_duto.clone();
                processar_ela(&mut mesa.jogo, &mut mesa.ui, true, &mut rng);
                assert!(!mesa.jogo.ela_no_duto);
                assert_eq!(mesa.jogo.posicao_perseguidor, saida);
                viu_duto = true;
                break;
            }
        }
        assert!(viu_duto);
    }

    #[test]
    fn encontro_vira_combate_e_correr_escapa_com_dano() {
        let mut mesa = Mesa::nova("PESADELO", "duas salas de festas");
        let hp = mesa.jogo.hp;
        mesa.encontro();
        assert_eq!(mesa.jogo.estado_atual, "COMBATE_ANIMATRONICO");
        assert_eq!(mesa.jogo.atacante_combate, "ela");
        mesa.comando("correr atrás");
        assert_eq!(mesa.jogo.sala_atual, "sala de jantar");
        assert_eq!(mesa.jogo.hp, hp - 1);
        assert_ne!(mesa.jogo.estado_atual, "FIM");
    }

    #[test]
    fn hesitar_diante_dela_mata() {
        let mut mesa = Mesa::nova("PESADELO", "duas salas de festas");
        mesa.encontro();
        mesa.comando("olhar");
        assert_eq!(mesa.jogo.sala_atual, "morte");
        assert_eq!(mesa.jogo.causa_morte, "ela");
    }

    #[test]
    fn esconder_evita_a_ela_e_tem_limite() {
        let mut mesa = Mesa::nova("PESADELO", "balcão");
        mesa.jogo.escondido = true;
        mesa.encontro();
        assert_eq!(mesa.jogo.estado_atual, "JOGO");
        assert_ne!(mesa.jogo.posicao_perseguidor, "balcão");

        mesa.jogo.ela_dormencia = 99;
        for _ in 0..3 {
            mesa.comando("esconder");
        }
        assert_eq!(mesa.jogo.turnos_escondido, 3);
        mesa.comando("esconder");
        assert_eq!(mesa.jogo.turnos_escondido, 3);
    }

    #[test]
    fn tropeco_no_escuro_nunca_mata() {
        let mut mesa = Mesa::nova("PESADELO", "sala de jantar");
        mesa.jogo.ela_dormencia = 100_000;
        for i in 0..400 {
            mesa.jogo.hp = 1;
            mesa.jogo.turnos_luz = 0;
            mesa.jogo.turnos_no_escuro = 0;
            mesa.comando(if i % 2 == 0 { "ir frente" } else { "ir atrás" });
            assert_ne!(mesa.jogo.sala_atual, "morte");
            assert_eq!(mesa.jogo.hp, 1);
        }
    }

    #[test]
    fn dica_falsa_do_jon_nao_tira_vida() {
        let mut mesa = Mesa::nova("PESADELO", "sala de fliperamas");
        mesa.comando("jogar jon");
        assert_eq!(mesa.jogo.estado_atual, "MINIGAME_JON");
        mesa.jogo.jon_dica_falsa = true;
        let certa = mesa.jogo.jon_caminho_certo[0].clone();
        let errada = ["f", "e", "d"].into_iter().find(|d| *d != certa).unwrap();
        let hp = mesa.jogo.hp;
        mesa.comando(errada);
        assert_eq!(mesa.jogo.hp, hp);
        assert_eq!(mesa.jogo.estado_atual, "JOGO");
    }

    #[test]
    fn esperar_ate_as_seis_vence_a_noite() {
        let mut mesa = Mesa::nova("PESADELO", "01");
        mesa.jogo.god_mode = true;
        crate::seguranca::iniciar(&mut mesa.jogo, &mut mesa.ui);
        mesa.jogo.seguranca_dados.as_mut().unwrap().turno = 23;
        mesa.comando("esperar");
        assert!(mesa.jogo.noite_vencida);
        assert_eq!(mesa.jogo.estado_atual, "JOGO");
    }

    #[test]
    fn animatronico_nao_entra_na_sala_segura() {
        let mut mesa = Mesa::nova("PESADELO", "01");
        mesa.jogo.dev_ia_congelada = false;
        mesa.jogo.ai_sala = "corredor".to_string();
        mesa.jogo.nivel_barulho = 100;
        for _ in 0..5 {
            mesa.jogo.avancar_ia_perseguidor();
            assert_ne!(mesa.jogo.ai_sala, "01");
        }
    }
}
