use crate::data::{obter_audio_embutido, resolver_arquivo_asset};
use rand::Rng;
use rodio::source::SineWave;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, Cursor};
use std::time::{Duration, Instant};

pub const ARQUIVO_CONFIG: &str = "config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistroPlacarLocal {
    pub nome: String,
    pub pontuacao: i32,
    pub dificuldade: String,
    pub tipo_final: String,
    pub tempo_segundos: u64,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Configuracoes {
    pub volume_geral: f32,
    pub volume_ambiente: f32,
    pub volume_efeitos: f32,
    pub volume_jumpscare: f32,
    pub mudo: bool,
    pub jumpscare_visual: bool,
    pub animacao_texto: bool,
    pub som_digitacao: bool,
    pub velocidade_digitacao: f32,
    pub efeito_crt: bool,
    pub legendas_audio: bool,
    pub conquistas_globais: Vec<String>,
    pub finais_desbloqueados: Vec<String>,
    pub placar_local: Vec<RegistroPlacarLocal>,
}

impl Default for Configuracoes {
    fn default() -> Self {
        Self {
            volume_geral: 1.0,
            volume_ambiente: 0.40,
            volume_efeitos: 0.80,
            volume_jumpscare: 1.0,
            mudo: false,
            jumpscare_visual: true,
            animacao_texto: true,
            som_digitacao: true,
            velocidade_digitacao: 1.0,
            efeito_crt: true,
            legendas_audio: true,
            conquistas_globais: Vec::new(),
            finais_desbloqueados: Vec::new(),
            placar_local: Vec::new(),
        }
    }
}

impl Configuracoes {
    pub fn carregar() -> Self {
        if let Ok(conteudo) = fs::read_to_string(ARQUIVO_CONFIG) {
            if let Ok(mut cfg) = serde_json::from_str::<Configuracoes>(&conteudo) {
                for f in ["mediocre", "bons_sonhos", "bom", "verdadeiro"] {
                    if cfg.conquistas_globais.contains(&f.to_string())
                        && !cfg.finais_desbloqueados.contains(&f.to_string())
                    {
                        cfg.finais_desbloqueados.push(f.to_string());
                    }
                }
                return cfg;
            }
        }
        Self::default()
    }

    pub fn salvar(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(ARQUIVO_CONFIG, json);
        }
    }

    pub fn registrar_final(&mut self, id_final: &str) -> bool {
        if !self.finais_desbloqueados.contains(&id_final.to_string()) {
            self.finais_desbloqueados.push(id_final.to_string());
        }
        if !self.conquistas_globais.contains(&id_final.to_string()) {
            self.conquistas_globais.push(id_final.to_string());
        }
        self.salvar();
        let todos = ["mediocre", "bons_sonhos", "bom", "verdadeiro"];
        todos
            .iter()
            .all(|f| self.finais_desbloqueados.iter().any(|c| c == f))
    }

    pub fn registrar_pontuacao(&mut self, registro: RegistroPlacarLocal) {
        self.placar_local.push(registro);
        self.placar_local
            .sort_by(|a, b| b.pontuacao.cmp(&a.pontuacao));
        if self.placar_local.len() > 25 {
            self.placar_local.truncate(25);
        }
        self.salvar();
    }
}

pub struct AudioManager {
    _stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    ambient_sink: Option<Sink>,
    pub config: Configuracoes,
    pub ultima_legenda: Option<(String, Instant)>,
    proximo_som_randomico: Instant,
    ultimo_som_tecla: Instant,
}

impl AudioManager {
    pub fn new() -> Self {
        let (stream, handle) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(_) => (None, None),
        };

        let mut rng = rand::thread_rng();
        let delay = rng.gen_range(35..=70);

