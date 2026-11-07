use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const SERVER_BUNDLE_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/server_bundle.json"));
const EMBEDDED_GAME_DATA: &str = include_str!("../../server/data/game_data.json");

pub const VIDA_NORMAL: i32 = 3;
pub const TURNOS_BATERIA: i32 = 15;
pub const LIMIAR_ALERTA: i32 = 40;
pub const LIMIAR_CACA: i32 = 80;
pub const CHANCE_MOVER_PATRULHA: f64 = 0.35;
pub const MAX_TENTATIVAS_COFRE: i32 = 3;
pub const LIMITE_ERROS_DICA: i32 = 3;
pub const BARULHO_ERROS_CONSECUTIVOS: i32 = 30;
pub const SENHA_COFRE: &str = "1983";

pub fn salas_terminais() -> [&'static str; 4] {
    ["morte", "saida", "cama", "final_bom"]
}

pub fn salas_seguras_patrulha() -> [&'static str; 7] {
    [
        "morte",
        "saida",
        "cama",
        "final_bom",
        "01",
        "sala de energia",
        "tubo de ventilação",
    ]
}

pub fn chaves_nao_saida() -> [&'static str; 8] {
    [
        "descrição",
        "descricao",
        "itens",
        "inspecionaveis",
        "energia",
        "cofre_important",
        "cadeira",
        "imagem",
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConquistaDef {
    #[serde(default)]
    pub id: String,
    #[serde(default, alias = "titulo", alias = "title", alias = "name")]
    pub nome: String,
    #[serde(default, alias = "description", alias = "desc")]
    pub descricao: String,
    #[serde(default, alias = "icon")]
    pub icone: String,
    #[serde(default, alias = "secret", alias = "oculta")]
    pub secreta: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Sala {
    #[serde(default, rename = "descrição", alias = "descricao")]
    pub descricao: String,
    #[serde(default)]
    pub itens: Vec<String>,
    #[serde(default)]
    pub inspecionaveis: HashMap<String, String>,
    #[serde(default)]
    pub energia: Option<Value>,
    #[serde(default)]
    pub cofre_important: Option<Value>,
    #[serde(default)]
    pub cadeira: Option<Value>,
    #[serde(default)]
    pub imagem: Option<String>,
    #[serde(flatten)]
    pub conexoes: HashMap<String, Value>,
}

impl Sala {
    pub fn obter_saida(&self, direcao: &str) -> Option<String> {
        self.conexoes
            .get(direcao)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    pub fn saidas_validas(&self, mapa: &HashMap<String, Sala>) -> Vec<(String, String)> {
        let ignorar = chaves_nao_saida();
        let mut res = Vec::new();
        for (k, v) in &self.conexoes {
            if ignorar.contains(&k.as_str()) {
                continue;
            }
            if let Some(dest) = v.as_str() {
                if mapa.contains_key(dest) {
                    res.push((k.clone(), dest.to_string()));
                }
            }
        }
        res.sort_by(|a, b| a.0.cmp(&b.0));
        res
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DadosJogo {
    #[serde(default)]
    pub mapa: HashMap<String, Sala>,
    #[serde(default)]
    pub descricoes_itens: HashMap<String, String>,
    #[serde(default)]
    pub sinonimos: HashMap<String, String>,
    #[serde(default)]
    pub ascii_arts: HashMap<String, String>,
    #[serde(default)]
    pub conquistas: Vec<ConquistaDef>,
}

impl DadosJogo {
    pub fn carregar_tudo() -> Self {
        let mut dados = Self::default();

        if let Ok(b) = serde_json::from_str::<DadosJogo>(SERVER_BUNDLE_JSON) {
            dados = b;
        }

        Self::mesclar_json_str(&mut dados, EMBEDDED_GAME_DATA);

        let pastas_data = [
            PathBuf::from("data"),
            PathBuf::from("../data"),
            PathBuf::from("../server/data"),
            PathBuf::from("../server/villas_boas/data"),
            PathBuf::from("../server/vilas_boas/data"),
            PathBuf::from("server/data"),
        ];

        for pasta in &pastas_data {
            if pasta.is_dir() {
                if let Ok(entradas) = fs::read_dir(pasta) {
                    for entrada in entradas.flatten() {
                        let caminho = entrada.path();
                        if caminho.extension().and_then(|e| e.to_str()) == Some("json") {
                            if let Ok(conteudo) = fs::read_to_string(&caminho) {
                                Self::mesclar_json_str(&mut dados, &conteudo);
                            }
                        }
                    }
                }
            }
        }

        dados.preencher_padroes_se_vazio();
        dados
    }

    pub fn carregar_da_pasta_data() -> Self {
        Self::carregar_tudo()
    }

    fn mesclar_json_str(dados: &mut DadosJogo, raw_json: &str) {
        let Ok(val) = serde_json::from_str::<Value>(raw_json) else {
            return;
        };

        if let Some(mapa_obj) = val.get("mapa") {
            if let Ok(m) = serde_json::from_value::<HashMap<String, Sala>>(mapa_obj.clone()) {
                for (k, v) in m {
                    dados.mapa.insert(k, v);
                }
            }
        } else if let Some(obj) = val.as_object() {
            let parece_mapa = obj
                .values()
                .any(|v| v.get("descrição").is_some() || v.get("descricao").is_some());
            if parece_mapa {
                for (k, v) in obj {
                    if let Ok(sala) = serde_json::from_value::<Sala>(v.clone()) {
                        if !sala.descricao.is_empty() || !sala.conexoes.is_empty() {
                            dados.mapa.insert(k.clone(), sala);
                        }
                    }
                }
            }
        }

        for chave_itens in ["descricoes_itens", "itens_descricoes", "itens_info"] {
            if let Some(itens_val) = val.get(chave_itens) {
                if let Ok(m) = serde_json::from_value::<HashMap<String, String>>(itens_val.clone())
                {
                    dados.descricoes_itens.extend(m);
                }
            }
        }

        if let Some(sin_val) = val.get("sinonimos") {
            if let Ok(m) = serde_json::from_value::<HashMap<String, String>>(sin_val.clone()) {
                dados.sinonimos.extend(m);
            }
        }
    }

    fn preencher_padroes_se_vazio(&mut self) {
        self.ascii_arts
            .entry("ARTE_COFRE".to_string())
            .or_insert_with(|| {
                r#"
  __________________________
 /  ______________________  \
|  |  __   __   __   __   |  |
|  | |  | |  | |  | |  |  |  |
|  | |__| |__| |__| |__|  |  |
|  |                      |  |
|  |       .------.       |  |
|  |      /        \      |  |
|  |     |   [**]   |     |  |
|  |      \        /      |  |
|  |       `------'       |  |
|  |                      |  |
|  |______________________|  |
 \__________________________/
"#
                .to_string()
            });

        self.ascii_arts
            .entry("ARTE_COFRE_TENSO".to_string())
            .or_insert_with(|| {
                r#"
  __________________________
 /  ______________/\_______  \
|  |  __   __   __\/  __   |  |
|  | |  | |  | |  |  |  |  |  |
|  | |__| |__|/|__|  |__|  |  |
|  |         /            |  |
|  |       .------.       |  |
|  |      /   !!   \      |  |
|  |     | x [**] x |     |  |
|  |      \        /      |  |
|  |       `------'       |  |
|  |                      |  |
|  |______________________|  |
 \__________________________/
"#
                .to_string()
            });
    }
}

pub fn localizar_pastas_assets(tipo: &str) -> Vec<PathBuf> {
    let subpastas: Vec<&str> = match tipo {
        "audio" => vec!["audio", "static/audio"],
        "secret_images" => vec![
            "secret_images",
            "secret images",
            "static/secret_images",
            "static/secret images",
        ],
        "images" => vec!["images", "static/images", "img"],
        "todas_imagens" => vec![
            "images",
            "static/images",
            "secret_images",
            "secret images",
            "static/secret_images",
        ],
        outro => vec![outro],
    };

    let prefixos = [
        "assets",
        "../client/assets",
        "client/assets",
        "../client",
        "client",
        "../server",
        "../server/villas_boas",
        "../server/vilas_boas",
        ".",
    ];

    let mut caminhos = Vec::new();
    for pref in &prefixos {
        for sub in &subpastas {
            let p = Path::new(pref).join(sub);
            if p.exists() && !caminhos.contains(&p) {
                caminhos.push(p);
            }
        }
    }
    caminhos
}

pub fn resolver_arquivo_asset(tipo: &str, nome: &str) -> Option<PathBuf> {
    for dir in localizar_pastas_assets(tipo) {
        let exato = dir.join(nome);
        if exato.is_file() {
            return Some(exato);
        }
    }
    let direto = Path::new(nome);
    if direto.is_file() {
        return Some(direto.to_path_buf());
    }
    None
}