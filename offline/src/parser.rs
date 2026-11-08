use std::collections::HashMap;

pub fn remover_acentos(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' | 'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' | 'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => 'u',
            'ç' | 'Ç' => 'c',
            _ => c,
        })
        .collect()
}

pub fn normalizar(entrada: &str) -> String {
    let limpo: String = remover_acentos(entrada)
        .to_lowercase()
        .chars()
        .filter(|c| !matches!(c, '.' | ',' | '!' | '?' | '"' | '\''))
        .collect();
    limpo.split_whitespace().collect::<Vec<&str>>().join(" ")
}

pub fn atalho_direcao(cmd: &str) -> Option<&'static str> {
    match cmd {
        "f" | "frente" | "n" | "norte" => Some("ir frente"),
        "t" | "tras" | "atras" | "atrás" | "s" | "sul" => Some("ir atrás"),
        "e" | "esquerda" | "w" | "oeste" => Some("ir esquerda"),
        "d" | "direita" | "leste" => Some("ir direita"),
        _ => None,
    }
}

pub fn expandir_atalho_direcao(cmd: &str, sinonimos: &HashMap<String, String>) -> String {
    if let Some(mapped) = sinonimos.get(cmd) {
        return mapped.clone();
    }
    if let Some(at) = atalho_direcao(cmd) {
        return at.to_string();
    }
    match cmd {
        "ir t" | "ir tras" | "ir atras" => "ir atrás".to_string(),
        "ir f" => "ir frente".to_string(),
        "ir e" => "ir esquerda".to_string(),
        "ir d" => "ir direita".to_string(),
        _ => cmd.to_string(),
    }
}
