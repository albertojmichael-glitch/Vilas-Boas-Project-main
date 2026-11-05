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

use actions::{COR_AMARELO, COR_BRANCO, COR_CIANO, COR_CINZA, COR_VERDE, COR_VERMELHO};
use audio::AudioManager;
use eframe::egui::{self, Color32, FontFamily, FontId, Key, RichText, Stroke, Vec2};
use engine::{imprimir_boot, processar_fluxo_jogo};
use media::MediaManager;
use state::GameState;

#[derive(Clone)]
pub struct LinhaTerminal {
    pub texto: String,
    pub cor: Color32,
}

#[derive(Default)]
pub struct TerminalBuffer {
    pub linhas: Vec<LinhaTerminal>,
}

impl TerminalBuffer {
    pub fn push(&mut self, texto: &str, cor: Color32) {
        for l in texto.lines() {
            self.linhas.push(LinhaTerminal {
                texto: l.to_string(),
                cor,
            });
        }
    }

    pub fn clear(&mut self) {
        self.linhas.clear();
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
}

impl VilasBoasApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::BLACK;
        visuals.window_fill = Color32::from_rgb(8, 8, 8);
        visuals.extreme_bg_color = Color32::BLACK;
        cc.egui_ctx.set_visuals(visuals);

        let audio = AudioManager::new();
        let mut jogo = GameState::new();
        jogo.conquistas = audio.config.conquistas_globais.clone();

        let mut term = TerminalBuffer::default();
        imprimir_boot(&mut term);

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
        }
    }
}

