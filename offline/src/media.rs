use crate::ui::{TerminalUI, DOS_AMARELO, DOS_CIANO, DOS_VERDE, DOS_VERMELHO, RESET};
use image::{imageops::FilterType, GenericImageView};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct MediaManager {
    image_dirs: Vec<PathBuf>,
    secret_dirs: Vec<PathBuf>,
    pub abrir_janela_externa: bool,
}

impl MediaManager {
    pub fn new() -> Self {
        Self {
            image_dirs: vec![
                PathBuf::from("assets/images"),
                PathBuf::from("assets/img"),
                PathBuf::from("assets"),
                PathBuf::from("../client/assets/images"),
                PathBuf::from("../client/assets/img"),
                PathBuf::from("../client/assets"),
                PathBuf::from("client/assets/images"),
                PathBuf::from("client/assets"),
            ],
            secret_dirs: vec![
                PathBuf::from("assets/secret_images"),
                PathBuf::from("assets/secrets"),
                PathBuf::from("../client/assets/secret_images"),
                PathBuf::from("../client/assets/secrets"),
                PathBuf::from("client/assets/secret_images"),
            ],
            abrir_janela_externa: false,
        }
    }

    fn buscar_arquivo(pastas: &[PathBuf], nome_busca: &str) -> Option<PathBuf> {
        let termo = nome_busca.to_lowercase();
        let extensoes = ["png", "jpg", "jpeg", "webp", "bmp", "gif"];

        for pasta in pastas {
            if !pasta.exists() {
                continue;
            }
            // Tentativa exata
            let direto = pasta.join(nome_busca);
            if direto.is_file() {
                return Some(direto);
            }
            for ext in &extensoes {
                let com_ext = pasta.join(format!("{}.{}", nome_busca, ext));
                if com_ext.is_file() {
                    return Some(com_ext);
                }
            }
            // Busca parcial nos arquivos do diretório
            if let Ok(entries) = fs::read_dir(pasta) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let nome_arq = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_lowercase();
                        let eh_imagem = extensoes.iter().any(|ext| nome_arq.ends_with(ext));
                        if eh_imagem && nome_arq.contains(&termo) {
                            return Some(path);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn listar_imagens_disponiveis(&self, secretas: bool) -> Vec<String> {
        let pastas = if secretas {
            &self.secret_dirs
        } else {
            &self.image_dirs
        };
        let extensoes = ["png", "jpg", "jpeg", "webp", "bmp", "gif"];
        let mut lista = Vec::new();

        for pasta in pastas {
            if let Ok(entries) = fs::read_dir(pasta) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(nome) = path.file_name().and_then(|n| n.to_str()) {
                            let lower = nome.to_lowercase();
                            if extensoes.iter().any(|ext| lower.ends_with(ext))
                                && !lista.contains(&nome.to_string())
                            {
                                lista.push(nome.to_string());
                            }
                        }
                    }
                }
            }
        }
        lista.sort();
        lista
    }

    pub fn exibir_imagem(&self, nome: &str, secreta: bool, ui: &TerminalUI) -> bool {
        let pastas = if secreta {
            &self.secret_dirs
        } else {
            &self.image_dirs
        };

        let caminho = Self::buscar_arquivo(pastas, nome).or_else(|| {
            if secreta {
                Self::buscar_arquivo(&self.image_dirs, nome)
            } else {
                Self::buscar_arquivo(&self.secret_dirs, nome)
            }
        });

        let Some(path) = caminho else {
            return false;
        };

        if secreta {
            ui.exibir(&format!(
                "{}[ ARQUIVO CONFIDENCIAL DESBLOQUEADO: {} ]{}",
                DOS_VERMELHO,
                path.file_name().and_then(|n| n.to_str()).unwrap_or(nome),
                RESET
            ));
        } else {
            ui.exibir(&format!(
                "{}[ REGISTRO VISUAL: {} ]{}",
                DOS_CIANO,
                path.file_name().and_then(|n| n.to_str()).unwrap_or(nome),
                RESET
            ));
        }

        Self::renderizar_ansi_no_terminal(&path, 68, 32, ui);

        if self.abrir_janela_externa {
            Self::abrir_no_sistema(&path);
        }

        true
    }

    fn renderizar_ansi_no_terminal(path: &Path, max_w: u32, max_h: u32, ui: &TerminalUI) {
        let Ok(img) = image::open(path) else {
            ui.exibir(&format!(
                "{}[Não foi possível decodificar a imagem no terminal]{}",
                DOS_AMARELO, RESET
            ));
            return;
        };

        let redimensionada = img.resize(max_w, max_h * 2, FilterType::Triangle);
        let (w, h) = redimensionada.dimensions();

        let mut buffer = String::new();
        let mut y = 0;
        while y < h {
            for x in 0..w {
                let top = redimensionada.get_pixel(x, y);
                let bottom = if y + 1 < h {
                    redimensionada.get_pixel(x, y + 1)
                } else {
                    top
                };
                buffer.push_str(&format!(
                    "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m▀",
                    top[0], top[1], top[2], bottom[0], bottom[1], bottom[2]
                ));
            }
            buffer.push_str(&format!("{}\n", RESET));
            y += 2;
        }
        print!("{}", buffer);
        let _ = ui;
        println!(
            "{}[ Dica: digite 'janela on' para abrir imagens na janela nativa do sistema ]{}",
            DOS_VERDE, RESET
        );
    }

    pub fn abrir_no_sistema(path: &Path) {
        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("cmd")
                .args(["/C", "start", "", &path.to_string_lossy()])
                .spawn();
        }
        #[cfg(target_os = "linux")]
        {
            let _ = Command::new("xdg-open").arg(path).spawn();
        }
    }
}