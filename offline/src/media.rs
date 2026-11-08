use crate::data::{
    localizar_pastas_assets, obter_imagem_embutida, IMAGENS_EMBUTIDAS, IMAGENS_SECRETAS_EMBUTIDAS,
};
use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct ImagemAtiva {
    pub titulo: String,
    pub chave_embutida: Option<String>,
    pub caminho: Option<PathBuf>,
    pub secreta: bool,
    pub eh_jumpscare: bool,
}

pub struct MediaManager {
    pub imagem_atual: Option<ImagemAtiva>,
    pub flash_ticks: u8,
    texturas_cache: HashMap<String, TextureHandle>,
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
            texturas_cache: HashMap::new(),
        }
    }

    /// Carrega (e mantém em cache) uma textura `egui::TextureHandle` a partir dos bytes embutidos ou disco.
    pub fn obter_textura(&mut self, ctx: &egui::Context, nome: &str) -> Option<TextureHandle> {
        let chave = nome.to_lowercase();
        if let Some(tex) = self.texturas_cache.get(&chave) {
            return Some(tex.clone());
        }

        if let Some(bytes) = obter_imagem_embutida(nome) {
            if let Ok(img) = image::load_from_memory(bytes) {
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.as_flat_samples();
                let color_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                let handle = ctx.load_texture(&chave, color_image, TextureOptions::LINEAR);
                self.texturas_cache.insert(chave, handle.clone());
                return Some(handle);
            }
        }

        if let Some((path, _)) = Self::buscar_imagem_disco(nome, false) {
            if let Ok(bytes) = fs::read(&path) {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    let rgba = img.to_rgba8();
                    let size = [rgba.width() as usize, rgba.height() as usize];
                    let pixels = rgba.as_flat_samples();
                    let color_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                    let handle = ctx.load_texture(&chave, color_image, TextureOptions::LINEAR);
                    self.texturas_cache.insert(chave, handle.clone());
                    return Some(handle);
                }
            }
        }

        None
    }

    pub fn listar_arquivos_disco(secretas: bool) -> Vec<(String, PathBuf)> {
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

    pub fn buscar_imagem_disco(termo: &str, priorizar_secreta: bool) -> Option<(PathBuf, bool)> {
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
            for (nome, path) in Self::listar_arquivos_disco(secreta) {
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
            for (nome, path) in Self::listar_arquivos_disco(secreta) {
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

    pub fn buscar_embutida(termo: &str, priorizar_secreta: bool) -> Option<(String, bool)> {
        let busca = termo.trim().to_lowercase();
        if busca.is_empty() {
            return None;
        }

        let listas: [(&[(&str, &[u8])], bool); 2] = if priorizar_secreta {
            [(IMAGENS_SECRETAS_EMBUTIDAS, true), (IMAGENS_EMBUTIDAS, false)]
        } else {
            [(IMAGENS_EMBUTIDAS, false), (IMAGENS_SECRETAS_EMBUTIDAS, true)]
        };

        for (lista, secreta) in listas {
            for (nome, _) in lista {
                let lower = nome.to_lowercase();
                let stem = lower.split('.').next().unwrap_or(&lower);
                if lower == busca || stem == busca {
                    return Some(((*nome).to_string(), secreta));
                }
            }
        }

        for (lista, secreta) in listas {
            for (nome, _) in lista {
                let lower = nome.to_lowercase();
                let stem = lower.split('.').next().unwrap_or(&lower);
                if stem.contains(&busca) || busca.contains(stem) {
                    return Some(((*nome).to_string(), secreta));
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
        if let Some((nome_emb, secreta)) = Self::buscar_embutida(termo, priorizar_secreta) {
            self.imagem_atual = Some(ImagemAtiva {
                titulo: nome_emb.clone(),
                chave_embutida: Some(nome_emb),
                caminho: None,
                secreta,
                eh_jumpscare,
            });
            if eh_jumpscare {
                self.flash_ticks = 12;
            }
            return true;
        }

        if let Some((caminho, secreta)) = Self::buscar_imagem_disco(termo, priorizar_secreta) {
            let titulo = caminho
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(termo)
                .to_string();
            self.imagem_atual = Some(ImagemAtiva {
                titulo,
                chave_embutida: None,
                caminho: Some(caminho),
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
