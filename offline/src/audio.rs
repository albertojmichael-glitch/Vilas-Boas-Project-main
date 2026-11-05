use crate::data::resolver_arquivo_asset;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::BufReader;

pub const ARQUIVO_CONFIG: &str = "config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuracoes {
    pub volume_geral: f32,
    pub volume_ambiente: f32,
    pub volume_efeitos: f32,
    pub volume_jumpscare: f32,
    pub mudo: bool,
    pub jumpscare_visual: bool,
    pub animacao_texto: bool,
    pub conquistas_globais: Vec<String>,
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
            conquistas_globais: Vec::new(),
        }
    }
}

impl Configuracoes {
    pub fn carregar() -> Self {
        if let Ok(conteudo) = fs::read_to_string(ARQUIVO_CONFIG) {
            if let Ok(cfg) = serde_json::from_str(&conteudo) {
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
}

pub struct AudioManager {
    _stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    ambient_sink: Option<Sink>,
    pub config: Configuracoes,
}

impl AudioManager {
    pub fn new() -> Self {
        let (stream, handle) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(_) => (None, None),
        };

        Self {
            _stream: stream,
            handle,
            ambient_sink: None,
            config: Configuracoes::carregar(),
        }
    }

    fn calcular_volume(&self, canal: f32) -> f32 {
        if self.config.mudo {
            0.0
        } else {
            (self.config.volume_geral * canal).clamp(0.0, 2.0)
        }
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
        let Some(ref handle) = self.handle else { return };
        let Some(caminho) = resolver_arquivo_asset("audio", "ambiente.mp3") else { return };

        if let Ok(file) = File::open(caminho) {
            if let Ok(source) = Decoder::new(BufReader::new(file)) {
                if let Ok(sink) = Sink::try_new(handle) {
                    sink.set_volume(self.calcular_volume(self.config.volume_ambiente));
                    sink.append(source.repeat_infinite());
                    self.ambient_sink = Some(sink);
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
        let Some(ref handle) = self.handle else { return };
        let Some(caminho) = resolver_arquivo_asset("audio", nome_arquivo) else { return };

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

    pub fn som_porta(&self) {
        self.tocar_efeito("portafechandoabrindo.mp3", self.config.volume_efeitos);
    }

    pub fn som_metal(&self) {
        self.tocar_efeito("metalrangendo.mp3", self.config.volume_efeitos);
    }

    pub fn som_randomico(&self) {
        self.tocar_efeito("sonsrandomicos.mp3", self.config.volume_efeitos * 0.85);
    }

    pub fn som_grito(&self) {
        self.tocar_efeito("grito.mp3", self.config.volume_jumpscare);
    }
}