impl eframe::App for VilasBoasApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mono = FontId::new(15.0, FontFamily::Monospace);
        let mono_hud = FontId::new(14.5, FontFamily::Monospace);

        // Flash visual de Jumpscare
        if self.media.flash_ticks > 0 {
            self.media.flash_ticks -= 1;
            ctx.request_repaint();
        }

        // PAINEL LATERAL DIREITO - Idêntico à imagem [ SISTEMA v20.08 ]
        egui::SidePanel::right("hud_sistema_v2008")
            .exact_width(250.0)
            .resizable(false)
            .frame(
                egui::Frame::none()
                    .fill(Color32::BLACK)
                    .inner_margin(egui::Margin::symmetric(14.0, 14.0))
                    .stroke(Stroke::new(1.5, COR_VERDE)),
            )
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("[ SISTEMA v20.08 ]")
                            .font(mono_hud.clone())
                            .color(COR_BRANCO)
                            .strong(),
                    );
                    ui.label(
                        RichText::new("--------------------------------")
                            .font(FontId::new(12.0, FontFamily::Monospace))
                            .color(COR_VERDE),
                    );
                    ui.add_space(6.0);

                    let EmJogo = self.jogo.estado_atual == "JOGO"
                        || self.jogo.estado_atual == "COMBATE_ANIMATRONICO"
                        || self.jogo.estado_atual.starts_with("MINIGAME_");

                    let hp_str = if EmJogo {
                        format!("{}", self.jogo.hp)
                    } else {
                        "...".to_string()
                    };
                    let luz_str = if EmJogo {
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
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("INV:")
                            .font(mono_hud.clone())
                            .color(COR_VERDE)
                            .strong(),
                    );

                    if EmJogo {
                        for item in &self.jogo.inventario {
                            ui.label(
                                RichText::new(format!(" - {}", item))
                                    .font(FontId::new(13.5, FontFamily::Monospace))
                                    .color(COR_VERDE),
                            );
                        }
                    }

                    // Empurra os 4 botões para o canto inferior direito (igual ao print)
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                        ui.horizontal(|ui| {
                            let btn_size = Vec2::new(34.0, 34.0);

                            // Botão [?] Ajuda
                            if ui
                                .add_sized(
                                    btn_size,
                                    egui::Button::new(
                                        RichText::new("?").color(COR_VERDE).strong(),
                                    )
                                    .fill(Color32::BLACK)
                                    .stroke(Stroke::new(1.2, COR_VERDE)),
                                )
                                .on_hover_text("Ajuda e Comandos")
                                .clicked()
                            {
                                self.modal_ajuda = !self.modal_ajuda;
                            }

                            // Botão [⚙] Configurações (Volume Geral, Jumpscare, etc.)
                            if ui
                                .add_sized(
                                    btn_size,
                                    egui::Button::new(
                                        RichText::new("⚙").color(COR_VERDE).strong(),
                                    )
                                    .fill(Color32::BLACK)
                                    .stroke(Stroke::new(1.2, COR_VERDE)),
                                )
                                .on_hover_text("Configurações de Áudio e Jumpscare")
                                .clicked()
                            {
                                self.modal_config = !self.modal_config;
                            }

                            // Botão [❐] Tela Cheia
                            if ui
                                .add_sized(
                                    btn_size,
                                    egui::Button::new(
                                        RichText::new("❐").color(COR_VERDE).strong(),
                                    )
                                    .fill(Color32::BLACK)
                                    .stroke(Stroke::new(1.2, COR_VERDE)),
                                )
                                .on_hover_text("Alternar Tela Cheia")
                                .clicked()
                            {
                                self.tela_cheia = !self.tela_cheia;
                                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                                    self.tela_cheia,
                                ));
                            }

                            // Botão [👑] Conquistas (Amarelo como na referência)
                            if ui
                                .add_sized(
                                    btn_size,
                                    egui::Button::new(
                                        RichText::new("👑").color(COR_AMARELO).strong(),
                                    )
                                    .fill(Color32::BLACK)
                                    .stroke(Stroke::new(1.2, COR_AMARELO)),
                                )
                                .on_hover_text("Conquistas e Desafios")
                                .clicked()
                            {
                                self.modal_conquistas = !self.modal_conquistas;
                            }
                        });
                    });
                });
            });

        // PAINEL CENTRAL ESQUERDO - Terminal principal C:\> + Visualizador de Imagens
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
                // Exibe imagem ativa (sala, item inspecionado, secret image ou jumpscare)
                if let Some(img_info) = self.media.imagem_atual.clone() {
                    egui::Frame::none()
                        .stroke(Stroke::new(
                            1.5,
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
                                    .max_height(240.0)
                                    .maintain_aspect_ratio(true),
                            );
                        });
                    ui.add_space(8.0);
                }

                // Histórico de linhas do Terminal + Prompt C:\>
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for item in &self.term.linhas {
                            ui.label(
                                RichText::new(&item.texto)
                                    .font(mono.clone())
                                    .color(item.cor),
                            );
                        }

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("C:\\>")
                                    .font(mono.clone())
                                    .color(COR_VERDE)
                                    .strong(),
                            );

                            let resp = ui.add(
                                egui::TextEdit::singleline(&mut self.entrada)
                                    .font(mono.clone())
                                    .text_color(COR_VERDE)
                                    .frame(false)
                                    .desired_width(f32::INFINITY),
                            );

                            if !self.modal_config && !self.modal_conquistas && !self.modal_ajuda {
                                resp.request_focus();
                            }

                            if resp.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                                let cmd = self.entrada.trim().to_string();
                                self.entrada.clear();
                                if !cmd.is_empty() {
                                    self.term.push(&format!("C:\\> {}", cmd), COR_VERDE);
                                    processar_fluxo_jogo(
                                        &cmd,
                                        &mut self.jogo,
                                        &mut self.term,
                                        &mut self.audio,
                                        &mut self.media,
                                    );
                                }
                                resp.request_focus();
                            }
                        });
                    });
            });

        // MODAL [⚙] CONFIGURAÇÕES DE ÁUDIO E JUMPSCARE
        if self.modal_config {
            egui::Window::new(
                RichText::new("[ CONFIGURAÇÕES DO SISTEMA ]")
                    .color(COR_VERDE)
                    .font(mono_hud.clone()),
            )
            .collapsible(false)
            .resizable(false)
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

                ui.add_space(6.0);
                if ui.button("🔊 Testar Som de Jumpscare").clicked() {
                    self.audio.som_grito();
                }
            });
        }

        // MODAL [👑] CONQUISTAS
        if self.modal_conquistas {
            egui::Window::new(
                RichText::new("[ 👑 CONQUISTAS E DESAFIOS ]")
                    .color(COR_AMARELO)
                    .font(mono_hud.clone()),
            )
            .collapsible(false)
            .resizable(false)
            .open(&mut self.modal_conquistas)
            .show(ctx, |ui| {
                if self.audio.config.conquistas_globais.is_empty() {
                    ui.label(
                        RichText::new("Nenhuma conquista desbloqueada ainda.").color(COR_CINZA),
                    );
                } else {
                    for c in &self.audio.config.conquistas_globais {
                        ui.label(
                            RichText::new(format!("👑 {}", c.to_uppercase())).color(COR_VERDE),
                        );
                    }
                }
            });
        }

        // MODAL [?] AJUDA
        if self.modal_ajuda {
            egui::Window::new(
                RichText::new("[ ? MANUAL DO SISTEMA v20.08 ]")
                    .color(COR_CIANO)
                    .font(mono_hud),
            )
            .collapsible(false)
            .resizable(false)
            .open(&mut self.modal_ajuda)
            .show(ctx, |ui| {
                ui.label(RichText::new("Comandos Básicos:").color(COR_AMARELO));
                ui.label(" • dir                   : Abre o diretório inicial");
                ui.label(" • ir <direção>          : Move entre salas (frente, atrás, esquerda, direita)");
                ui.label(" • correr <direção>      : Foge do animatrônico (gera 100% de ruído)");
                ui.label(" • olhar                 : Examina a sala atual e conexões");
                ui.label(" • inspecionar <objeto>  : Examina pistas/objetos e abre imagens");
                ui.label(" • pegar / largar <item> : Gerencia itens da mochila");
                ui.label(" • usar <item>           : Usa bateria, alicate, gasolina, isqueiro, etc.");
                ui.label(" • salvar                : Salva a partida atual localmente");
            });
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Vilas Boas - Terminal [ SISTEMA v20.08 ]")
            .with_inner_size([1150.0, 680.0])
            .with_min_inner_size([800.0, 480.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Vilas Boas",
        options,
        Box::new(|cc| Ok(Box::new(VilasBoasApp::new(cc)))),
    )
}