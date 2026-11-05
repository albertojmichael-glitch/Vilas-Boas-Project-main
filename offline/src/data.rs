use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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

pub fn chaves_nao_saida() -> [&'static str; 6] {
    [
        "descrição",
        "itens",
        "inspecionaveis",
        "energia",
        "cofre_important",
        "cadeira",
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Sala {
    #[serde(default, rename = "descrição")]
    pub descricao: String,
    #[serde(default)]
    pub itens: Vec<String>,
    #[serde(default)]
    pub inspecionaveis: HashMap<String, String>,
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

pub fn descricoes_itens() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert(
        "lanterna".to_string(),
        "Uma lanterna pesada de metal. Essencial para enxergar no escuro.".to_string(),
    );
    m.insert(
        "bateria nova".to_string(),
        "Pilha alcalina industrial. Recarrega a lanterna (+15 turnos).".to_string(),
    );
    m.insert(
        "bateria".to_string(),
        "Bateria reserva para a lanterna.".to_string(),
    );
    m.insert(
        "chave de fenda".to_string(),
        "Ferramenta com cabo isolado. Útil para grades de ventilação ou painéis.".to_string(),
    );
    m.insert(
        "cartao de seguranca".to_string(),
        "Credencial magnética de nível 2. Libera o terminal da cadeira na sala 01.".to_string(),
    );
    m.insert(
        "moeda velha".to_string(),
        "Uma ficha oxidada de fliperama dos anos 80.".to_string(),
    );
    m.insert(
        "partitura rasgada".to_string(),
        "Notas musicais manchadas. Necessária na máquina de Julgamento.".to_string(),
    );
    m.insert(
        "isqueiro".to_string(),
        "Isqueiro fluido metálico. Pode iniciar um incêndio se houver combustível.".to_string(),
    );
    m.insert(
        "galao de gasolina".to_string(),
        "Recipiente cheio de combustível inflamável.".to_string(),
    );
    m.insert(
        "alicate".to_string(),
        "Alicate de corte. Ideal para cortar fios de alta tensão.".to_string(),
    );
    m.insert(
        "chave mestra".to_string(),
        "Chave pesada que destranca a saída principal no Hall de Entrada.".to_string(),
    );
    m.insert(
        "bolsa".to_string(),
        "Bolsa lateral que expande a capacidade da sua mochila (+3 espaços).".to_string(),
    );
    m
}

pub fn carregar_mapa() -> HashMap<String, Sala> {
    let possiveis_caminhos = [
        "data/game_data.json",
        "../server/data/game_data.json",
        "server/data/game_data.json",
    ];

    for caminho in &possiveis_caminhos {
        if Path::new(caminho).exists() {
            if let Ok(conteudo) = fs::read_to_string(caminho) {
                if let Ok(json) = serde_json::from_str::<Value>(&conteudo) {
                    if let Some(mapa_val) = json.get("mapa").or(Some(&json)) {
                        if let Ok(mapa) =
                            serde_json::from_value::<HashMap<String, Sala>>(mapa_val.clone())
                        {
                            if !mapa.is_empty() {
                                return mapa;
                            }
                        }
                    }
                }
            }
        }
    }

    mapa_padrao_embutido()
}

fn mapa_padrao_embutido() -> HashMap<String, Sala> {
    let raw = r#"{
        "01": {
            "descrição": "Sala de Monitoramento 01. Monitores quebrados cercam uma cadeira giratória e um cofre de ferro no canto.",
            "frente": "entrada",
            "direita": "sala de energia",
            "esquerda": "tubo de ventilação",
            "itens": ["bateria nova"],
            "inspecionaveis": {
                "monitores": "As telas piscam com estática. Uma nota diz: 'Use a cadeira com o Cartão de Segurança'.",
                "cofre": "Um cofre antigo. Digite 'abrir cofre' para tentar a senha."
            }
        },
        "entrada": {
            "descrição": "Corredor de Entrada. O ar cheira a mofo e óleo queimado. Há pegadas escuras no piso.",
            "atrás": "01",
            "frente": "sala de jantar",
            "esquerda": "hall de entrada",
            "direita": "quarto de refrigeração",
            "itens": ["moeda velha"],
            "inspecionaveis": {
                "pegadas": "Marcas pesadas de metal arrastado indo em direção à sala de jantar."
            }
        },
        "sala de jantar": {
            "descrição": "Salão de Jantar Principal. Mesas longas cobertas com toalhas rasgadas e pratos sujos.",
            "atrás": "entrada",
            "frente": "palco",
            "direita": "cozinha principal",
            "esquerda": "sala de fliperamas",
            "itens": ["bolsa"],
            "inspecionaveis": {
                "mesas": "Há nove marcas de unha cravadas duas vezes na lateral metálica."
            }
        },
        "palco": {
            "descrição": "O Palco Central. Cortinas vermelhas pesadas e holofotes apagados. Os animatrônicos costumavam ficar aqui.",
            "atrás": "sala de jantar",
            "esquerda": "sala do piano",
            "itens": ["cartao de seguranca"],
            "inspecionaveis": {
                "cortinas": "Atrás das cortinas há cabos soltos e marcas de garras."
            }
        },
        "sala do piano": {
            "descrição": "Sala do Piano. Um piano de cauda desafinado cercado por quatro cadeiras; apenas uma está intacta.",
            "direita": "palco",
            "itens": ["partitura rasgada"],
            "inspecionaveis": {
                "piano": "Sobre as teclas há uma partitura rasgada e o número 1 entalhado na cadeira vazia."
            }
        },
        "cozinha principal": {
            "descrição": "Cozinha Principal. Panelas enferrujadas, fogões industriais e um cheiro forte de gás.",
            "esquerda": "sala de jantar",
            "itens": ["isqueiro", "alicate"],
            "inspecionaveis": {
                "fogao": "Tubulações antigas vazando resquícios inflamáveis."
            }
        },
        "quarto de refrigeração": {
            "descrição": "Quarto de Refrigeração. Ganchos pendurados no teto e uma névoa gélida.",
            "esquerda": "entrada",
            "itens": ["galao de gasolina"],
            "inspecionaveis": {
                "ganchos": "Frios como gelo. Algo pesado esteve pendurado aqui recentemente."
            }
        },
        "sala de fliperamas": {
            "descrição": "Sala de Fliperamas. Três máquinas ainda brilham na penumbra: 'Fome de Jon', 'Consertos' e 'Julgamento'.",
            "direita": "sala de jantar",
            "itens": ["bateria nova"],
            "inspecionaveis": {
                "maquinas": "Comandos disponíveis: 'jogar jon', 'jogar consertos' (exige moeda) e 'jogar adivinha' (exige partitura)."
            }
        },
        "sala de energia": {
            "descrição": "Sala de Energia. O gerador mestre zumbe alto. O terminal do Labirinto do Minotauro controla a tensão.",
            "esquerda": "01",
            "itens": [],
            "inspecionaveis": {
                "gerador": "Os fios principais da instalação passam por aqui."
            }
        },
        "tubo de ventilação": {
            "descrição": "Tubo de Ventilação estreito e claustrofóbico. Liga a sala 01 ao Hall de Entrada.",
            "direita": "01",
            "frente": "hall de entrada",
            "itens": ["chave de fenda"],
            "inspecionaveis": {}
        },
        "hall de entrada": {
            "descrição": "Hall de Entrada da Unidade. A grande porta dupla de aço leva para fora do complexo.",
            "direita": "entrada",
            "atrás": "tubo de ventilação",
            "frente": "saida",
            "itens": [],
            "inspecionaveis": {
                "porta": "Pesada e trancada eletronicamente até que a noite seja vencida ou destrancada."
            }
        },
        "morte": {
            "descrição": "Fim da linha.",
            "itens": [],
            "inspecionaveis": {}
        },
        "saida": {
            "descrição": "O ar frio da madrugada finalmente toca o seu rosto.",
            "itens": [],
            "inspecionaveis": {}
        },
        "cama": {
            "descrição": "Você fechou os olhos tentando ignorar o pesadelo.",
            "itens": [],
            "inspecionaveis": {}
        },
        "final_bom": {
            "descrição": "O silêncio absoluto.",
            "itens": [],
            "inspecionaveis": {}
        }
    }"#;

    serde_json::from_str(raw).unwrap_or_default()
}