use crate::data::localizar_pastas_assets;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct ImagemAtiva {
    pub titulo: String,
    pub caminho: PathBuf,
    pub secreta: bool,
    pub eh_jumpscare: bool,
}

pub struct MediaManager {
    pub imagem_atual: Option<ImagemAtiva>,
    pub flash_ticks: u8,
}

impl Default for MediaManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaManager {
    pub fn new() -> Self {
        Self {
            imagem_atual: None,
            flash_ticks: 0,
        }
    }

    pub fn listar_arquivos(secretas: bool) -> Vec<(String, PathBuf)> {
        let tipo = if secretas { "secret_images" } else { "images" };
        let mut encontrados = Vec::new();
        let extensoes = ["png", "jpg", "jpeg", "webp", "bmp", "gif"];

        for pasta in localizar_pastas_assets(tipo) {
            if let Ok(entries) = fs::read_dir(pasta) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(nome) = path.file_name().and_then(|n| n.to_str()) {
                            let lower = nome.to_lowercase();
                            if extensoes.iter().any(|ext| lower.ends_with(ext))
                                && !encontrados.iter().any(|(n, _)| n == nome)
                            {
                                encontrados.push((nome.to_string(), path));
                            }
                        }
                    }
                }
            }
        }
        encontrados.sort_by(|a, b| a.0.cmp(&b.0));
        encontrados
    }

    pub fn buscar_imagem(termo: &str, priorizar_secreta: bool) -> Option<(PathBuf, bool)> {
        let busca = termo.trim().to_lowercase();
        if busca.is_empty() {
            return None;
        }
        let ordens = if priorizar_secreta {
            [true, false]
        } else {
            [false, true]
        };

        for secreta in ordens {
            for (nome, path) in Self::listar_arquivos(secreta) {
                let nome_sem_ext = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&nome)
                    .to_lowercase();
                if nome.to_lowercase() == busca || nome_sem_ext == busca {
                    return Some((path, secreta));
                }
            }
        }

        for secreta in ordens {
            for (nome, path) in Self::listar_arquivos(secreta) {
                let nome_sem_ext = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&nome)
                    .to_lowercase();
                if nome_sem_ext.contains(&busca) || busca.contains(&nome_sem_ext) {
                    return Some((path, secreta));
                }
            }
        }
        None
    }

    pub fn acionar_imagem(
        &mut self,
        termo: &str,
        priorizar_secreta: bool,
        eh_jumpscare: bool,
    ) -> bool {
        if let Some((caminho, secreta)) = Self::buscar_imagem(termo, priorizar_secreta) {
            let titulo = caminho
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(termo)
                .to_string();
            self.imagem_atual = Some(ImagemAtiva {
                titulo,
                caminho,
                secreta,
                eh_jumpscare,
            });
            if eh_jumpscare {
                self.flash_ticks = 12;
            }
            return true;
        }
        false
    }
}