        Self {
            _stream: stream,
            handle,
            ambient_sink: None,
            config: Configuracoes::carregar(),
            ultima_legenda: None,
            proximo_som_randomico: Instant::now() + Duration::from_secs(delay),
            ultimo_som_tecla: Instant::now(),
        }
    }

    fn calcular_volume(&self, canal: f32) -> f32 {
        if self.config.mudo {
            0.0
        } else {
            (self.config.volume_geral * canal).clamp(0.0, 2.0)
        }
    }

    pub fn definir_legenda(&mut self, texto: &str) {
        if self.config.legendas_audio {
            self.ultima_legenda = Some((texto.to_string(), Instant::now()));
        }
    }

    pub fn obter_legenda_ativa(&self) -> Option<&str> {
        if !self.config.legendas_audio {
            return None;
        }
        if let Some((ref txt, quando)) = self.ultima_legenda {
            if quando.elapsed() < Duration::from_secs(4) {
                return Some(txt.as_str());
            }
        }
        None
    }

    pub fn atualizar_volume_ambiente(&self) {
        if let Some(ref sink) = self.ambient_sink {
            sink.set_volume(self.calcular_volume(self.config.volume_ambiente));
        }
    }

    pub fn iniciar_ambiente(&mut self) {
        if self.ambient_sink.is_some() {
            self.atualizar_volume_ambiente();
            return;
        }
        let Some(ref handle) = self.handle else {
            return;
        };

        if let Some(bytes) = obter_audio_embutido("ambiente.mp3") {
            if let Ok(source) = Decoder::new(Cursor::new(bytes)) {
                if let Ok(sink) = Sink::try_new(handle) {
                    sink.set_volume(self.calcular_volume(self.config.volume_ambiente));
                    sink.append(source.repeat_infinite());
                    self.ambient_sink = Some(sink);
                    self.definir_legenda("[ Áudio: Zumbido elétrico constante do prédio ]");
                    return;
                }
            }
        }

        if let Some(caminho) = resolver_arquivo_asset("audio", "ambiente.mp3") {
            if let Ok(file) = File::open(caminho) {
                if let Ok(source) = Decoder::new(BufReader::new(file)) {
                    if let Ok(sink) = Sink::try_new(handle) {
                        sink.set_volume(self.calcular_volume(self.config.volume_ambiente));
                        sink.append(source.repeat_infinite());
                        self.ambient_sink = Some(sink);
                        self.definir_legenda("[ Áudio: Zumbido elétrico constante do prédio ]");
                    }
                }
            }
        }
    }

    pub fn parar_ambiente(&mut self) {
        if let Some(sink) = self.ambient_sink.take() {
            sink.stop();
        }
    }

    pub fn tocar_efeito(&self, nome_arquivo: &str, multiplicador_canal: f32) {
        let vol = self.calcular_volume(multiplicador_canal);
        if vol <= 0.0 {
            return;
        }
        let Some(ref handle) = self.handle else {
            return;
        };

        if let Some(bytes) = obter_audio_embutido(nome_arquivo) {
            if let Ok(source) = Decoder::new(Cursor::new(bytes)) {
                if let Ok(sink) = Sink::try_new(handle) {
                    sink.set_volume(vol);
                    sink.append(source);
                    sink.detach();
                    return;
                }
            }
        }

        if let Some(caminho) = resolver_arquivo_asset("audio", nome_arquivo) {
            if let Ok(file) = File::open(caminho) {
                if let Ok(source) = Decoder::new(BufReader::new(file)) {
                    if let Ok(sink) = Sink::try_new(handle) {
                        sink.set_volume(vol);
                        sink.append(source);
                        sink.detach();
                    }
                }
            }
        }
    }

    pub fn tocar_bipe(&self, freq: f32, duracao_ms: u64, amp: f32) {
        let vol = self.calcular_volume(self.config.volume_efeitos * amp);
        if vol <= 0.0 {
            return;
        }
        let Some(ref handle) = self.handle else {
            return;
        };
        if let Ok(sink) = Sink::try_new(handle) {
            let wave = SineWave::new(freq)
                .take_duration(Duration::from_millis(duracao_ms))
                .amplify(vol * 0.25);
            sink.append(wave);
            sink.detach();
        }
    }

    pub fn som_tecla(&mut self) {
        if !self.config.som_digitacao {
            return;
        }
        if self.ultimo_som_tecla.elapsed() < Duration::from_millis(28) {
            return;
        }
        self.ultimo_som_tecla = Instant::now();
        let mut rng = rand::thread_rng();
        let freq = rng.gen_range(400.0..=550.0);
        self.tocar_bipe(freq, 22, 0.35);
    }

    pub fn som_bipe_entrada(&self) {
        let vol = self.calcular_volume(self.config.volume_efeitos * 0.5);
        if vol <= 0.0 {
            return;
        }
        let Some(ref handle) = self.handle else {
            return;
        };
        if let Ok(sink) = Sink::try_new(handle) {
            let w1 = SineWave::new(550.0)
                .take_duration(Duration::from_millis(45))
                .amplify(vol * 0.25);
            let w2 = SineWave::new(1100.0)
                .take_duration(Duration::from_millis(65))
                .amplify(vol * 0.25);
            sink.append(w1);
            sink.append(w2);
            sink.detach();
        }
    }

    pub fn som_sucesso(&mut self) {
        self.tocar_bipe(800.0, 120, 0.65);
        self.definir_legenda("[ Áudio: Bipe positivo do sistema ]");
    }

    pub fn som_erro(&mut self) {
        self.tocar_bipe(150.0, 160, 0.75);
        self.definir_legenda("[ Áudio: Bipe grave de erro ]");
    }

    pub fn som_apito_tv(&mut self) {
        self.tocar_bipe(1000.0, 850, 0.45);
        self.definir_legenda("[ Áudio: Sinal de teste de TV 1000Hz ]");
    }

    pub fn som_chiado_tv(&mut self) {
        self.tocar_efeito("chiado.mp3", self.config.volume_efeitos * 0.65);
        self.definir_legenda("[ Áudio: Chiado de estática analógica ]");
    }

    pub fn som_porta(&mut self) {
        self.tocar_efeito("metalrangendo.mp3", self.config.volume_efeitos);
        self.definir_legenda("[ Áudio: Rangido pesado de metal e tranca ]");
    }

    pub fn som_metal(&mut self) {
        self.tocar_efeito("metalrangendo.mp3", self.config.volume_efeitos);
        self.definir_legenda("[ Áudio: Metal pesado rangendo no escuro ]");
    }

    pub fn som_randomico(&mut self) {
        let mut rng = rand::thread_rng();
        let arquivo = if rng.gen::<bool>() {
            "sonsrandomicos.mp3"
        } else {
            "sonsrandomicos2.mp3"
        };
        self.tocar_efeito(arquivo, self.config.volume_efeitos * 0.85);
        self.definir_legenda("[ Áudio: Ruídos estranhos ecoando pelo restaurante... ]");
    }

    pub fn som_grito(&mut self) {
        self.tocar_efeito("grito.mp3", self.config.volume_jumpscare);
        self.definir_legenda("[ Áudio: GRITO MECÂNICO ESTRIDENTE! ]");
    }

    pub fn atualizar_tick_ambiente(&mut self, em_jogo: bool) {
        if !em_jogo {
            return;
        }
        if Instant::now() >= self.proximo_som_randomico {
            self.som_randomico();
            let mut rng = rand::thread_rng();
            let prox = rng.gen_range(40..=85);
            self.proximo_som_randomico = Instant::now() + Duration::from_secs(prox);
        }
    }
}
