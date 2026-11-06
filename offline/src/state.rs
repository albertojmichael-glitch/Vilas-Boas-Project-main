use crate::data::{
    chaves_nao_saida, salas_seguras_patrulha, ConquistaDef, DadosJogo, Sala,
    CHANCE_MOVER_PATRULHA, LIMIAR_ALERTA, LIMIAR_CACA, VIDA_NORMAL,
};
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;

pub const ARQUIVO_SAVE: &str = "savegame.json";
pub const DOS_VERDE: &str = "\x1b[92m";
pub const DOS_VERMELHO: &str = "\x1b[91m";
pub const DOS_AMARELO: &str = "\x1b[93m";
pub const DOS_BRANCO: &str = "\x1b[97m";
pub const DOS_CIANO: &str = "\x1b[96m";
pub const DOS_CINZA: &str = "\x1b[90m";
pub const RESET: &str = "\x1b[0m";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatorioIA {
    pub ativa: bool,
    pub distancia: usize,
    pub combate_iniciado: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub hp: i32,
    pub inventario: Vec<String>,
    pub turnos_luz: i32,
    pub turnos_no_escuro: i32,
    pub turnos_enjoado: i32,
    pub turnos_mesma_sala: i32,
    pub turnos_perseguidor_aviso: i32,
    pub turnos_perseguidor_morte: i32,
    pub turnos_fuga: i32,
    pub posicao_perseguidor: String,
    pub amanheceu: bool,
    pub texto_rapido: bool,
    pub modo_desafio: Option<String>,
    pub sala_atual: String,
    pub estado_atual: String,
    pub dificuldade_escolhida: String,
    pub nivel_barulho: i32,
    pub ai_sala: String,
    pub ai_estado: String,
    pub ai_alvo: Option<String>,
    pub bolsas_coletadas: usize,
    pub conquistas: Vec<String>,
    pub cofre_tentativas: i32,
    pub cofre_aberto: bool,
    pub erros_consecutivos: i32,
    pub noite_vencida: bool,
    pub fios_cortados_inventario: bool,
    pub gasolina_derramada: bool,
    pub incendio: bool,
    pub dev_mode: bool,
    pub god_mode: bool,
    pub dev_usado: bool,
    pub dev_ia: bool,
    pub dev_radar: bool,
    pub minotauro_pos: (usize, usize),
    pub seguranca_progresso: i32,
    pub mapa: HashMap<String, Sala>,
    #[serde(default)]
    pub descricoes_itens: HashMap<String, String>,
    #[serde(default)]
    pub sinonimos: HashMap<String, String>,
    #[serde(default)]
    pub ascii_arts: HashMap<String, String>,
    #[serde(default)]
    pub catalogo_conquistas: Vec<ConquistaDef>,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    pub fn new() -> Self {
        let dados = DadosJogo::carregar_tudo();
        let sala_inicial = if dados.mapa.contains_key("01") {
            "01".to_string()
        } else {
            dados
                .mapa
                .keys()
                .next()
                .cloned()
                .unwrap_or_else(|| "01".to_string())
        };

        Self {
            hp: VIDA_NORMAL,
            inventario: vec!["lanterna".to_string()],
            turnos_luz: 6,
            turnos_no_escuro: 0,
            turnos_enjoado: 0,
            turnos_mesma_sala: 0,
            turnos_perseguidor_aviso: 3,
            turnos_perseguidor_morte: 5,
            turnos_fuga: 6,
            posicao_perseguidor: "palco".to_string(),
            amanheceu: false,
            texto_rapido: false,
            modo_desafio: None,
            sala_atual: sala_inicial,
            estado_atual: "AGUARDANDO_DIR".to_string(),
            dificuldade_escolhida: "NORMAL".to_string(),
            nivel_barulho: 0,
            ai_sala: "palco".to_string(),
            ai_estado: "PATRULHA".to_string(),
            ai_alvo: None,
            bolsas_coletadas: 0,
            conquistas: Vec::new(),
            cofre_tentativas: 0,
            cofre_aberto: false,
            erros_consecutivos: 0,
            noite_vencida: false,
            fios_cortados_inventario: false,
            gasolina_derramada: false,
            incendio: false,
            dev_mode: false,
            god_mode: false,
            dev_usado: false,
            dev_ia: false,
            dev_radar: true,
            minotauro_pos: (0, 0),
            seguranca_progresso: 0,
            mapa: dados.mapa,
            descricoes_itens: dados.descricoes_itens,
            sinonimos: dados.sinonimos,
            ascii_arts: dados.ascii_arts,
            catalogo_conquistas: dados.conquistas,
        }
    }

    pub fn reiniciar_partida(&mut self, dificuldade: &str) {
        let conquistas_atuais = self.conquistas.clone();
        let dev_mode = self.dev_mode;
        let god_mode = self.god_mode;
        let dev_usado = self.dev_usado;
        *self = Self::new();
        self.dificuldade_escolhida = dificuldade.to_string();
        self.hp = if dificuldade == "PESADELO" {
            1
        } else {
            VIDA_NORMAL
        };
        self.estado_atual = "JOGO".to_string();
        self.conquistas = conquistas_atuais;
        self.dev_mode = dev_mode;
        self.god_mode = god_mode;
        self.dev_usado = dev_usado;
    }

    pub fn obter_limite_inventario(&self) -> usize {
        if self.dev_mode {
            return 999;
        }
        3 + (self.bolsas_coletadas * 3)
    }

    pub fn alterar_barulho(&mut self, delta: i32) {
        self.nivel_barulho = (self.nivel_barulho + delta).clamp(0, 100);
    }

    pub fn desbloquear_conquista(&mut self, id: &str) -> bool {
        if self.dev_usado && id != "trapaceiro" {
            return false;
        }
        if !self.conquistas.contains(&id.to_string()) {
            self.conquistas.push(id.to_string());
            return true;
        }
        false
    }

    pub fn atingiu_meta_acumulador(&self) -> bool {
        self.inventario.len() >= 9
    }

    pub fn salvar_local(&self) -> bool {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            return fs::write(ARQUIVO_SAVE, json).is_ok();
        }
        false
    }

    pub fn carregar_local() -> Option<Self> {
        let conteudo = fs::read_to_string(ARQUIVO_SAVE).ok()?;
        let mut carregado: Self = serde_json::from_str(&conteudo).ok()?;
        let dados = DadosJogo::carregar_tudo();
        if carregado.ascii_arts.is_empty() {
            carregado.ascii_arts = dados.ascii_arts;
        }
        Some(carregado)
    }

    pub fn save_existe() -> bool {
        std::path::Path::new(ARQUIVO_SAVE).exists()
    }

    pub fn avancar_ia_perseguidor(&mut self) -> RelatorioIA {
        let ia_congelada = self.god_mode && !self.dev_ia;
        if ia_congelada || self.estado_atual != "JOGO" {
            return RelatorioIA {
                ativa: false,
                distancia: 99,
                combate_iniciado: false,
            };
        }

        if self.nivel_barulho > 0 {
            self.nivel_barulho = (self.nivel_barulho - 10).max(0);
        }

        if self.nivel_barulho >= LIMIAR_CACA {
            self.ai_estado = "CACA".to_string();
            self.ai_alvo = Some(self.sala_atual.clone());
        } else if self.nivel_barulho >= LIMIAR_ALERTA {
            self.ai_estado = "ALERTA".to_string();
        } else {
            self.ai_estado = "PATRULHA".to_string();
        }

        let seguras = salas_seguras_patrulha();
        let nao_saida = chaves_nao_saida();
        let mut rng = rand::thread_rng();

        if self.ai_estado == "PATRULHA" {
            let mut vizinhos = Vec::new();
            if let Some(sala_obj) = self.mapa.get(&self.ai_sala) {
                for (k, v) in &sala_obj.conexoes {
                    if nao_saida.contains(&k.as_str()) {
                        continue;
                    }
                    if let Some(dest) = v.as_str() {
                        if self.mapa.contains_key(dest) && !seguras.contains(&dest) {
                            vizinhos.push(dest.to_string());
                        }
                    }
                }
            }
            if rng.gen::<f64>() > CHANCE_MOVER_PATRULHA && !vizinhos.is_empty() {
                if let Some(escolhida) = vizinhos.choose(&mut rng) {
                    self.ai_sala = escolhida.clone();
                }
            }
        } else {
            let alvo = self
                .ai_alvo
                .clone()
                .unwrap_or_else(|| self.sala_atual.clone());
            if let Some(caminho) = calcular_caminho_bfs(&self.mapa, &self.ai_sala, &alvo) {
                if caminho.len() > 1 && !seguras.contains(&caminho[1].as_str()) {
                    self.ai_sala = caminho[1].clone();
                }
            }
        }

        let combate = self.ai_sala == self.sala_atual;
        if combate {
            self.estado_atual = "COMBATE_ANIMATRONICO".to_string();
        }

        let dist = calcular_caminho_bfs(&self.mapa, &self.sala_atual, &self.ai_sala)
            .map(|c| c.len().saturating_sub(1))
            .unwrap_or(99);

        RelatorioIA {
            ativa: true,
            distancia: dist,
            combate_iniciado: combate,
        }
    }
}

pub fn calcular_caminho_bfs(
    mapa: &HashMap<String, Sala>,
    inicio: &str,
    destino: &str,
) -> Option<Vec<String>> {
    if inicio == destino {
        return Some(vec![inicio.to_string()]);
    }
    let nao_saida = chaves_nao_saida();
    let mut fila = VecDeque::new();
    let mut visitados = HashSet::new();

    fila.push_back(vec![inicio.to_string()]);
    visitados.insert(inicio.to_string());

    while let Some(caminho) = fila.pop_front() {
        let atual = caminho.last()?;
        if let Some(sala) = mapa.get(atual) {
            for (k, v) in &sala.conexoes {
                if nao_saida.contains(&k.as_str()) {
                    continue;
                }
                if let Some(vizinho) = v.as_str() {
                    if !mapa.contains_key(vizinho) || visitados.contains(vizinho) {
                        continue;
                    }
                    let mut novo_caminho = caminho.clone();
                    novo_caminho.push(vizinho.to_string());
                    if vizinho == destino {
                        return Some(novo_caminho);
                    }
                    visitados.insert(vizinho.to_string());
                    fila.push_back(novo_caminho);
                }
            }
        }
    }
    None
}