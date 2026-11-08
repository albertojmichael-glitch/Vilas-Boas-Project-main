#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

mod actions;
mod audio;
mod data;
mod devmode;
mod engine;
mod media;
mod minigames;
mod parser;
mod state;
mod ui;
mod utils;
mod views;

use audio::{AudioManager, RegistroPlacarLocal};
use data::IMAGENS_SECRETAS_EMBUTIDAS;
use eframe::egui::{
    self, text::LayoutJob, Align, Color32, FontFamily, FontId, Key, Layout, Order, Pos2, Rect,
    RichText, Rounding, Sense, Stroke, TextFormat, Vec2,
};
use engine::processar_fluxo_jogo;
use media::MediaManager;
use rand::seq::SliceRandom;
use rand::Rng;
use state::GameState;
use std::collections::VecDeque;
use std::fs;
use std::time::{Duration, Instant};
use ui::{COR_AMARELO, COR_BRANCO, COR_CIANO, COR_CINZA, COR_VERDE, COR_VERMELHO};
use utils::CONQUISTAS_DB;
use views::imprimir_tela_boot;

#[derive(Clone)]
pub struct SegmentoTexto {
    pub texto: String,
    pub cor: Color32,
}

#[derive(Clone)]
pub struct LinhaTerminal {
    pub segmentos: Vec<SegmentoTexto>,
}

impl LinhaTerminal {
    pub fn total_chars(&self) -> usize {
        self.segmentos.iter().map(|s| s.texto.chars().count()).sum()
    }
}

#[derive(Clone)]
pub enum ItemFilaTerminal {
    Limpar,
    Linha {
        linha: LinhaTerminal,
        animada: bool,
    },
    Pausa(f32),
    Efeito(String),
}

#[derive(Default)]
pub struct TerminalBuffer {
    pub fila: VecDeque<ItemFilaTerminal>,
}

impl TerminalBuffer {
    pub fn limpar(&mut self) {
        self.fila.push_back(ItemFilaTerminal::Limpar);
    }

    pub fn clear(&mut self) {
        self.limpar();
    }

    pub fn exibir(&mut self, msg: &str) {
        self.enfileirar_ansi(msg, COR_BRANCO, false);
    }

    pub fn animar(&mut self, msg: &str, cor_padrao_ansi: &str) {
        let padrao = Self::cor_de_codigo(cor_padrao_ansi, COR_BRANCO);
        self.enfileirar_ansi(msg, padrao, true);
    }

    pub fn pausa(&mut self, segundos: f32) {
        self.fila.push_back(ItemFilaTerminal::Pausa(segundos));
    }

    pub fn push(&mut self, texto: &str, cor_padrao: Color32) {
        self.enfileirar_ansi(texto, cor_padrao, false);
    }

    pub fn buffer_efeito(&mut self, marcador: &str) {
        self.fila
            .push_back(ItemFilaTerminal::Efeito(marcador.to_string()));
    }

    fn cor_de_codigo(cod: &str, atual: Color32) -> Color32 {
        match cod {
            "\x1b[91m" => COR_VERMELHO,
            "\x1b[92m" => COR_VERDE,
            "\x1b[93m" => COR_AMARELO,
            "\x1b[96m" => COR_CIANO,
            "\x1b[90m" => COR_CINZA,
            "\x1b[97m" | "\x1b[0m" => COR_BRANCO,
            _ => atual,
        }
    }

    pub fn enfileirar_ansi(&mut self, texto: &str, cor_inicial: Color32, animada: bool) {
        let limpo = texto.replace("@@TYPE@@", "").replace("@@CLEAR@@", "");
        for raw_line in limpo.split('\n') {
            let mut segmentos = Vec::new();
            let mut cor_atual = cor_inicial;
            let mut resto = raw_line;

            while let Some(pos) = resto.find("\x1b[") {
                if pos > 0 {
                    segmentos.push(SegmentoTexto {
                        texto: resto[..pos].to_string(),
                        cor: cor_atual,
                    });
                }
                let apos_esc = &resto[pos..];
                if let Some(fim_m) = apos_esc.find('m') {
                    let codigo = &apos_esc[..=fim_m];
                    cor_atual = if codigo == "\x1b[0m" {
                        cor_inicial
                    } else {
                        Self::cor_de_codigo(codigo, cor_atual)
                    };
                    resto = &apos_esc[fim_m + 1..];
                } else {
                    break;
                }
            }

            if !resto.is_empty() || segmentos.is_empty() {
                segmentos.push(SegmentoTexto {
                    texto: resto.to_string(),
                    cor: cor_atual,
                });
            }

            self.fila.push_back(ItemFilaTerminal::Linha {
                linha: LinhaTerminal { segmentos },
                animada,
            });
        }
    }
}

#[derive(Clone, PartialEq)]
enum EstadoIntroTv {
    Desligada,
    Estatica(Instant),
    BarrasCor(Instant),
    Video2(Instant),
    Concluida,
}

struct LinhaEmDigitacao {
    linha: LinhaTerminal,
    chars_visiveis: usize,
    total_chars: usize,
    ultimo_tick: Instant,
}

struct VilasBoasApp {
    jogo: GameState,
    term: TerminalBuffer,
    linhas_visiveis: Vec<LinhaTerminal>,
    linha_digitando: Option<LinhaEmDigitacao>,
    pausa_ate: Option<Instant>,
    entrada: String,
    historico_comandos: Vec<String>,
    indice_historico: Option<usize>,
    audio: AudioManager,
    media: MediaManager,
    intro_tv: EstadoIntroTv,
    // Modais
    modal_conquistas: bool,
    modal_config: bool,
    modal_ajuda: bool,
    modal_saves: bool,
    modal_placar: bool,
    // Estado do modal de saves
    json_save_buffer: String,
    feedback_save: Option<(String, Color32, Instant)>,
    // Estado do modal de fim de jogo / placar
    nome_jogador_placar: String,
    pontuacao_pendente: Option<(i32, String, String, u64)>,
    feedback_placar: Option<String>,
    // Overlays e Efeitos Visuais
    tela_cheia: bool,
    modo_energia: String,
    jumpscare_overlay_ate: Option<(Instant, String)>,
    bsod_overlay_ate: Option<(Instant, String)>,
    panic_ativo: bool,
    flash_secrets_ativo: bool,
    slideshow_1980: Option<(usize, Instant)>,
    conquistas_conhecidas: usize,
    toast_conquista: Option<(String, String, Instant)>,
}

