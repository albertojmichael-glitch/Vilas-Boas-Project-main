use crate::data::{
    chaves_nao_saida, salas_seguras_patrulha, ConquistaDef, DadosJogo, EventoAleatorio, Sala,
    CHANCE_MOVER_PATRULHA, ISQUEIRO_BARULHO, ISQUEIRO_LIMITE_LUZ, ISQUEIRO_TURNOS_BONUS,
    LIMIAR_ALERTA, LIMIAR_CACA, MAX_INVENTARIO, META_ACUMULADOR, REDUCAO_BARULHO_TURNO,
    SLOTS_POR_BOLSA, TURNOS_BATERIA, TURNOS_LUZ_INICIAL, VIDA_GOD_MODE, VIDA_NORMAL,
    VIDA_PESADELO,
};
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub const ARQUIVO_SAVE: &str = "autosave.json";
pub const ARQUIVO_SAVE_LEGADO: &str = "savegame.json";
pub const MAX_LOG_COMANDOS: usize = 100;

pub fn agora_segundos() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatorioIA {
    pub ativa: bool,
    pub distancia: usize,
    pub combate_iniciado: bool,
    pub ai_estado: String,
    pub ai_sala: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MinotauroData {
    pub px: i32,
    pub py: i32,
    pub mx: i32,
    pub my: i32,
    pub fusivel: (i32, i32),
    pub saida: (i32, i32),
    pub armadilhas: Vec<(i32, i32)>,
    pub bateria: i32,
    pub tem_fios: bool,
    pub tesoura_no_chao: bool,
    pub celular_inspecionado: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CaptchaData {
    pub pergunta: String,
    pub resposta: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SegurancaData {
    pub turno: i32,
    pub energia: i32,
    pub porta_fechada: bool,
    pub gerador_ligado: bool,
    pub usos_sistema_turno: i32,
    pub camera_quebrada: bool,
    pub deteccao_quebrada: bool,
    pub relogio_quebrado: bool,
    pub captcha_ativo: Option<CaptchaData>,
    pub sistema_alvo_conserto: Option<String>,
    pub rick_pos: i32,
    pub jon_pos: i32,
    pub caroline_ativa: bool,
    pub caroline_local: String,
    pub indio_ativo: bool,
    pub alberto_msg_falsa: bool,
    pub calor_ativo: bool,
    pub fios_cortados: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub hp: i32,
    pub inventario: Vec<String>,
    pub turnos_luz: i32,
    pub turnos_no_escuro: i32,
    pub turnos_enjoado: i32,
    pub sala_atual: String,
    pub cofre_tentativas: i32,
    #[serde(default)]
    pub cofre_aberto: bool,
    #[serde(default)]
    pub jon_erros: usize,
    #[serde(default)]
    pub jon_vencido: bool,
    #[serde(default)]
    pub jon_passos: Vec<String>,
    #[serde(default)]
    pub jon_passo_atual: usize,
    #[serde(default)]
    pub jon_passos_dados: usize,
    #[serde(default)]
    pub jon_caminho_certo: Vec<String>,
    #[serde(default)]
    pub consertos_vencido: bool,
    #[serde(default)]
    pub consertos_escolhas: HashMap<String, String>,
    #[serde(default)]
    pub julgamento_vencido: bool,
    #[serde(default)]
    pub julgamento_pontos: i32,
    #[serde(default)]
    pub vitimas_restantes: Vec<String>,
    pub bolsas_coletadas: usize,
    pub turnos_mesma_sala: i32,
    #[serde(default = "default_dificuldade")]
    pub dificuldade: String,
    #[serde(default = "default_dificuldade")]
    pub dificuldade_escolhida: String,
    #[serde(default = "default_velocidade_texto")]
    pub velocidade_texto: f32,
    #[serde(default = "default_custom_bat")]
    pub custom_bat: i32,
    #[serde(default = "default_custom_inv")]
    pub custom_inv: usize,
    #[serde(default = "default_custom_hp")]
    pub custom_hp: i32,
    #[serde(default = "default_chance_sprint")]
    pub chance_sprint_minotauro: i32,
    pub turnos_perseguidor_aviso: i32,
    pub turnos_perseguidor_morte: i32,
    #[serde(default)]
    pub bateria_ajuda_gerada: bool,
    #[serde(default = "default_energia_min")]
    pub energia_min_noite: i32,
    #[serde(default = "default_energia_max")]
    pub energia_max_noite: i32,
    #[serde(default = "default_furia")]
    pub furia_noite: i32,
    pub fios_cortados_inventario: bool,
    pub noite_vencida: bool,
    pub incendio: bool,
    pub turnos_fuga: i32,
    #[serde(default = "default_isqueiro_usos")]
    pub isqueiro_usos: i32,
    pub posicao_perseguidor: String,
    pub estado_atual: String,
    pub god_mode: bool,
    pub dev_mode: bool,
    pub dev_usado: bool,
    pub dev_radar: bool,
    #[serde(default = "default_true")]
    pub dev_radar_ativo: bool,
    pub dev_ia: bool,
    #[serde(default)]
    pub dev_ia_congelada: bool,
    pub erros_consecutivos: i32,
    #[serde(default)]
    pub alberto_desativado: bool,
    #[serde(default, alias = "texto_rapido")]
    pub fast_mode: bool,
    pub amanheceu: bool,
    pub ai_sala: String,
    pub ai_estado: String,
    #[serde(default)]
    pub ai_alvo: Option<String>,
    pub nivel_barulho: i32,
    pub conquistas: Vec<String>,
    #[serde(default)]
    pub minotauro_dados: Option<MinotauroData>,
    #[serde(default)]
    pub seguranca_dados: Option<SegurancaData>,
    #[serde(default)]
    pub log_comandos: Vec<String>,
    #[serde(default = "agora_segundos")]
    pub tempo_inicio: f64,
    #[serde(default)]
    pub tempo_total_segundos: f64,
    pub mapa: HashMap<String, Sala>,
    #[serde(default)]
    pub descricoes_itens: HashMap<String, String>,
    #[serde(default)]
    pub eventos_aleatorios: Vec<EventoAleatorio>,
    #[serde(default)]
    pub sinonimos: HashMap<String, String>,
    #[serde(default)]
    pub ascii_arts: HashMap<String, String>,
    #[serde(default)]
    pub catalogo_conquistas: Vec<ConquistaDef>,
}

fn default_dificuldade() -> String {
    "NORMAL".to_string()
}
fn default_velocidade_texto() -> f32 {
    0.02
}
fn default_custom_bat() -> i32 {
    TURNOS_LUZ_INICIAL
}
fn default_custom_inv() -> usize {
    MAX_INVENTARIO
}
fn default_custom_hp() -> i32 {
    VIDA_NORMAL
}
fn default_chance_sprint() -> i32 {
    15
}
fn default_energia_min() -> i32 {
    90
}
fn default_energia_max() -> i32 {
    100
}
fn default_furia() -> i32 {
    1
}
fn default_isqueiro_usos() -> i32 {
    3
}
fn default_true() -> bool {
    true
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    pub fn new() -> Self {
        let dados = DadosJogo::carregar_tudo();
        let sala_inicial = if dados.mapa.contains_key("entrada") {
            "entrada".to_string()
        } else if dados.mapa.contains_key("01") {
            "01".to_string()
        } else {
            dados
                .mapa
                .keys()
                .next()
                .cloned()
                .unwrap_or_else(|| "entrada".to_string())
        };

        Self {
            hp: VIDA_NORMAL,
            inventario: vec!["lanterna".to_string()],
            turnos_luz: TURNOS_LUZ_INICIAL,
            turnos_no_escuro: 0,
            turnos_enjoado: 0,
            sala_atual: sala_inicial,
            cofre_tentativas: 0,
            cofre_aberto: false,
            jon_erros: 0,
            jon_vencido: false,
            jon_passos: Vec::new(),
            jon_passo_atual: 0,
            jon_passos_dados: 0,
            jon_caminho_certo: Vec::new(),
            consertos_vencido: false,
            consertos_escolhas: HashMap::new(),
            julgamento_vencido: false,
            julgamento_pontos: 0,
            vitimas_restantes: Vec::new(),
            bolsas_coletadas: 0,
            turnos_mesma_sala: 0,
            dificuldade: "NORMAL".to_string(),
            dificuldade_escolhida: "NORMAL".to_string(),
            velocidade_texto: 0.02,
            custom_bat: TURNOS_LUZ_INICIAL,
            custom_inv: MAX_INVENTARIO,
            custom_hp: VIDA_NORMAL,
            chance_sprint_minotauro: 15,
            turnos_perseguidor_aviso: 3,
            turnos_perseguidor_morte: 5,
            bateria_ajuda_gerada: false,
            energia_min_noite: 90,
            energia_max_noite: 100,
            furia_noite: 1,
            fios_cortados_inventario: false,
            noite_vencida: false,
            incendio: false,
            turnos_fuga: 5,
            isqueiro_usos: 3,
            posicao_perseguidor: "palco".to_string(),
            estado_atual: "AGUARDANDO_DIR".to_string(),
            god_mode: false,
            dev_mode: false,
            dev_usado: false,
            dev_radar: true,
            dev_radar_ativo: true,
            dev_ia: false,
            dev_ia_congelada: false,
            erros_consecutivos: 0,
            alberto_desativado: false,
            fast_mode: false,
            amanheceu: false,
            ai_sala: "palco".to_string(),
            ai_estado: "PATRULHA".to_string(),
            ai_alvo: None,
            nivel_barulho: 0,
            conquistas: Vec::new(),
            minotauro_dados: None,
            seguranca_dados: None,
            log_comandos: Vec::new(),
            tempo_inicio: agora_segundos(),
            tempo_total_segundos: 0.0,
            mapa: dados.mapa,
            descricoes_itens: dados.descricoes_itens,
            eventos_aleatorios: dados.eventos_aleatorios,
            sinonimos: dados.sinonimos,
            ascii_arts: dados.ascii_arts,
            catalogo_conquistas: dados.conquistas,
        }
    }

    pub fn novo() -> Self {
        Self::new()
    }

    pub fn tempo_decorrido_segundos(&self) -> u64 {
        let diff = (agora_segundos() - self.tempo_inicio).max(0.0);
        (self.tempo_total_segundos + diff) as u64
    }

    pub fn consumir_luz(&mut self, qtd: i32) {
        if self.god_mode {
            return;
        }
        self.turnos_luz = (self.turnos_luz - qtd).max(0);
    }

    pub fn registrar_comando(&mut self, comando: &str) {
        if !comando.is_empty() {
            self.log_comandos.push(comando.to_string());
            if self.log_comandos.len() > MAX_LOG_COMANDOS {
                let excesso = self.log_comandos.len() - MAX_LOG_COMANDOS;
                self.log_comandos.drain(0..excesso);
            }
        }
    }

    pub fn obter_limite_inventario(&self) -> usize {
        let base = if self.dificuldade == "CUSTOM" || self.dificuldade_escolhida == "CUSTOM" {
            self.custom_inv
        } else {
            MAX_INVENTARIO
        };
        base + (self.bolsas_coletadas * SLOTS_POR_BOLSA)
    }

    pub fn obter_hp_maximo(&self) -> i32 {
        if self.god_mode {
            VIDA_GOD_MODE
        } else if self.dificuldade == "CUSTOM" || self.dificuldade_escolhida == "CUSTOM" {
            self.custom_hp
        } else if self.dificuldade_escolhida == "PESADELO" || self.dificuldade == "PESADELO" {
            VIDA_PESADELO
        } else {
            VIDA_NORMAL
        }
    }

    pub fn tem_espaco_inventario(&self) -> bool {
        self.god_mode || self.inventario.len() < self.obter_limite_inventario()
    }

    pub fn equipar_bolsa(&mut self) -> usize {
        self.bolsas_coletadas += 1;
        self.obter_limite_inventario()
    }

    pub fn adicionar_ao_inventario(&mut self, item: &str) -> bool {
        self.adicionar_item_inventario(item)
    }

    pub fn adicionar_item_inventario(&mut self, item: &str) -> bool {
        if !self.tem_espaco_inventario() {
            return false;
        }
        self.inventario.push(item.to_string());
        if item == "fios cortados" {
            self.fios_cortados_inventario = true;
        }
        true
    }

    pub fn largar_item(&mut self, item: &str) -> bool {
        if let Some(pos) = self.inventario.iter().position(|x| x == item) {
            let removido = self.inventario.remove(pos);
            if let Some(sala) = self.mapa.get_mut(&self.sala_atual) {
                sala.itens.push(removido);
            }
            return true;
        }
        false
    }

    pub fn atingiu_meta_acumulador(&self) -> bool {
        self.inventario.len() >= META_ACUMULADOR
    }

    pub fn alterar_barulho(&mut self, delta: i32) -> i32 {
        self.nivel_barulho = (self.nivel_barulho + delta).clamp(0, 100);
        self.nivel_barulho
    }

    pub fn aplicar_dano(&mut self, quantidade: i32) -> bool {
        if self.god_mode {
            return false;
        }
        self.hp = (self.hp - quantidade).max(0);
        if self.hp <= 0 {
            self.sala_atual = "morte".to_string();
            return true;
        }
        false
    }

    pub fn curar(&mut self, item: &str, quantidade: i32) -> bool {
        if self.hp >= self.obter_hp_maximo() {
            return false;
        }
        self.hp = (self.hp + quantidade).min(self.obter_hp_maximo());
        if let Some(pos) = self.inventario.iter().position(|x| x == item) {
            self.inventario.remove(pos);
        }
        true
    }

    pub fn recarregar_lanterna(&mut self) -> &'static str {
        if let Some(pos) = self.inventario.iter().position(|x| x == "bateria nova") {
            self.inventario.remove(pos);
        }
        if self.dificuldade_escolhida == "DESAFIO_BREU" || self.dificuldade == "DESAFIO_BREU" {
            return "quebrada_breu";
        }
        self.turnos_luz = TURNOS_BATERIA;
        "recarregada"
    }

    pub fn acender_isqueiro(&mut self) -> bool {
        if self.turnos_luz > ISQUEIRO_LIMITE_LUZ {
            return false;
        }
        self.turnos_luz += ISQUEIRO_TURNOS_BONUS;
        self.alterar_barulho(ISQUEIRO_BARULHO);
        true
    }

    pub fn configurar_dificuldade(&mut self, modo: &str, velocidade: f32) {
        self.dificuldade = modo.to_string();
        self.dificuldade_escolhida = modo.to_string();
        self.velocidade_texto = velocidade;
        self.fast_mode = velocidade == 0.0;

        if modo == "GOD MODE" {
            self.god_mode = true;
            self.hp = VIDA_GOD_MODE;
            self.furia_noite = 0;
            self.energia_min_noite = VIDA_GOD_MODE;
            self.energia_max_noite = VIDA_GOD_MODE;
            self.turnos_luz = VIDA_GOD_MODE;
        } else if modo == "PESADELO" {
            self.god_mode = false;
            self.hp = VIDA_PESADELO;
            self.furia_noite = 2;
            self.energia_min_noite = 70;
            self.energia_max_noite = 82;
        } else if modo == "CUSTOM" {
            self.god_mode = false;
            self.hp = self.custom_hp;
            self.turnos_luz = self.custom_bat;
            self.furia_noite = 1;
            self.energia_min_noite = 90;
            self.energia_max_noite = 100;
        } else {
            self.god_mode = false;
            self.hp = VIDA_NORMAL;
            self.furia_noite = 1;
            self.energia_min_noite = 100;
            self.energia_max_noite = 100;
            if modo == "DESAFIO_BREU" {
                self.turnos_luz = 0;
            }
        }
    }

    pub fn resetar_para_novo_jogo(&mut self) {
        let conquistas_atuais = self.conquistas.clone();
        *self = Self::new();
        self.conquistas = conquistas_atuais;
    }

    pub fn desbloquear_conquista(&mut self, id: &str) -> bool {
        if !self.conquistas.contains(&id.to_string()) {
            self.conquistas.push(id.to_string());
            return true;
        }
        false
    }

    pub fn salvar_local(&self) -> bool {
        if self.estado_atual != "JOGO" && self.estado_atual != "COMBATE_ANIMATRONICO" {
            return false;
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            return fs::write(ARQUIVO_SAVE, json).is_ok();
        }
        false
    }

    pub fn carregar_local() -> Option<Self> {
        let conteudo = fs::read_to_string(ARQUIVO_SAVE)
            .or_else(|_| fs::read_to_string(ARQUIVO_SAVE_LEGADO))
            .ok()?;
        Self::from_json_str(&conteudo)
    }

    pub fn from_json_str(conteudo: &str) -> Option<Self> {
        let mut carregado: Self = serde_json::from_str(conteudo).ok()?;
        let dados = DadosJogo::carregar_tudo();
        if carregado.ascii_arts.is_empty() {
            carregado.ascii_arts = dados.ascii_arts;
        }
        if carregado.descricoes_itens.is_empty() {
            carregado.descricoes_itens = dados.descricoes_itens;
        }
        if carregado.eventos_aleatorios.is_empty() {
            carregado.eventos_aleatorios = dados.eventos_aleatorios;
        }
        if carregado.mapa.is_empty() {
            carregado.mapa = dados.mapa;
        }
        if carregado.dificuldade.is_empty() {
            carregado.dificuldade = carregado.dificuldade_escolhida.clone();
        }
        if carregado.dificuldade_escolhida.is_empty() {
            carregado.dificuldade_escolhida = carregado.dificuldade.clone();
        }
        Some(carregado)
    }

    pub fn save_existe() -> bool {
        std::path::Path::new(ARQUIVO_SAVE).exists()
            || std::path::Path::new(ARQUIVO_SAVE_LEGADO).exists()
    }

    /// Port 1:1 de `avancar_ia_perseguidor` em server/state.py
    pub fn avancar_ia_perseguidor(&mut self) -> RelatorioIA {
        let ia_congelada = (self.god_mode && !self.dev_ia) || self.dev_ia_congelada;
        if ia_congelada || self.estado_atual != "JOGO" {
            return RelatorioIA {
                ativa: false,
                distancia: 99,
                combate_iniciado: false,
                ai_estado: self.ai_estado.clone(),
                ai_sala: self.ai_sala.clone(),
            };
        }

        if self.nivel_barulho > 0 {
            self.alterar_barulho(-REDUCAO_BARULHO_TURNO);
        }

        if self.nivel_barulho >= LIMIAR_CACA || self.ai_sala == self.sala_atual {
            self.ai_estado = "CACA".to_string();
        } else if self.nivel_barulho >= LIMIAR_ALERTA && self.ai_estado != "CACA" {
            self.ai_estado = "ALERTA".to_string();
        } else if self.nivel_barulho < LIMIAR_ALERTA {
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
                    if let Some(dest) = v.as_deref() {
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
        } else if self.ai_estado == "ALERTA" || self.ai_estado == "CACA" {
            let destino = if self.ai_estado == "CACA" {
                self.sala_atual.clone()
            } else {
                self.ai_alvo
                    .clone()
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| self.sala_atual.clone())
            };
            if let Some(caminho) = calcular_caminho_bfs(&self.mapa, &self.ai_sala, &destino) {
                if caminho.len() > 1 {
                    self.ai_sala = caminho[1].clone();
                }
            }
        }

        let distancia = calcular_caminho_bfs(&self.mapa, &self.sala_atual, &self.ai_sala)
            .map(|c| c.len().saturating_sub(1))
            .unwrap_or(99);

        let mut combate = false;
        if distancia == 0 && self.sala_atual != "01" && self.sala_atual != "sala de energia" {
            self.estado_atual = "COMBATE_ANIMATRONICO".to_string();
            combate = true;
        }

        RelatorioIA {
            ativa: true,
            distancia,
            combate_iniciado: combate,
            ai_estado: self.ai_estado.clone(),
            ai_sala: self.ai_sala.clone(),
        }
    }
}

pub fn calcular_caminho_bfs(
    mapa: &HashMap<String, Sala>,
    inicio: &str,
    destino: &str,
) -> Option<Vec<String>> {
    if !mapa.contains_key(inicio) || !mapa.contains_key(destino) {
        return None;
    }
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
            let mut pares: Vec<(&String, &Option<String>)> = sala.conexoes.iter().collect();
            pares.sort_by(|a, b| a.0.cmp(b.0));
            for (k, v) in pares {
                if nao_saida.contains(&k.as_str()) {
                    continue;
                }
                if let Some(vizinho) = v.as_deref() {
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
