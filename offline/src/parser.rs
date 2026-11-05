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
    let sem_acento = remover_acentos(entrada).to_lowercase();
    sem_acento
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

pub fn expandir_atalho_direcao(cmd: &str) -> String {
    match cmd {
        "f" | "frente" | "w" | "norte" => "ir frente".to_string(),
        "t" | "atras" | "s" | "sul" => "ir atrás".to_string(),
        "e" | "esquerda" | "a" | "oeste" => "ir esquerda".to_string(),
        "d" | "direita" | "leste" => "ir direita".to_string(),
        "ir t" | "ir atras" => "ir atrás".to_string(),
        "ir f" => "ir frente".to_string(),
        "ir e" => "ir esquerda".to_string(),
        "ir d" => "ir direita".to_string(),
        _ => cmd.to_string(),
    }
}