impl VilasBoasApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::BLACK;
        visuals.window_fill = Color32::from_rgb(8, 12, 8);
        visuals.extreme_bg_color = Color32::BLACK;
        cc.egui_ctx.set_visuals(visuals);

        let audio = AudioManager::new();
        let mut jogo = GameState::new();
        jogo.conquistas = audio.config.conquistas_globais.clone();
        let qtd_conquistas = jogo.conquistas.len();

        let mut term = TerminalBuffer::default();
        imprimir_tela_boot(&mut term);

        Self {
            jogo,
            term,
            linhas_visiveis: Vec::new(),
            linha_digitando: None,
            pausa_ate: None,
            entrada: String::new(),
            historico_comandos: Vec::new(),
            indice_historico: None,
            audio,
            media: MediaManager::new(),
            intro_tv: EstadoIntroTv::Desligada,
            modal_conquistas: false,
            modal_config: false,
            modal_ajuda: false,
            modal_saves: false,
            modal_placar: false,
            json_save_buffer: String::new(),
            feedback_save: None,
            nome_jogador_placar: "ROGERIO".to_string(),
            pontuacao_pendente: None,
            feedback_placar: None,
            tela_cheia: false,
            modo_energia: "NORMAL".to_string(),
            jumpscare_overlay_ate: None,
            bsod_overlay_ate: None,
            panic_ativo: false,
            flash_secrets_ativo: false,
            slideshow_1980: None,
            conquistas_conhecidas: qtd_conquistas,
            toast_conquista: None,
        }
    }

    fn aplicar_efeito(&mut self, marcador: &str, ctx: &egui::Context) {
        let mut rng = rand::thread_rng();
        match marcador {
            "@@JUMPSCARE@@" => {
                self.media.flash_ticks = 18;
                let img = if rng.gen::<bool>() {
                    "JUMPSCARE1.jpg"
                } else {
                    "JUMPSCARE2.jpg"
                };
                self.jumpscare_overlay_ate =
                    Some((Instant::now() + Duration::from_millis(1250), img.to_string()));
            }
            "@@BSOD@@" => {
                let opcoes = ["JAPONESMEDO.jpg", "SHEISHERE1.jpg", "SHEISHERE2.jpg"];
                let img = *opcoes.choose(&mut rng).unwrap_or(&"SHEISHERE1.jpg");
                self.bsod_overlay_ate =
                    Some((Instant::now() + Duration::from_millis(1450), img.to_string()));
            }
            "@@PANIC@@" => {
                self.panic_ativo = true;
            }
            "@@START_FLASH_SECRETS@@" => {
                self.flash_secrets_ativo = true;
            }
            "@@STOP_FLASH_SECRETS@@" => {
                self.flash_secrets_ativo = false;
            }
            "@@PLAY_1980_SLIDESHOW@@" => {
                self.audio.som_chiado_tv();
                self.slideshow_1980 = Some((0, Instant::now()));
            }
            "@@GLITCH_LUZ@@" => {
                self.media.flash_ticks = 6;
            }
            "@@LOW_POWER@@" => {
                self.modo_energia = "LOW".to_string();
            }
            "@@BLACKOUT@@" => {
                self.modo_energia = "BLACKOUT".to_string();
            }
            "@@NORMAL_POWER@@" => {
                self.modo_energia = "NORMAL".to_string();
            }
            "@@EXIT@@" => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            "@@RELOAD@@" => {
                self.reiniciar_sistema();
            }
            _ => {}
        }
    }

    fn reiniciar_sistema(&mut self) {
        self.media.imagem_atual = None;
        self.modo_energia = "NORMAL".to_string();
        self.panic_ativo = false;
        self.flash_secrets_ativo = false;
        self.jumpscare_overlay_ate = None;
        self.bsod_overlay_ate = None;
        self.slideshow_1980 = None;
        self.linhas_visiveis.clear();
        self.linha_digitando = None;
        self.term.fila.clear();
        imprimir_tela_boot(&mut self.term);
        self.jogo.estado_atual = "AGUARDANDO_DIR".to_string();
    }

    fn descarregar_fila_instantaneamente(&mut self, ctx: &egui::Context) {
        if let Some(atual) = self.linha_digitando.take() {
            self.linhas_visiveis.push(atual.linha);
        }
        self.pausa_ate = None;
        while let Some(item) = self.term.fila.pop_front() {
            match item {
                ItemFilaTerminal::Limpar => self.linhas_visiveis.clear(),
                ItemFilaTerminal::Linha { linha, .. } => self.linhas_visiveis.push(linha),
                ItemFilaTerminal::Pausa(_) => {}
                ItemFilaTerminal::Efeito(ef) => self.aplicar_efeito(&ef, ctx),
            }
        }
    }

    fn atualizar_animacao_terminal(&mut self, ctx: &egui::Context) {
        let vel_config = if !self.audio.config.animacao_texto {
            0.0
        } else {
            self.audio.config.velocidade_digitacao
        };
        let vel_jogo = self.jogo.velocidade_texto;
        let instantaneo = vel_config <= 0.0 || vel_jogo <= 0.0;

        if instantaneo {
            self.descarregar_fila_instantaneamente(ctx);
            return;
        }

        if let Some(ate) = self.pausa_ate {
            if Instant::now() < ate {
                ctx.request_repaint_after(Duration::from_millis(30));
                return;
            }
            self.pausa_ate = None;
        }

        if let Some(ref mut atual) = self.linha_digitando {
            let intervalo_ms = (14.0 / vel_config.max(0.25)) as u64;
            let decorrido = atual.ultimo_tick.elapsed().as_millis() as u64;
            if decorrido >= intervalo_ms {
                let avanco = ((decorrido / intervalo_ms.max(1)) as usize).clamp(1, 6);
                atual.chars_visiveis = (atual.chars_visiveis + avanco).min(atual.total_chars);
                atual.ultimo_tick = Instant::now();
                self.audio.som_tecla();
            }

            if atual.chars_visiveis >= atual.total_chars {
                let concluida = self.linha_digitando.take().unwrap();
                self.linhas_visiveis.push(concluida.linha);
            } else {
                ctx.request_repaint_after(Duration::from_millis(16));
                return;
            }
        }

        while let Some(item) = self.term.fila.pop_front() {
            match item {
                ItemFilaTerminal::Limpar => {
                    self.linhas_visiveis.clear();
                }
                ItemFilaTerminal::Efeito(ef) => {
                    self.aplicar_efeito(&ef, ctx);
                }
                ItemFilaTerminal::Pausa(segs) => {
                    let dur = Duration::from_secs_f32((segs / vel_config.max(0.5)).min(3.0));
                    self.pausa_ate = Some(Instant::now() + dur);
                    ctx.request_repaint_after(Duration::from_millis(30));
                    break;
                }
                ItemFilaTerminal::Linha { linha, animada } => {
                    let total = linha.total_chars();
                    if !animada || total == 0 {
                        self.linhas_visiveis.push(linha);
                    } else {
                        self.linha_digitando = Some(LinhaEmDigitacao {
                            linha,
                            chars_visiveis: 1,
                            total_chars: total,
                            ultimo_tick: Instant::now(),
                        });
                        ctx.request_repaint_after(Duration::from_millis(16));
                        break;
                    }
                }
            }
        }
    }

    fn verificar_novas_conquistas_e_finais(&mut self) {
        if self.jogo.conquistas.len() > self.conquistas_conhecidas {
            if let Some(id_novo) = self.jogo.conquistas.last().cloned() {
                if let Some(info) = CONQUISTAS_DB
                    .iter()
                    .find(|c| c.id == id_novo || c.chave_db == id_novo)
                {
                    self.audio.som_sucesso();
                    self.toast_conquista = Some((
                        format!("{} {}", info.icone, info.nome),
                        info.desc.to_string(),
                        Instant::now(),
                    ));
                }
            }
            self.conquistas_conhecidas = self.jogo.conquistas.len();
        }
    }

    fn calcular_pontuacao_final(&self, tipo_final: &str) -> i32 {
        if self.jogo.god_mode || self.jogo.dev_usado {
            return 0;
        }
        let base = match tipo_final {
            "verdadeiro" => 5000,
            "bom" => 3000,
            "bons_sonhos" => 1500,
            _ => 1000,
        };
        let mult = match self.jogo.dificuldade.as_str() {
            "PESADELO" => 2.0,
            d if d.starts_with("DESAFIO_") => 1.5,
            "NORMAL" => 1.0,
            _ => 0.5,
        };
        let tempo = self.jogo.tempo_decorrido_segundos() as i32;
        let bonus_tempo = (900 - tempo).max(0) * 2;
        let bonus_hp = (self.jogo.hp.max(0)) * 250;
        ((base + bonus_tempo + bonus_hp) as f32 * mult) as i32
    }

    fn executar_comando_direto(&mut self, cmd: &str, ctx: &egui::Context) {
        self.descarregar_fila_instantaneamente(ctx);
        let cmd_trim = cmd.trim().to_string();
        if !cmd_trim.is_empty() {
            self.audio.som_bipe_entrada();
            self.term.push(&format!("\nC:\\> {}", cmd_trim), COR_VERDE);
            if self.historico_comandos.last() != Some(&cmd_trim) {
                self.historico_comandos.push(cmd_trim.clone());
            }
            self.indice_historico = None;
        }

        let finais_antes = self.audio.config.finais_desbloqueados.len();
        let estado_antes = self.jogo.estado_atual.clone();

        if estado_antes == "FIM" {
            self.panic_ativo = false;
            self.pontuacao_pendente = None;
        }

        processar_fluxo_jogo(
            &cmd_trim,
            &mut self.jogo,
            &mut self.term,
            &mut self.audio,
            &mut self.media,
        );

        self.verificar_novas_conquistas_e_finais();

        // Se acabou de terminar um final vitorioso (não morte/panic), prepara pontuação para o Placar Local
        if self.jogo.estado_atual == "FIM"
            && estado_antes != "FIM"
            && self.jogo.sala_atual != "morte"
            && !self.panic_ativo
        {
            let tipo_final = if self.jogo.fios_cortados_inventario
                && (self.jogo.sala_atual == "hall" || self.jogo.sala_atual == "hall de entrada")
            {
                "verdadeiro"
            } else if self.jogo.sala_atual == "hall" || self.jogo.sala_atual == "hall de entrada" {
                "bom"
            } else if self.jogo.sala_atual == "cama" {
                "bons_sonhos"
            } else {
                "mediocre"
            };
            let pts = self.calcular_pontuacao_final(tipo_final);
            let tempo = self.jogo.tempo_decorrido_segundos();
            self.pontuacao_pendente = Some((
                pts,
                self.jogo.dificuldade.clone(),
                tipo_final.to_string(),
                tempo,
            ));
            let _ = finais_antes;
        }
    }

    fn submeter_comando(&mut self, ctx: &egui::Context) {
        // Se ainda estiver digitando linhas animadas e o input estiver vazio, apenas avança a animação
        if (self.linha_digitando.is_some() || !self.term.fila.is_empty())
            && self.entrada.trim().is_empty()
            && self.jogo.estado_atual != "FIM"
        {
            self.descarregar_fila_instantaneamente(ctx);
            return;
        }

        let cmd = self.entrada.trim().to_string();
        self.entrada.clear();
        self.executar_comando_direto(&cmd, ctx);
    }

    fn desenhar_intro_tv(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::BLACK))
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                let agora = Instant::now();

                // Verifica transições temporizadas da TV
                match self.intro_tv.clone() {
                    EstadoIntroTv::Estatica(inicio) => {
                        ctx.request_repaint_after(Duration::from_millis(40));
                        if agora.duration_since(inicio) >= Duration::from_millis(1700) {
                            self.audio.som_apito_tv();
                            self.intro_tv = EstadoIntroTv::BarrasCor(agora);
                        }
                    }
                    EstadoIntroTv::BarrasCor(inicio) => {
                        ctx.request_repaint_after(Duration::from_millis(40));
                        if agora.duration_since(inicio) >= Duration::from_millis(1600) {
                            self.intro_tv = EstadoIntroTv::Video2(agora);
                        }
                    }
                    EstadoIntroTv::Video2(inicio) => {
                        ctx.request_repaint_after(Duration::from_millis(40));
                        if agora.duration_since(inicio) >= Duration::from_millis(1500) {
                            self.audio.som_bipe_entrada();
                            self.intro_tv = EstadoIntroTv::Concluida;
                        }
                    }
                    _ => {}
                }

                // Atalho para pular ou ligar com Enter/Espaço
                if ctx.input(|i| i.key_pressed(Key::Escape)) {
                    self.intro_tv = EstadoIntroTv::Concluida;
                    return;
                }
                if ctx.input(|i| i.key_pressed(Key::Enter) || i.key_pressed(Key::Space)) {
                    if self.intro_tv == EstadoIntroTv::Desligada {
                        self.audio.som_chiado_tv();
                        self.intro_tv = EstadoIntroTv::Estatica(Instant::now());
                    } else {
                        self.intro_tv = EstadoIntroTv::Concluida;
                        return;
                    }
                }

                let tv_w = (rect.width() * 0.78).min(860.0);
                let tv_h = (rect.height() * 0.78).min(640.0);
                let tv_rect = Rect::from_center_size(rect.center(), Vec2::new(tv_w, tv_h));

                // Área da tela interna do tubo da TV
                let screen_rect = Rect::from_min_max(
                    Pos2::new(tv_rect.left() + tv_w * 0.11, tv_rect.top() + tv_h * 0.11),
                    Pos2::new(tv_rect.right() - tv_w * 0.22, tv_rect.bottom() - tv_h * 0.13),
                );

                match &self.intro_tv {
                    EstadoIntroTv::Desligada => {
                        ui.painter()
                            .rect_filled(screen_rect, 8.0, Color32::from_rgb(8, 10, 8));
                    }
                    EstadoIntroTv::Estatica(_) => {
                        if let Some(tex) = self.media.obter_textura(ctx, "estatica.png") {
                            ui.painter().image(
                                tex.id(),
                                screen_rect,
                                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                Color32::WHITE,
                            );
                        } else {
                            ui.painter()
                                .rect_filled(screen_rect, 8.0, Color32::from_rgb(90, 90, 90));
                        }
                    }
                    EstadoIntroTv::BarrasCor(_) => {
                        if let Some(tex) = self.media.obter_textura(ctx, "tvcolorida.png") {
                            ui.painter().image(
                                tex.id(),
                                screen_rect,
                                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                Color32::WHITE,
                            );
                        }
                    }
                    EstadoIntroTv::Video2(_) => {
                        ui.painter()
                            .rect_filled(screen_rect, 8.0, Color32::from_rgb(0, 0, 165));
                        ui.painter().text(
                            Pos2::new(screen_rect.left() + 28.0, screen_rect.top() + 28.0),
                            egui::Align2::LEFT_TOP,
                            "VIDEO 2",
                            FontId::new(32.0, FontFamily::Monospace),
                            COR_VERDE,
                        );
                        ui.painter().text(
                            screen_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "SINAL AUXILIAR DETECTADO\nVILLAS-BOAS ENTERTAINMENT (1982)",
                            FontId::new(16.0, FontFamily::Monospace),
                            COR_BRANCO,
                        );
                    }
                    EstadoIntroTv::Concluida => {}
                }

                // Desenha a moldura da TV (`tv.png`) por cima
                if let Some(tex_tv) = self.media.obter_textura(ctx, "tv.png") {
                    ui.painter().image(
                        tex_tv.id(),
                        tv_rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                } else {
                    ui.painter()
                        .rect_stroke(tv_rect, 12.0, Stroke::new(4.0_f32, COR_CINZA));
                }

                // LED de energia e botões interativos na base/lateral da TV
                let ligada = self.intro_tv != EstadoIntroTv::Desligada;
                let cor_led = if ligada {
                    COR_VERDE
                } else {
                    Color32::from_rgb(180, 20, 20)
                };

                let controles_rect = Rect::from_center_size(
                    Pos2::new(rect.center().x, tv_rect.bottom() + 28.0),
                    Vec2::new(560.0, 44.0),
                );
                ui.allocate_ui_at_rect(controles_rect, |ui| {
                    ui.horizontal_centered(|ui| {
                        let (led_rect, _) =
                            ui.allocate_exact_size(Vec2::new(14.0, 14.0), Sense::hover());
                        ui.painter()
                            .circle_filled(led_rect.center(), 6.0, cor_led);
                        ui.add_space(8.0);

                        if !ligada {
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("⏻ LIGAR TV (ENTER)")
                                            .font(FontId::new(16.0, FontFamily::Monospace))
                                            .color(COR_VERDE)
                                            .strong(),
                                    )
                                    .fill(Color32::from_rgb(0, 35, 0))
                                    .stroke(Stroke::new(1.5_f32, COR_VERDE)),
                                )
                                .clicked()
                            {
                                self.audio.som_chiado_tv();
                                self.intro_tv = EstadoIntroTv::Estatica(Instant::now());
                            }
                        } else {
                            ui.label(
                                RichText::new("SINTONIZANDO CANAL MS-DOS...")
                                    .font(FontId::new(15.0, FontFamily::Monospace))
                                    .color(COR_AMARELO),
                            );
                        }

                        ui.add_space(18.0);
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("[Pular Intro >>]")
                                        .font(FontId::new(13.0, FontFamily::Monospace))
                                        .color(COR_CINZA),
                                )
                                .fill(Color32::BLACK)
                                .stroke(Stroke::new(1.0_f32, COR_CINZA)),
                            )
                            .clicked()
                        {
                            self.intro_tv = EstadoIntroTv::Concluida;
                        }
                    });
                });
            });
    }

    fn desenhar_overlays_terror(&mut self, ctx: &egui::Context) {
        let agora = Instant::now();

        // 1. Overlay Jumpscare em tela cheia
        if let Some((ate, nome_img)) = self.jumpscare_overlay_ate.clone() {
            if agora < ate && self.audio.config.jumpscare_visual {
                ctx.request_repaint_after(Duration::from_millis(25));
                egui::Area::new(egui::Id::new("overlay_jumpscare_fullscreen"))
                    .order(Order::Foreground)
                    .fixed_pos(Pos2::ZERO)
                    .show(ctx, |ui| {
                        let screen = ctx.screen_rect();
                        let pisca = (agora.elapsed().subsec_millis() / 45) % 2 == 0;
                        ui.painter().rect_filled(
                            screen,
                            0.0,
                            if pisca {
                                Color32::from_rgb(160, 0, 0)
                            } else {
                                Color32::BLACK
                            },
                        );
                        if let Some(tex) = self.media.obter_textura(ctx, &nome_img) {
                            ui.painter().image(
                                tex.id(),
                                screen,
                                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                Color32::WHITE,
                            );
                        }
                    });
            } else {
                self.jumpscare_overlay_ate = None;
            }
        }

        // 2. Overlay BSOD (Tela Azul do VILAS-BOAS OS antes do CAPTCHA)
        if let Some((ate, nome_img)) = self.bsod_overlay_ate.clone() {
            if agora < ate {
                ctx.request_repaint_after(Duration::from_millis(40));
                egui::Area::new(egui::Id::new("overlay_bsod_fullscreen"))
                    .order(Order::Foreground)
                    .fixed_pos(Pos2::ZERO)
                    .show(ctx, |ui| {
                        let screen = ctx.screen_rect();
                        ui.painter()
                            .rect_filled(screen, 0.0, Color32::from_rgb(0, 0, 170));
                        let msg = "VILAS-BOAS OS\n\n\
                            Ocorreu uma exceção fatal 0E em 0028:C0011E36 no VXD VMM(01).\n\
                            O módulo de segurança secundário parou de responder.\n\n\
                            * Iniciando protocolo de calibração manual (CAPTCHA)...\n\
                            * NÃO DESLIGUE O TERMINAL.";
                        ui.painter().text(
                            Pos2::new(screen.left() + 80.0, screen.top() + 120.0),
                            egui::Align2::LEFT_TOP,
                            msg,
                            FontId::new(20.0, FontFamily::Monospace),
                            Color32::WHITE,
                        );
                        // Nos últimos 450ms pisca a imagem subliminar de terror
                        if ate.duration_since(agora) < Duration::from_millis(450)
                            && self.audio.config.jumpscare_visual
                        {
                            if let Some(tex) = self.media.obter_textura(ctx, &nome_img) {
                                let r = Rect::from_center_size(
                                    screen.center(),
                                    Vec2::new(420.0, 420.0),
                                );
                                ui.painter().image(
                                    tex.id(),
                                    r,
                                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                    Color32::from_rgba_unmultiplied(255, 255, 255, 210),
                                );
                            }
                        }
                    });
            } else {
                self.bsod_overlay_ate = None;
            }
        }

        // 3. Flash de Imagens Secretas (Disquete 3)
        if self.flash_secrets_ativo && self.linha_digitando.is_some() {
            ctx.request_repaint_after(Duration::from_millis(80));
            let mut rng = rand::thread_rng();
            if rng.gen::<f64>() < 0.35 && self.audio.config.jumpscare_visual {
                if let Some((nome, _)) = IMAGENS_SECRETAS_EMBUTIDAS.choose(&mut rng) {
                    if let Some(tex) = self.media.obter_textura(ctx, nome) {
                        egui::Area::new(egui::Id::new("overlay_flash_secrets"))
                            .order(Order::Foreground)
                            .fixed_pos(Pos2::ZERO)
                            .show(ctx, |ui| {
                                let screen = ctx.screen_rect();
                                let r = Rect::from_center_size(
                                    screen.center(),
                                    Vec2::new(360.0, 360.0),
                                );
                                ui.painter().image(
                                    tex.id(),
                                    r,
                                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                    Color32::from_rgba_unmultiplied(255, 180, 180, 170),
                                );
                            });
                    }
                }
            }
        }

        // 4. Slideshow Secreto de 1980
        if let Some((mut idx, mut ultimo)) = self.slideshow_1980 {
            ctx.request_repaint_after(Duration::from_millis(50));
            if agora.duration_since(ultimo) >= Duration::from_millis(1200) {
                idx += 1;
                ultimo = agora;
                self.audio.som_tecla();
            }
            if idx >= IMAGENS_SECRETAS_EMBUTIDAS.len()
                || ctx.input(|i| i.key_pressed(Key::Escape) || i.key_pressed(Key::Enter))
            {
                self.slideshow_1980 = None;
            } else {
                self.slideshow_1980 = Some((idx, ultimo));
                let nome_img = IMAGENS_SECRETAS_EMBUTIDAS[idx].0;
                egui::Area::new(egui::Id::new("overlay_slideshow_1980"))
                    .order(Order::Foreground)
                    .fixed_pos(Pos2::ZERO)
                    .show(ctx, |ui| {
                        let screen = ctx.screen_rect();
                        ui.painter().rect_filled(screen, 0.0, Color32::BLACK);
                        if let Some(tex) = self.media.obter_textura(ctx, nome_img) {
                            let img_rect = Rect::from_center_size(
                                screen.center(),
                                Vec2::new(screen.width() * 0.7, screen.height() * 0.72),
                            );
                            ui.painter().image(
                                tex.id(),
                                img_rect,
                                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                Color32::WHITE,
                            );
                        }
                        ui.painter().text(
                            Pos2::new(screen.center().x, screen.top() + 34.0),
                            egui::Align2::CENTER_CENTER,
                            format!(
                                "[ ARQUIVO DE MEMÓRIA 1980 — REGISTRO {:02}/{} ] (ESC/ENTER para fechar)",
                                idx + 1,
                                IMAGENS_SECRETAS_EMBUTIDAS.len()
                            ),
                            FontId::new(16.0, FontFamily::Monospace),
                            COR_VERMELHO,
                        );
                    });
            }
        }

        // 5. Efeito CRT Scanlines sutis sobre toda a tela
        if self.audio.config.efeito_crt {
            let screen = ctx.screen_rect();
            let painter = ctx.layer_painter(egui::LayerId::new(
                Order::Foreground,
                egui::Id::new("crt_scanlines_layer"),
            ));
            let mut y = screen.top();
            let cor_scan = Color32::from_rgba_unmultiplied(0, 0, 0, 42);
            while y < screen.bottom() {
                painter.line_segment(
                    [Pos2::new(screen.left(), y), Pos2::new(screen.right(), y)],
                    Stroke::new(1.0_f32, cor_scan),
                );
                y += 3.0;
            }
        }
    }
}

