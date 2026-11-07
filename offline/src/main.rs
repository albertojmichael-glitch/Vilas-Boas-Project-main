#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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

use actions::{COR_AMARELO, COR_BRANCO, COR_CIANO, COR_CINZA, COR_VERDE, COR_VERMELHO};
use audio::AudioManager;
use eframe::egui::{
    self, text::LayoutJob, Color32, FontFamily, FontId, Key, RichText, Rounding, Stroke, TextFormat,
    Vec2,
};
use engine::processar_fluxo_jogo;
use media::MediaManager;
use state::GameState;
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

#[derive(Default)]
pub struct TerminalBuffer {
    pub linhas: Vec<LinhaTerminal>,
    pub efeitos_pendentes: Vec<String>,
}

impl TerminalBuffer {
    pub fn limpar(&mut self) {
        self.linhas.clear();
    }

    pub fn clear(&mut self) {
        self.linhas.clear();
    }

    pub fn exibir(&mut self, msg: &str) {
        self.push_ansi(msg, COR_BRANCO);
    }

    pub fn animar(&mut self, msg: &str, cor_padrao_ansi: &str) {
        let padrao = Self::cor_de_codigo(cor_padrao_ansi, COR_BRANCO);
        self.push_ansi(msg, padrao);
    }

    pub fn push(&mut self, texto: &str, cor_padrao: Color32) {
        self.push_ansi(texto, cor_padrao);
    }

    pub fn buffer_efeito(&mut self, marcador: &str) {
        self.efeitos_pendentes.push(marcador.to_string());
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

    pub fn push_ansi(&mut self, texto: &str, cor_inicial: Color32) {
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

            self.linhas.push(LinhaTerminal { segmentos });
        }
    }
}

struct VilasBoasApp {
    jogo: GameState,
    term: TerminalBuffer,
    entrada: String,
    audio: AudioManager,
    media: MediaManager,
    modal_conquistas: bool,
    modal_config: bool,
    modal_ajuda: bool,
    tela_cheia: bool,
    modo_energia: String,
}

impl VilasBoasApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::BLACK;
        visuals.window_fill = Color32::from_rgb(10, 10, 10);
        visuals.extreme_bg_color = Color32::BLACK;
        cc.egui_ctx.set_visuals(visuals);

        let audio = AudioManager::new();
        let mut jogo = GameState::new();
        jogo.conquistas = audio.config.conquistas_globais.clone();

        let mut term = TerminalBuffer::default();
        imprimir_tela_boot(&mut term);

        Self {
            jogo,
            term,
            entrada: String::new(),
            audio,
            media: MediaManager::new(),
            modal_conquistas: false,
            modal_config: false,
            modal_ajuda: false,
            tela_cheia: false,
            modo_energia: "NORMAL".to_string(),
        }
    }

    fn processar_efeitos_visuais(&mut self) {
        let efeitos: Vec<String> = self.term.efeitos_pendentes.drain(..).collect();
        for ef in efeitos {
            match ef.as_str() {
                "@@JUMPSCARE@@" => {
                    self.media.flash_ticks = 14;
                }
                "@@GLITCH_LUZ@@" => {
                    self.media.flash_ticks = 4;
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
                _ => {}
            }
        }
    }

    fn submeter_comando(&mut self) {
        let cmd = self.entrada.trim().to_string();
        self.entrada.clear();
        if !cmd.is_empty() {
            self.term.push(&format!("C:\\> {}", cmd), COR_VERDE);
        }
        processar_fluxo_jogo(
            &cmd,
            &mut self.jogo,
            &mut self.term,
            &mut self.audio,
            &mut self.media,
        );
        self.processar_efeitos_visuais();
    }
}

