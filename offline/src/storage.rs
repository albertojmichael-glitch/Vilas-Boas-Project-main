use serde::Serialize;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[cfg(test)]
thread_local! {
    static PASTA_DE_TESTE: tempfile::TempDir = tempfile::tempdir().expect("pasta de testes");
}

pub fn pasta() -> PathBuf {
    if cfg!(test) {
        #[cfg(test)]
        return PASTA_DE_TESTE.with(|diretorio| diretorio.path().to_path_buf());
    }
    if let Some(overridden) = env::var_os("VILAS_BOAS_DATA_DIR").filter(|s| !s.is_empty()) {
        return PathBuf::from(overridden);
    }
    let base = if cfg!(windows) {
        env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else {
        env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
    };
    base.unwrap_or_else(|| PathBuf::from(".")).join("VilasBoas")
}

pub fn ler(nome: &str, legados: &[&str]) -> io::Result<String> {
    let atual = pasta().join(nome);
    match fs::read_to_string(atual) {
        Err(erro) if erro.kind() == io::ErrorKind::NotFound => legados
            .iter()
            .find_map(|p| fs::read_to_string(p).ok())
            .ok_or(erro),
        result => result,
    }
}

pub fn gravar(nome: &str, registro: &impl Serialize) -> io::Result<()> {
    gravar_em(&pasta(), nome, registro)
}

fn gravar_em(pasta: &Path, nome: &str, registro: &impl Serialize) -> io::Result<()> {
    fs::create_dir_all(pasta)?;
    let mut arquivo = tempfile::NamedTempFile::new_in(pasta)?;
    serde_json::to_writer_pretty(arquivo.as_file_mut(), registro)?;
    arquivo.as_file_mut().flush()?;
    arquivo.as_file().sync_all()?;
    arquivo.persist(pasta.join(nome))?;
    Ok(())
}

pub fn apagar(nome: &str, legados: &[&str]) -> io::Result<()> {
    for caminho in legados
        .iter()
        .map(PathBuf::from)
        .chain([pasta().join(nome)])
    {
        match fs::remove_file(caminho) {
            Ok(()) => {}
            Err(erro) if erro.kind() == io::ErrorKind::NotFound => {}
            Err(erro) => return Err(erro),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitui_arquivo_depois_de_serializar() {
        struct Falha;
        impl Serialize for Falha {
            fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
                Err(serde::ser::Error::custom("Falha intencional"))
            }
        }

        let pasta = tempfile::tempdir().unwrap();
        let nome = "save.json";
        let caminho = pasta.path().join(nome);
        gravar_em(pasta.path(), nome, &"partida 1").unwrap();
        assert!(gravar_em(pasta.path(), nome, &Falha).is_err());
        assert_eq!(fs::read_to_string(&caminho).unwrap(), "\"partida 1\"");
        gravar_em(pasta.path(), nome, &"partida 2").unwrap();
        assert_eq!(fs::read_to_string(caminho).unwrap(), "\"partida 2\"");
    }
}