impl eframe::App for VilasBoasApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Se estiver na sequência de introdução da TV retrô, desenha a TV
        if self.intro_tv != EstadoIntroTv::Concluida {
            self.desenhar_intro_tv(ctx);
            if self.audio.config.efeito_crt {
                self.desenhar_overlays_terror(ctx);
            }
            return;
        }

        let mono = FontId::new(15.5, FontFamily::Monospace);
        let mono_hud = FontId::new(14.5, FontFamily::Monospace);

        let em_jogo = self.jogo.estado_atual == "JOGO"
            || self.jogo.estado_atual == "COMBATE_ANIMATRONICO"
            || self.jogo.estado_atual.starts_with("MINIGAME_");

        self.audio.atualizar_tick_ambiente(em_jogo);
        self.atualizar_animacao_terminal(ctx);

        if self.media.flash_ticks > 0 {
            self.media.flash_ticks -= 1;
            ctx.request_repaint_after(Duration::from_millis(35));
        }

        // Atalhos de teclado globais
        let enter_pressionado = ctx.input(|i| i.key_pressed(Key::Enter));
        let f5_pressionado = ctx.input(|i| i.key_pressed(Key::F5));
        let f11_pressionado = ctx.input(|i| i.key_pressed(Key::F11));
        let seta_cima = ctx.input(|i| i.key_pressed(Key::ArrowUp));
        let seta_baixo = ctx.input(|i| i.key_pressed(Key::ArrowDown));

        if f11_pressionado {
            self.tela_cheia = !self.tela_cheia;
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.tela_cheia));
        }

        if f5_pressionado {
            self.reiniciar_sistema();
        }

        // Navegação no histórico de comandos com ArrowUp / ArrowDown
        if seta_cima && !self.historico_comandos.is_empty() {
            let novo_idx = match self.indice_historico {
                None => self.historico_comandos.len().saturating_sub(1),
                Some(i) => i.saturating_sub(1),
            };
            self.indice_historico = Some(novo_idx);
            self.entrada = self.historico_comandos[novo_idx].clone();
        } else if seta_baixo && !self.historico_comandos.is_empty() {
            if let Some(i) = self.indice_historico {
                if i + 1 < self.historico_comandos.len() {
                    self.indice_historico = Some(i + 1);
                    self.entrada = self.historico_comandos[i + 1].clone();
                } else {
                    self.indice_historico = None;
                    self.entrada.clear();
                }
            }
        }

        let cor_borda_painel = match self.modo_energia.as_str() {
            "LOW" => COR_AMARELO,
            "BLACKOUT" => Color32::from_rgb(45, 45, 45),
            _ => {
                if self.jogo.hp == 1 && !self.jogo.god_mode && em_jogo {
                    COR_VERMELHO
                } else {
                    COR_VERDE
                }
            }
        };

        let mut comando_atalho_clicado: Option<String> = None;

        // =====================================================================
        // PAINEL LATERAL DIREITO - [ SISTEMA v20.08 ]
        // =====================================================================
        egui::SidePanel::right("hud_sistema_v2008")
            .exact_width(300.0)
            .resizable(false)
            .frame(
                egui::Frame::none()
                    .fill(Color32::BLACK)
                    .inner_margin(egui::Margin::symmetric(14.0, 14.0))
                    .stroke(Stroke::new(1.5_f32, cor_borda_painel)),
            )
            .show(ctx, |ui| {
                // Rodapé fixo com os 6 botões de ferramentas do sistema
                egui::TopBottomPanel::bottom("hud_botoes_rodape")
                    .frame(egui::Frame::none().inner_margin(egui::Margin::symmetric(0.0, 4.0)))
                    .show_separator_line(false)
                    .show_inside(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 6.0;
                            let btn_size = Vec2::new(40.0, 42.0);
                            let icon_font = FontId::new(19.0, FontFamily::Proportional);

                            if ui
                                .add_sized(
                                    btn_size,
                                    egui::Button::new(
                                        RichText::new("👑")
                                            .font(icon_font.clone())
                                            .color(COR_AMARELO),
                                    )
                                    .fill(Color32::BLACK)
                                    .rounding(Rounding::same(4.0))
                                    .stroke(Stroke::new(1.5_f32, COR_AMARELO)),
                                )
                                .on_hover_text("Conquistas Desbloqueadas")
                                .clicked()
                            {
                                self.modal_conquistas = !self.modal_conquistas;
                            }

                            if ui
                                .add_sized(
                                    btn_size,
                                    egui::Button::new(
                                        RichText::new("🏆")
                                            .font(icon_font.clone())
                                            .color(COR_AMARELO),
                                    )
                                    .fill(Color32::BLACK)
                                    .rounding(Rounding::same(4.0))
                                    .stroke(Stroke::new(1.5_f32, COR_AMARELO)),
                                )
                                .on_hover_text("Quadro de Líderes Offline")
                                .clicked()
                            {
                                self.modal_placar = !self.modal_placar;
                            }

                            if ui
                                .add_sized(
                                    btn_size,
                                    egui::Button::new(
                                        RichText::new("💾")
                                            .font(icon_font.clone())
                                            .color(COR_CIANO),
                                    )
                                    .fill(Color32::BLACK)
                                    .rounding(Rounding::same(4.0))
                                    .stroke(Stroke::new(1.5_f32, COR_CIANO)),
                                )
                                .on_hover_text("Gerenciar Saves (Salvar / Importar / Exportar)")
                                .clicked()
                            {
                                self.modal_saves = !self.modal_saves;
                            }

                            if ui
                                .add_sized(
                                    btn_size,
                                    egui::Button::new(
                                        RichText::new("❐")
                                            .font(icon_font.clone())
                                            .color(COR_VERDE),
                                    )
                                    .fill(Color32::BLACK)
                                    .rounding(Rounding::same(4.0))
                                    .stroke(Stroke::new(1.5_f32, COR_VERDE)),
                                )
                                .on_hover_text("Alternar Tela Cheia (F11)")
                                .clicked()
                            {
                                self.tela_cheia = !self.tela_cheia;
                                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                                    self.tela_cheia,
                                ));
                            }

                            if ui
                                .add_sized(
                                    btn_size,
                                    egui::Button::new(
                                        RichText::new("⚙")
                                            .font(icon_font.clone())
                                            .color(COR_VERDE),
                                    )
                                    .fill(Color32::BLACK)
                                    .rounding(Rounding::same(4.0))
                                    .stroke(Stroke::new(1.5_f32, COR_VERDE)),
                                )
                                .on_hover_text("Configurações de Áudio, Texto e CRT")
                                .clicked()
                            {
                                self.modal_config = !self.modal_config;
                            }

                            if ui
                                .add_sized(
                                    btn_size,
                                    egui::Button::new(
                                        RichText::new("?")
                                            .font(icon_font)
                                            .color(COR_VERDE)
                                            .strong(),
                                    )
                                    .fill(Color32::BLACK)
                                    .rounding(Rounding::same(4.0))
                                    .stroke(Stroke::new(1.5_f32, COR_VERDE)),
                                )
                                .on_hover_text("Manual de Sobrevivência")
                                .clicked()
                            {
                                self.modal_ajuda = !self.modal_ajuda;
                            }
                        });
                    });

                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("[ SISTEMA v20.08 ]")
                            .font(mono_hud.clone())
                            .color(COR_BRANCO)
                            .strong(),
                    );
                    ui.label(
                        RichText::new("------------------------------")
                            .font(FontId::new(12.5, FontFamily::Monospace))
                            .color(COR_VERDE),
                    );
                    ui.add_space(6.0);

                    // 1. Barra de HP em blocos ([███] / [ GOD MODE ])
                    let (hp_str, cor_hp) = if !em_jogo {
                        ("...".to_string(), COR_VERDE)
                    } else if self.jogo.god_mode {
                        ("[ GOD MODE ]".to_string(), COR_AMARELO)
                    } else {
                        let max_hp = if self.jogo.dificuldade == "PESADELO" {
                            2
                        } else if self.jogo.dificuldade == "CUSTOM" {
                            self.jogo.custom_hp.max(1)
                        } else {
                            3
                        };
                        let atual = self.jogo.hp.clamp(0, max_hp);
                        let cheios = "█".repeat(atual as usize);
                        let vazios = "░".repeat((max_hp - atual).max(0) as usize);
                        let cor = if atual <= 1 {
                            COR_VERMELHO
                        } else if atual == 2 && max_hp >= 3 {
                            COR_AMARELO
                        } else {
                            COR_VERDE
                        };
                        (format!("[{}{}] ({}/{})", cheios, vazios, atual, max_hp), cor)
                    };

                    ui.label(
                        RichText::new(format!("HP:  {}", hp_str))
                            .font(mono_hud.clone())
                            .color(cor_hp),
                    );
                    ui.add_space(4.0);

                    // 2. Indicador de LUZ
                    let (luz_str, cor_luz) = if !em_jogo {
                        ("...".to_string(), COR_VERDE)
                    } else if self.jogo.god_mode {
                        ("∞".to_string(), COR_AMARELO)
                    } else if self.jogo.turnos_luz <= 0 {
                        ("0 [BREU]".to_string(), COR_VERMELHO)
                    } else if self.jogo.turnos_luz <= 3 {
                        (format!("{} turnos", self.jogo.turnos_luz), COR_AMARELO)
                    } else {
                        (format!("{} turnos", self.jogo.turnos_luz), COR_VERDE)
                    };

                    ui.label(
                        RichText::new(format!("LUZ: {}", luz_str))
                            .font(mono_hud.clone())
                            .color(cor_luz),
                    );
                    ui.add_space(4.0);

                    // 3. Indicador de SOM
                    let cor_som = if self.jogo.nivel_barulho >= 80 {
                        COR_VERMELHO
                    } else if self.jogo.nivel_barulho >= 50 {
                        COR_AMARELO
                    } else {
                        COR_VERDE
                    };
                    ui.label(
                        RichText::new(format!("SOM: {}%", self.jogo.nivel_barulho))
                            .font(mono_hud.clone())
                            .color(cor_som),
                    );

                    ui.add_space(8.0);

                    // 4. Indicador de INVENTÁRIO com slots visuais [■] [□]
                    let limite_inv = self.jogo.obter_limite_inventario();
                    let ocupados = self.jogo.inventario.len().min(limite_inv);
                    let mut slots_visuais = String::new();
                    for i in 0..limite_inv {
                        if i < ocupados {
                            slots_visuais.push_str("[■]");
                        } else {
                            slots_visuais.push_str("[□]");
                        }
                    }

                    ui.label(
                        RichText::new(format!("INV: {}", slots_visuais))
                            .font(mono_hud.clone())
                            .color(COR_VERDE)
                            .strong(),
                    );

                    if em_jogo {
                        egui::ScrollArea::vertical()
                            .id_salt("scroll_inventario_hud")
                            .max_height(135.0)
                            .show(ui, |ui| {
                                if self.jogo.inventario.is_empty() {
                                    ui.label(
                                        RichText::new(" (Vazio)")
                                            .font(FontId::new(13.0, FontFamily::Monospace))
                                            .color(COR_CINZA),
                                    );
                                } else {
                                    for item in self.jogo.inventario.clone() {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(format!("• {}", item))
                                                    .font(FontId::new(13.5, FontFamily::Monospace))
                                                    .color(COR_VERDE),
                                            );
                                            if ui
                                                .small_button(
                                                    RichText::new("[Usar]")
                                                        .font(FontId::new(
                                                            11.0,
                                                            FontFamily::Monospace,
                                                        ))
                                                        .color(COR_AMARELO),
                                                )
                                                .on_hover_text(format!("Executar: usar {}", item))
                                                .clicked()
                                            {
                                                comando_atalho_clicado =
                                                    Some(format!("usar {}", item));
                                            }
                                        });
                                    }
                                }
                            });
                    }

                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("------------------------------")
                            .font(FontId::new(12.5, FontFamily::Monospace))
                            .color(COR_VERDE),
                    );

                    // 5. ATALHOS RÁPIDOS E D-PAD CLICÁVEL (igual ao #quick-actions do web)
                    ui.label(
                        RichText::new("[ CONTROLES RÁPIDOS ]")
                            .font(FontId::new(13.0, FontFamily::Monospace))
                            .color(COR_AMARELO)
                            .strong(),
                    );
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        let w = (ui.available_width() - 8.0) / 2.0;
                        if ui
                            .add_sized(
                                Vec2::new(w, 28.0),
                                egui::Button::new(
                                    RichText::new("⏿ OLHAR")
                                        .font(FontId::new(12.5, FontFamily::Monospace))
                                        .color(COR_VERDE),
                                )
                                .fill(Color32::from_rgb(0, 22, 0))
                                .stroke(Stroke::new(1.0_f32, COR_VERDE)),
                            )
                            .clicked()
                        {
                            comando_atalho_clicado = Some("olhar".to_string());
                        }
                        if ui
                            .add_sized(
                                Vec2::new(w, 28.0),
                                egui::Button::new(
                                    RichText::new("🎒 INV")
                                        .font(FontId::new(12.5, FontFamily::Monospace))
                                        .color(COR_VERDE),
                                )
                                .fill(Color32::from_rgb(0, 22, 0))
                                .stroke(Stroke::new(1.0_f32, COR_VERDE)),
                            )
                            .clicked()
                        {
                            comando_atalho_clicado = Some("inventario".to_string());
                        }
                    });

                    ui.add_space(6.0);

                    let em_combate = self.jogo.estado_atual == "COMBATE_ANIMATRONICO";
                    let prefixo_mov = if em_combate { "correr " } else { "" };
                    let cor_mov = if em_combate { COR_VERMELHO } else { COR_VERDE };

                    // Linha superior do D-Pad: [ ▲ FRENTE ]
                    ui.vertical_centered(|ui| {
                        if ui
                            .add_sized(
                                Vec2::new(82.0, 28.0),
                                egui::Button::new(
                                    RichText::new("▲ FRENTE")
                                        .font(FontId::new(12.0, FontFamily::Monospace))
                                        .color(cor_mov),
                                )
                                .fill(Color32::BLACK)
                                .stroke(Stroke::new(1.0_f32, cor_mov)),
                            )
                            .clicked()
                        {
                            comando_atalho_clicado = Some(format!("{}f", prefixo_mov));
                        }
                    });

                    // Linha central do D-Pad: [ ◀ ESQ ] [ ▼ TRÁS ] [ ▶ DIR ]
                    ui.horizontal(|ui| {
                        let w3 = (ui.available_width() - 12.0) / 3.0;
                        if ui
                            .add_sized(
                                Vec2::new(w3, 28.0),
                                egui::Button::new(
                                    RichText::new("◀ ESQ")
                                        .font(FontId::new(12.0, FontFamily::Monospace))
                                        .color(cor_mov),
                                )
                                .fill(Color32::BLACK)
                                .stroke(Stroke::new(1.0_f32, cor_mov)),
                            )
                            .clicked()
                        {
                            comando_atalho_clicado = Some(format!("{}e", prefixo_mov));
                        }
                        if ui
                            .add_sized(
                                Vec2::new(w3, 28.0),
                                egui::Button::new(
                                    RichText::new("▼ TRÁS")
                                        .font(FontId::new(12.0, FontFamily::Monospace))
                                        .color(cor_mov),
                                )
                                .fill(Color32::BLACK)
                                .stroke(Stroke::new(1.0_f32, cor_mov)),
                            )
                            .clicked()
                        {
                            comando_atalho_clicado = Some(format!("{}a", prefixo_mov));
                        }
                        if ui
                            .add_sized(
                                Vec2::new(w3, 28.0),
                                egui::Button::new(
                                    RichText::new("▶ DIR")
                                        .font(FontId::new(12.0, FontFamily::Monospace))
                                        .color(cor_mov),
                                )
                                .fill(Color32::BLACK)
                                .stroke(Stroke::new(1.0_f32, cor_mov)),
                            )
                            .clicked()
                        {
                            comando_atalho_clicado = Some(format!("{}d", prefixo_mov));
                        }
                    });
                });
            });

        if let Some(cmd_rapido) = comando_atalho_clicado {
            self.executar_comando_direto(&cmd_rapido, ctx);
        }

        // =====================================================================
        // PAINEL CENTRAL ESQUERDO - TERMINAL MS-DOS C:\>
        // =====================================================================
        let cor_fundo = if self.panic_ativo {
            Color32::from_rgb(0, 0, 95)
        } else if self.media.flash_ticks % 2 == 1 && self.audio.config.jumpscare_visual {
            Color32::from_rgb(130, 0, 0)
        } else {
            Color32::BLACK
        };

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(cor_fundo)
                    .inner_margin(egui::Margin::symmetric(18.0, 16.0)),
            )
            .show(ctx, |ui| {
                // Se estiver em Kernel Panic, exibe `image.png` no topo
                if self.panic_ativo {
                    if let Some(tex) = self.media.obter_textura(ctx, "image.png") {
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::Image::new(&tex)
                                    .max_height(150.0)
                                    .maintain_aspect_ratio(true),
                            );
                        });
                    }
                }

                // Se houver uma imagem de inspeção/sala ativa
                if let Some(img_info) = self.media.imagem_atual.clone() {
                    let chave_tex = img_info
                        .chave_embutida
                        .clone()
                        .unwrap_or_else(|| img_info.titulo.clone());
                    let tex_opt = self.media.obter_textura(ctx, &chave_tex);

                    if tex_opt.is_some() || img_info.caminho.is_some() {
                        egui::Frame::none()
                            .stroke(Stroke::new(
                                1.5_f32,
                                if img_info.secreta || img_info.eh_jumpscare {
                                    COR_VERMELHO
                                } else {
                                    COR_VERDE
                                },
                            ))
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!(
                                            "[ ARQUIVO VISUAL: {} ]",
                                            img_info.titulo
                                        ))
                                        .font(mono.clone())
                                        .color(if img_info.secreta {
                                            COR_AMARELO
                                        } else {
                                            COR_VERDE
                                        }),
                                    );
                                    if ui
                                        .button(RichText::new("[Fechar X]").color(COR_VERMELHO))
                                        .clicked()
                                    {
                                        self.media.imagem_atual = None;
                                    }
                                });
                                if let Some(tex) = tex_opt {
                                    ui.add(
                                        egui::Image::new(&tex)
                                            .max_height(240.0)
                                            .maintain_aspect_ratio(true),
                                    );
                                } else if let Some(caminho) = img_info.caminho {
                                    let uri = format!("file://{}", caminho.display());
                                    ui.add(
                                        egui::Image::new(uri)
                                            .max_height(240.0)
                                            .maintain_aspect_ratio(true),
                                    );
                                }
                            });
                        ui.add_space(8.0);
                    }
                }

                // Se houve vitória recente, exibe barra convidando para registrar no Quadro de Líderes Offline
                if let Some((pts, ref dif, ref t_final, tempo)) = self.pontuacao_pendente.clone() {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(0, 28, 0))
                        .stroke(Stroke::new(1.5_f32, COR_AMARELO))
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!(
                                        "🏆 PARTIDA CONCLUÍDA! Pontuação: {} pts | Dificuldade: {} | Final: {} | Tempo: {}s",
                                        pts,
                                        dif,
                                        t_final.to_uppercase(),
                                        tempo
                                    ))
                                    .font(FontId::new(13.5, FontFamily::Monospace))
                                    .color(COR_AMARELO)
                                    .strong(),
                                );
                                if ui
                                    .button(
                                        RichText::new("[Registrar no Placar / Copiar Resumo]")
                                            .color(COR_VERDE)
                                            .strong(),
                                    )
                                    .clicked()
                                {
                                    self.modal_placar = true;
                                }
                            });
                        });
                    ui.add_space(6.0);
                }

                // Legenda de áudio ativa no rodapé do terminal (se habilitada)
                if let Some(legenda) = self.audio.obter_legenda_ativa().map(|s| s.to_string()) {
                    egui::TopBottomPanel::bottom("painel_legenda_acessibilidade")
                        .frame(egui::Frame::none().inner_margin(egui::Margin::symmetric(0.0, 2.0)))
                        .show_separator_line(false)
                        .show_inside(ui, |ui| {
                            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                                ui.label(
                                    RichText::new(legenda)
                                        .font(FontId::new(13.0, FontFamily::Monospace))
                                        .color(COR_AMARELO)
                                        .italics(),
                                );
                            });
                        });
                }

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for linha in &self.linhas_visiveis {
                            let mut job = LayoutJob::default();
                            for seg in &linha.segmentos {
                                job.append(
                                    &seg.texto,
                                    0.0,
                                    TextFormat {
                                        font_id: mono.clone(),
                                        color: seg.cor,
                                        ..Default::default()
                                    },
                                );
                            }
                            ui.label(job);
                        }

                        // Renderiza a linha atualmente em digitação (efeito typewriter)
                        if let Some(ref atual) = self.linha_digitando {
                            let mut job = LayoutJob::default();
                            let mut restantes = atual.chars_visiveis;
                            for seg in &atual.linha.segmentos {
                                if restantes == 0 {
                                    break;
                                }
                                let qtd_seg = seg.texto.chars().count();
                                let slice: String = seg.texto.chars().take(restantes).collect();
                                job.append(
                                    &slice,
                                    0.0,
                                    TextFormat {
                                        font_id: mono.clone(),
                                        color: seg.cor,
                                        ..Default::default()
                                    },
                                );
                                restantes = restantes.saturating_sub(qtd_seg);
                            }
                            job.append(
                                "█",
                                0.0,
                                TextFormat {
                                    font_id: mono.clone(),
                                    color: COR_VERDE,
                                    ..Default::default()
                                },
                            );
                            ui.label(job);
                        }

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("C:\\>")
                                    .font(mono.clone())
                                    .color(COR_VERDE)
                                    .strong(),
                            );

                            let input_id = egui::Id::new("terminal_cmd_input");
                            let texto_antes = self.entrada.clone();
                            let resp = ui.add(
                                egui::TextEdit::singleline(&mut self.entrada)
                                    .id(input_id)
                                    .font(mono.clone())
                                    .text_color(COR_VERDE)
                                    .frame(false)
                                    .lock_focus(true)
                                    .desired_width(ui.available_width() - 95.0),
                            );

                            if self.entrada != texto_antes {
                                self.audio.som_tecla();
                            }

                            let clicou_enviar = ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("ENVIAR ↵")
                                            .font(FontId::new(13.0, FontFamily::Monospace))
                                            .color(COR_VERDE),
                                    )
                                    .fill(Color32::from_rgb(0, 30, 0))
                                    .stroke(Stroke::new(1.0_f32, COR_VERDE)),
                                )
                                .clicked();

                            let modal_aberto = self.modal_config
                                || self.modal_conquistas
                                || self.modal_ajuda
                                || self.modal_saves
                                || self.modal_placar;

                            if !modal_aberto {
                                if enter_pressionado || clicou_enviar {
                                    self.submeter_comando(ctx);
                                }
                                resp.request_focus();
                            }
                        });
                    });
            });

        // =====================================================================
        // TOAST FLUTUANTE DE CONQUISTA DESBLOQUEADA
        // =====================================================================
        if let Some((ref titulo, ref desc, quando)) = self.toast_conquista.clone() {
            if quando.elapsed() < Duration::from_secs(4) {
                ctx.request_repaint_after(Duration::from_millis(100));
                egui::Area::new(egui::Id::new("toast_conquista_area"))
                    .order(Order::Foreground)
                    .fixed_pos(Pos2::new(ctx.screen_rect().right() - 360.0, 20.0))
                    .show(ctx, |ui| {
                        egui::Frame::none()
                            .fill(Color32::from_rgb(15, 18, 5))
                            .stroke(Stroke::new(2.0_f32, COR_AMARELO))
                            .rounding(Rounding::same(6.0))
                            .inner_margin(10.0)
                            .show(ui, |ui| {
                                ui.set_max_width(330.0);
                                ui.label(
                                    RichText::new("🏆 CONQUISTA DESBLOQUEADA!")
                                        .font(FontId::new(12.5, FontFamily::Monospace))
                                        .color(COR_AMARELO)
                                        .strong(),
                                );
                                ui.label(
                                    RichText::new(titulo)
                                        .font(FontId::new(14.5, FontFamily::Monospace))
                                        .color(COR_VERDE)
                                        .strong(),
                                );
                                ui.label(
                                    RichText::new(desc)
                                        .font(FontId::new(12.0, FontFamily::Monospace))
                                        .color(COR_BRANCO),
                                );
                            });
                    });
            } else {
                self.toast_conquista = None;
            }
        }

        // =====================================================================
        // JANELA [⚙] CONFIGURAÇÕES DO SISTEMA
        // =====================================================================
        if self.modal_config {
            let mut modal_aberto = self.modal_config;
            let mut fechar_modal = false;
            egui::Window::new(
                RichText::new("[ ⚙ CONFIGURAÇÕES DO SISTEMA ]")
                    .color(COR_VERDE)
                    .font(mono_hud.clone()),
            )
            .collapsible(false)
            .resizable(false)
            .default_width(450.0)
            .open(&mut modal_aberto)
            .show(ctx, |ui| {
                let mut mudou = false;
                ui.label(
                    RichText::new(">> CONTROLE DE ÁUDIO")
                        .color(COR_AMARELO)
                        .strong(),
                );
                mudou |= ui
                    .add(
                        egui::Slider::new(&mut self.audio.config.volume_geral, 0.0..=1.0)
                            .text("Volume Geral"),
                    )
                    .changed();
                mudou |= ui
                    .add(
                        egui::Slider::new(&mut self.audio.config.volume_jumpscare, 0.0..=1.0)
                            .text("Volume Jumpscare (grito.mp3)"),
                    )
                    .changed();
                mudou |= ui
                    .add(
                        egui::Slider::new(&mut self.audio.config.volume_ambiente, 0.0..=1.0)
                            .text("Volume Ambiente (ambiente.mp3)"),
                    )
                    .changed();
                mudou |= ui
                    .add(
                        egui::Slider::new(&mut self.audio.config.volume_efeitos, 0.0..=1.0)
                            .text("Volume Efeitos / Bipes"),
                    )
                    .changed();
                mudou |= ui
                    .checkbox(&mut self.audio.config.mudo, "Silenciar Todo o Áudio (Mudo)")
                    .changed();
                mudou |= ui
                    .checkbox(
                        &mut self.audio.config.som_digitacao,
                        "Som Mecânico de Digitação (Teclas)",
                    )
                    .changed();
                mudou |= ui
                    .checkbox(
                        &mut self.audio.config.legendas_audio,
                        "Legendas de Acessibilidade para Efeitos Sonoros",
                    )
                    .changed();

                ui.add_space(8.0);
                ui.label(
                    RichText::new(">> TERMINAL E VÍDEO")
                        .color(COR_AMARELO)
                        .strong(),
                );
                mudou |= ui
                    .checkbox(
                        &mut self.audio.config.jumpscare_visual,
                        "Exibir Flash e Imagens de Jumpscare",
                    )
                    .changed();
                mudou |= ui
                    .checkbox(
                        &mut self.audio.config.efeito_crt,
                        "Filtro de Monitor CRT (Scanlines)",
                    )
                    .changed();

                ui.horizontal(|ui| {
                    ui.label("Velocidade do Texto:");
                    for (rotulo, valor) in [
                        ("Instantâneo", 0.0_f32),
                        ("0.5x", 0.5_f32),
                        ("1x (Padrão)", 1.0_f32),
                        ("2x (Rápido)", 2.0_f32),
                    ] {
                        let selecionado =
                            (self.audio.config.velocidade_digitacao - valor).abs() < 0.05;
                        if ui.selectable_label(selecionado, rotulo).clicked() {
                            self.audio.config.velocidade_digitacao = valor;
                            self.audio.config.animacao_texto = valor > 0.0;
                            mudou = true;
                        }
                    }
                });

                if mudou {
                    self.audio.atualizar_volume_ambiente();
                    self.audio.config.salvar();
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("🔊 Testar Som de Jumpscare").clicked() {
                        self.audio.som_grito();
                    }
                    if ui.button("📺 Rever Intro da TV").clicked() {
                        self.intro_tv = EstadoIntroTv::Desligada;
                        fechar_modal = true;
                    }
                });
            });
            self.modal_config = modal_aberto && !fechar_modal;
        }

        // =====================================================================
        // JANELA [👑] CONQUISTAS DO SISTEMA
        // =====================================================================
        if self.modal_conquistas {
            egui::Window::new(
                RichText::new("[ 👑 CONQUISTAS DO SISTEMA ]")
                    .color(COR_AMARELO)
                    .font(mono_hud.clone()),
            )
            .collapsible(false)
            .resizable(true)
            .default_width(540.0)
            .open(&mut self.modal_conquistas)
            .show(ctx, |ui| {
                let total = CONQUISTAS_DB.len();
                let desbloqueadas_count = CONQUISTAS_DB
                    .iter()
                    .filter(|c| {
                        self.audio
                            .config
                            .conquistas_globais
                            .iter()
                            .any(|x| x == c.id || x == c.chave_db)
                            || self
                                .jogo
                                .conquistas
                                .iter()
                                .any(|x| x == c.id || x == c.chave_db)
                    })
                    .count();

                ui.label(
                    RichText::new(format!(
                        "Progresso Global: {}/{} Conquistas ({:.0}%)",
                        desbloqueadas_count,
                        total,
                        (desbloqueadas_count as f32 / total as f32) * 100.0
                    ))
                    .color(COR_VERDE)
                    .strong(),
                );
                ui.add_space(6.0);

                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for c in &CONQUISTAS_DB {
                            let desbloqueada = self
                                .audio
                                .config
                                .conquistas_globais
                                .iter()
                                .any(|x| x == c.id || x == c.chave_db)
                                || self
                                    .jogo
                                    .conquistas
                                    .iter()
                                    .any(|x| x == c.id || x == c.chave_db);
                            let cor_titulo = if desbloqueada { COR_VERDE } else { COR_CINZA };
                            let status = if desbloqueada { "[X]" } else { "[ ]" };

                            ui.label(
                                RichText::new(format!("{} {} {}", status, c.icone, c.nome))
                                    .font(mono_hud.clone())
                                    .color(cor_titulo)
                                    .strong(),
                            );
                            ui.label(
                                RichText::new(format!("    {}", c.desc))
                                    .color(if desbloqueada { COR_BRANCO } else { COR_CINZA }),
                            );
                            ui.add_space(6.0);
                        }
                    });
            });
        }

        // =====================================================================
        // JANELA [💾] GERENCIAMENTO DE SAVES (LOCAL + EXPORTAR/IMPORTAR JSON)
        // =====================================================================
        if self.modal_saves {
            egui::Window::new(
                RichText::new("[ 💾 GERENCIAMENTO DE SAVES OFFLINE ]")
                    .color(COR_CIANO)
                    .font(mono_hud.clone()),
            )
            .collapsible(false)
            .resizable(true)
            .default_width(520.0)
            .open(&mut self.modal_saves)
            .show(ctx, |ui| {
                let existe = GameState::save_existe();
                ui.label(
                    RichText::new(if existe {
                        "Status do Autosave Local (savegame.json): ENCONTRADO ✔"
                    } else {
                        "Status do Autosave Local (savegame.json): NENHUM SAVE SALVO"
                    })
                    .color(if existe { COR_VERDE } else { COR_AMARELO })
                    .strong(),
                );
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    if ui.button("💾 Salvar Jogo Atual").clicked() {
                        if self.jogo.salvar_local() {
                            self.feedback_save = Some((
                                "Progresso salvo em savegame.json!".to_string(),
                                COR_VERDE,
                                Instant::now(),
                            ));
                        }
                    }

                    if ui
                        .add_enabled(existe, egui::Button::new("📂 Carregar Save Local"))
                        .clicked()
                    {
                        if let Some(salvo) = GameState::carregar_local() {
                            self.jogo = salvo;
                            self.audio.iniciar_ambiente();
                            self.term.limpar();
                            views::imprimir_contexto_sala(
                                &self.jogo,
                                &mut self.term,
                                &mut self.media,
                            );
                            self.feedback_save = Some((
                                "Save carregado com sucesso!".to_string(),
                                COR_VERDE,
                                Instant::now(),
                            ));
                        }
                    }

                    if ui
                        .add_enabled(existe, egui::Button::new("🗑 Apagar Save Local"))
                        .clicked()
                    {
                        let _ = fs::remove_file(state::ARQUIVO_SAVE);
                        self.feedback_save = Some((
                            "Arquivo savegame.json removido.".to_string(),
                            COR_AMARELO,
                            Instant::now(),
                        ));
                    }
                });

                ui.add_space(10.0);
                ui.separator();
                ui.label(
                    RichText::new(">> EXPORTAR / IMPORTAR SAVE EM JSON")
                        .color(COR_AMARELO)
                        .strong(),
                );
                ui.label(
                    "Você pode exportar seu save para texto JSON (para backup) ou colar um JSON abaixo para importar:",
                );

                ui.horizontal(|ui| {
                    if ui.button("📤 Gerar JSON do Estado Atual").clicked() {
                        if let Ok(json) = serde_json::to_string_pretty(&self.jogo) {
                            self.json_save_buffer = json.clone();
                            ctx.copy_text(json);
                            self.feedback_save = Some((
                                "JSON gerado e copiado para a Área de Transferência!".to_string(),
                                COR_VERDE,
                                Instant::now(),
                            ));
                        }
                    }

                    if ui.button("📥 Importar JSON Abaixo").clicked() {
                        match serde_json::from_str::<GameState>(&self.json_save_buffer) {
                            Ok(mut novo) => {
                                novo.ascii_arts = data::carregar_ascii_arts();
                                novo.sinonimos = data::sinonimos_padrao();
                                self.jogo = novo;
                                let _ = self.jogo.salvar_local();
                                self.audio.iniciar_ambiente();
                                self.term.limpar();
                                views::imprimir_contexto_sala(
                                    &self.jogo,
                                    &mut self.term,
                                    &mut self.media,
                                );
                                self.feedback_save = Some((
                                    "Save JSON importado com sucesso!".to_string(),
                                    COR_VERDE,
                                    Instant::now(),
                                ));
                            }
                            Err(e) => {
                                self.feedback_save = Some((
                                    format!("Erro ao ler JSON: {}", e),
                                    COR_VERMELHO,
                                    Instant::now(),
                                ));
                            }
                        }
                    }
                });

                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.json_save_buffer)
                                .font(FontId::new(12.0, FontFamily::Monospace))
                                .desired_width(f32::INFINITY)
                                .desired_rows(6),
                        );
                    });

                if let Some((ref msg, cor, quando)) = self.feedback_save {
                    if quando.elapsed() < Duration::from_secs(5) {
                        ui.add_space(4.0);
                        ui.label(RichText::new(msg).color(cor).strong());
                    }
                }
            });
        }

        // =====================================================================
        // JANELA [🏆] QUADRO DE LÍDERES OFFLINE & RESUMO DA PARTIDA
        // =====================================================================
        if self.modal_placar {
            egui::Window::new(
                RichText::new("[ 🏆 QUADRO DE LÍDERES LOCAL (OFFLINE) ]")
                    .color(COR_AMARELO)
                    .font(mono_hud.clone()),
            )
            .collapsible(false)
            .resizable(true)
            .default_width(540.0)
            .open(&mut self.modal_placar)
            .show(ctx, |ui| {
                if let Some((pts, dif, t_final, tempo)) = self.pontuacao_pendente.clone() {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(15, 25, 15))
                        .stroke(Stroke::new(1.0_f32, COR_VERDE))
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(format!(
                                    "REGISTRAR NOVA VITÓRIA: {} pts ({} - Final {})",
                                    pts,
                                    dif,
                                    t_final.to_uppercase()
                                ))
                                .color(COR_VERDE)
                                .strong(),
                            );
                            ui.horizontal(|ui| {
                                ui.label("Seu Nome / Iniciais:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.nome_jogador_placar)
                                        .desired_width(140.0),
                                );
                                if ui.button("✔ Salvar no Ranking Local").clicked() {
                                    let reg = RegistroPlacarLocal {
                                        nome: if self.nome_jogador_placar.trim().is_empty() {
                                            "ANONIMO".to_string()
                                        } else {
                                            self.nome_jogador_placar.trim().to_uppercase()
                                        },
                                        pontuacao: pts,
                                        dificuldade: dif.clone(),
                                        tipo_final: t_final.clone(),
                                        tempo_segundos: tempo,
                                        data: "LOCAL".to_string(),
                                    };
                                    self.audio.config.registrar_pontuacao(reg);
                                    self.pontuacao_pendente = None;
                                    self.feedback_placar = Some(
                                        "Pontuação registrada no Quadro de Líderes Local!"
                                            .to_string(),
                                    );
                                }
                            });

                            if ui.button("📋 Copiar Card de Resumo da Partida").clicked() {
                                let card = format!(
                                    "📼 VILLAS-BOAS ENTERTAINMENT (1982) - RELATÓRIO DE TURNO\n\
                                    👤 Operador: {}\n\
                                    🏆 Pontuação: {} pts\n\
                                    💀 Dificuldade: {}\n\
                                    🎬 Desfecho: Final {}\n\
                                    ⏱ Tempo: {}m {:02}s\n\
                                    👑 Conquistas: {}/{}",
                                    self.nome_jogador_placar.trim(),
                                    pts,
                                    dif,
                                    t_final.to_uppercase(),
                                    tempo / 60,
                                    tempo % 60,
                                    self.audio.config.conquistas_globais.len(),
                                    CONQUISTAS_DB.len()
                                );
                                ctx.copy_text(card);
                                self.feedback_placar = Some(
                                    "Card copiado para a Área de Transferência!".to_string(),
                                );
                            }
                        });
                    ui.add_space(8.0);
                }

                if let Some(ref fb) = self.feedback_placar {
                    ui.label(RichText::new(fb).color(COR_VERDE));
                    ui.add_space(4.0);
                }

                if self.audio.config.placar_local.is_empty() {
                    ui.label(
                        RichText::new("Nenhuma partida registrada ainda. Vença a noite para estrear o placar!")
                            .color(COR_CINZA),
                    );
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for (idx, reg) in self.audio.config.placar_local.iter().enumerate() {
                                let cor = match idx {
                                    0 => COR_AMARELO,
                                    1 => COR_BRANCO,
                                    2 => COR_CIANO,
                                    _ => COR_VERDE,
                                };
                                ui.label(
                                    RichText::new(format!(
                                        "#{:02} | {:<12} | {:>5} pts | {:<16} | Final {:<10} | {}m{:02}s",
                                        idx + 1,
                                        reg.nome,
                                        reg.pontuacao,
                                        reg.dificuldade,
                                        reg.tipo_final,
                                        reg.tempo_segundos / 60,
                                        reg.tempo_segundos % 60
                                    ))
                                    .font(FontId::new(13.0, FontFamily::Monospace))
                                    .color(cor),
                                );
                            }
                        });
                }
            });
        }

        // =====================================================================
        // JANELA [?] MANUAL DE SOBREVIVÊNCIA
        // =====================================================================
        if self.modal_ajuda {
            egui::Window::new(
                RichText::new("[ ? MANUAL DE SOBREVIVÊNCIA v1.0 ]")
                    .color(COR_AMARELO)
                    .font(mono_hud),
            )
            .collapsible(false)
            .resizable(false)
            .default_width(480.0)
            .open(&mut self.modal_ajuda)
            .show(ctx, |ui| {
                ui.label(RichText::new(">> NAVEGAÇÃO:").color(COR_BRANCO).strong());
                ui.label("   ir frente (ou f), ir atrás (ou a), ir esquerda (ou e), ir direita (ou d)");
                ui.label("   Você também pode digitar o nome da saída diretamente (ex: 'ir cozinha').");
                ui.add_space(6.0);
                ui.label(RichText::new(">> INTERAÇÃO:").color(COR_BRANCO).strong());
                ui.label("   olhar [objeto] (ou x [objeto]) : Examina objetos em amarelo na sala");
                ui.label("   pegar [item] (ou p [item])     : Coleta itens em verde no chão");
                ui.label("   largar [item] (ou l [item])    : Solta um item da mochila no chão");
                ui.label("   usar [item] (ou u [item])      : Usa um item do seu inventário");
                ui.label("   combinar [item 1] com [item 2] : Combina dois itens (ex: tesoura + fita)");
                ui.add_space(6.0);
                ui.label(RichText::new(">> SOBREVIVÊNCIA:").color(COR_VERMELHO).strong());
                ui.label("   inventario (ou i) : Lista os itens na sua mochila");
                ui.label("   correr [direção]  : Foge do animatrônico, mas gera 100% de barulho!");
                ui.label("   Setas ↑ / ↓       : Navega pelo histórico de comandos digitados");
                ui.label("   F5                : Reinicia o computador MS-DOS");
            });
        }

        // Desenha overlays em tela cheia (Jumpscare, BSOD, Flash Secrets, Slideshow 1980, Scanlines CRT)
        self.desenhar_overlays_terror(ctx);
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Villas-Boas Entertainment (1982) - Terminal [ SISTEMA v20.08 ]")
            .with_inner_size([1180.0, 720.0])
            .with_min_inner_size([850.0, 520.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Villas-Boas Entertainment",
        options,
        Box::new(|cc| Ok(Box::new(VilasBoasApp::new(cc)))),
    )
}