impl eframe::App for VilasBoasApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mono = FontId::new(15.5, FontFamily::Monospace);
        let mono_hud = FontId::new(15.0, FontFamily::Monospace);

        if self.media.flash_ticks > 0 {
            self.media.flash_ticks -= 1;
            ctx.request_repaint();
        }

        let enter_pressionado = ctx.input(|i| i.key_pressed(Key::Enter));
        let f5_pressionado = ctx.input(|i| i.key_pressed(Key::F5));

        if f5_pressionado {
            self.media.imagem_atual = None;
            self.modo_energia = "NORMAL".to_string();
            imprimir_tela_boot(&mut self.term);
            self.jogo.estado_atual = "AGUARDANDO_DIR".to_string();
        }

        let cor_borda_painel = match self.modo_energia.as_str() {
            "LOW" => COR_AMARELO,
            "BLACKOUT" => Color32::from_rgb(45, 45, 45),
            _ => COR_VERDE,
        };

        // PAINEL LATERAL DIREITO - [ SISTEMA v20.08 ]
        egui::SidePanel::right("hud_sistema_v2008")
            .exact_width(275.0)
            .resizable(false)
            .frame(
                egui::Frame::none()
                    .fill(Color32::BLACK)
                    .inner_margin(egui::Margin::symmetric(16.0, 16.0))
                    .stroke(Stroke::new(1.5_f32, cor_borda_painel)),
            )
            .show(ctx, |ui| {
                // Rodapé fixo com os 4 botões grandes (50x50)
                egui::TopBottomPanel::bottom("hud_botoes_rodape")
                    .frame(egui::Frame::none().inner_margin(egui::Margin::symmetric(0.0, 6.0)))
                    .show_separator_line(false)
                    .show_inside(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 10.0;
                            let btn_size = Vec2::new(50.0, 50.0);
                            let icon_font = FontId::new(23.0, FontFamily::Proportional);

                            if ui
                                .add_sized(
                                    btn_size,
                                    egui::Button::new(
                                        RichText::new("👑")
                                            .font(icon_font.clone())
                                            .color(COR_AMARELO),
                                    )
                                    .fill(Color32::BLACK)
                                    .rounding(Rounding::same(5.0))
                                    .stroke(Stroke::new(2.0_f32, COR_AMARELO)),
                                )
                                .on_hover_text("Conquistas e Desafios")
                                .clicked()
                            {
                                self.modal_conquistas = !self.modal_conquistas;
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
                                    .rounding(Rounding::same(5.0))
                                    .stroke(Stroke::new(2.0_f32, COR_VERDE)),
                                )
                                .on_hover_text("Alternar Tela Cheia")
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
                                    .rounding(Rounding::same(5.0))
                                    .stroke(Stroke::new(2.0_f32, COR_VERDE)),
                                )
                                .on_hover_text("Configurações de Áudio e Jumpscare")
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
                                    .rounding(Rounding::same(5.0))
                                    .stroke(Stroke::new(2.0_f32, COR_VERDE)),
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
                        RichText::new("---------------------------")
                            .font(FontId::new(13.0, FontFamily::Monospace))
                            .color(COR_VERDE),
                    );
                    ui.add_space(8.0);

                    let em_jogo = self.jogo.estado_atual == "JOGO"
                        || self.jogo.estado_atual == "COMBATE_ANIMATRONICO"
                        || self.jogo.estado_atual.starts_with("MINIGAME_");

                    let hp_str = if em_jogo {
                        format!("{}", self.jogo.hp)
                    } else {
                        "...".to_string()
                    };
                    let luz_str = if em_jogo {
                        format!("{}", self.jogo.turnos_luz)
                    } else {
                        "...".to_string()
                    };
                    let som_str = format!("{}%", self.jogo.nivel_barulho);

                    ui.label(
                        RichText::new(format!("HP:   {}", hp_str))
                            .font(mono_hud.clone())
                            .color(COR_VERDE),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!("LUZ:  {}", luz_str))
                            .font(mono_hud.clone())
                            .color(COR_VERDE),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!("SOM:  {}", som_str))
                            .font(mono_hud.clone())
                            .color(if self.jogo.nivel_barulho >= 80 {
                                COR_VERMELHO
                            } else {
                                COR_VERDE
                            }),
                    );
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("INV:")
                            .font(mono_hud.clone())
                            .color(COR_VERDE)
                            .strong(),
                    );

                    if em_jogo {
                        for item in &self.jogo.inventario {
                            ui.label(
                                RichText::new(format!(" - {}", item))
                                    .font(FontId::new(14.0, FontFamily::Monospace))
                                    .color(COR_VERDE),
                            );
                        }
                    }
                });
            });

        // PAINEL CENTRAL ESQUERDO - Terminal C:\>
        let cor_fundo = if self.media.flash_ticks % 2 == 1 && self.audio.config.jumpscare_visual {
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
                if let Some(img_info) = self.media.imagem_atual.clone() {
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
                            let uri = format!("file://{}", img_info.caminho.display());
                            ui.add(
                                egui::Image::new(uri)
                                    .max_height(250.0)
                                    .maintain_aspect_ratio(true),
                            );
                        });
                    ui.add_space(8.0);
                }

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for linha in &self.term.linhas {
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

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("C:\\>")
                                    .font(mono.clone())
                                    .color(COR_VERDE)
                                    .strong(),
                            );

                            let input_id = egui::Id::new("terminal_cmd_input");
                            let resp = ui.add(
                                egui::TextEdit::singleline(&mut self.entrada)
                                    .id(input_id)
                                    .font(mono.clone())
                                    .text_color(COR_VERDE)
                                    .frame(false)
                                    .lock_focus(true)
                                    .desired_width(ui.available_width() - 95.0),
                            );

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

                            let modal_aberto =
                                self.modal_config || self.modal_conquistas || self.modal_ajuda;

                            if !modal_aberto {
                                if enter_pressionado || clicou_enviar {
                                    self.submeter_comando();
                                }
                                resp.request_focus();
                            }
                        });
                    });
            });

        // JANELA [⚙] CONFIGURAÇÕES
        if self.modal_config {
            egui::Window::new(
                RichText::new("[ ⚙ CONFIGURAÇÕES DO SISTEMA ]")
                    .color(COR_VERDE)
                    .font(mono_hud.clone()),
            )
            .collapsible(false)
            .resizable(false)
            .default_width(420.0)
            .open(&mut self.modal_config)
            .show(ctx, |ui| {
                let mut mudou = false;
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
                            .text("Volume Efeitos (Portas/Metal)"),
                    )
                    .changed();
                mudou |= ui
                    .checkbox(&mut self.audio.config.mudo, "Silenciar Todo o Áudio (Mudo)")
                    .changed();
                mudou |= ui
                    .checkbox(
                        &mut self.audio.config.jumpscare_visual,
                        "Flash Vermelho de Jumpscare",
                    )
                    .changed();

                if mudou {
                    self.audio.atualizar_volume_ambiente();
                    self.audio.config.salvar();
                }

                ui.add_space(8.0);
                if ui.button("🔊 Testar Som de Jumpscare").clicked() {
                    self.audio.som_grito();
                }
            });
        }

        // JANELA [👑] CONQUISTAS (CONQUISTAS_DB do utils.py)
        if self.modal_conquistas {
            egui::Window::new(
                RichText::new("[ 👑 CONQUISTAS DO SISTEMA ]")
                    .color(COR_AMARELO)
                    .font(mono_hud.clone()),
            )
            .collapsible(false)
            .resizable(true)
            .default_width(520.0)
            .open(&mut self.modal_conquistas)
            .show(ctx, |ui| {
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

        // JANELA [?] MANUAL DE SOBREVIVÊNCIA
        if self.modal_ajuda {
            egui::Window::new(
                RichText::new("[ ? MANUAL DE SOBREVIVÊNCIA v1.0 ]")
                    .color(COR_AMARELO)
                    .font(mono_hud),
            )
            .collapsible(false)
            .resizable(false)
            .open(&mut self.modal_ajuda)
            .show(ctx, |ui| {
                ui.label(RichText::new(">> NAVEGAÇÃO:").color(COR_BRANCO).strong());
                ui.label("   ir frente, ir esquerda, ir atrás, ir direita ou ir <nome da sala>");
                ui.add_space(4.0);
                ui.label(RichText::new(">> INTERAÇÃO:").color(COR_BRANCO).strong());
                ui.label("   olhar [objeto] | pegar [item] (ou p [item]) | usar [item]");
                ui.add_space(4.0);
                ui.label(RichText::new(">> GERENCIAMENTO:").color(COR_BRANCO).strong());
                ui.label("   inventario (ou i) | correr [direção] | F5 para reiniciar no fim");
            });
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Vilas Boas - Terminal [ SISTEMA v20.08 ]")
            .with_inner_size([1180.0, 720.0])
            .with_min_inner_size([850.0, 520.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Vilas Boas",
        options,
        Box::new(|cc| Ok(Box::new(VilasBoasApp::new(cc)))),
    )
}