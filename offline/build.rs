use serde_json::Value;
use std::{env, fmt::Write as _, fs, path::PathBuf};

fn main() {
    const BALANCE: &str = "../server/data/balance_config.json";
    const GAME: &str = "../server/data/game_data.json";
    println!("cargo:rerun-if-changed={BALANCE}");
    println!("cargo:rerun-if-changed={GAME}");

    let balance: Value = serde_json::from_slice(&fs::read(BALANCE).expect(BALANCE))
        .expect("JSON de balanceamento inválido");
    let game: Value =
        serde_json::from_slice(&fs::read(GAME).expect(GAME)).expect("JSON de jogo inválido");
    let overrides = [
        ("MAX_INVENTARIO", "max_inventario"),
        ("VIDA_NORMAL", "vida_normal"),
        ("VIDA_PESADELO", "vida_pesadelo"),
        ("TURNOS_BATERIA", "turnos_bateria"),
        ("COFRE_SENHA", "cofre_senha"),
    ];
    let mut output = String::new();
    for (group, key, name, ty) in [
        ("inventario", "max_inicial", "MAX_INVENTARIO", "usize"),
        ("inventario", "slots_por_bolsa", "SLOTS_POR_BOLSA", "usize"),
        (
            "inventario",
            "meta_conquista_acumulador",
            "META_ACUMULADOR",
            "usize",
        ),
        ("status_jogador", "vida_normal", "VIDA_NORMAL", "i32"),
        ("status_jogador", "vida_pesadelo", "VIDA_PESADELO", "i32"),
        ("status_jogador", "vida_god_mode", "VIDA_GOD_MODE", "i32"),
        ("status_jogador", "turnos_bateria", "TURNOS_BATERIA", "i32"),
        (
            "status_jogador",
            "turnos_luz_inicial",
            "TURNOS_LUZ_INICIAL",
            "i32",
        ),
        (
            "status_jogador",
            "isqueiro_turnos_bonus",
            "ISQUEIRO_TURNOS_BONUS",
            "i32",
        ),
        (
            "status_jogador",
            "isqueiro_limite_luz_uso",
            "ISQUEIRO_LIMITE_LUZ",
            "i32",
        ),
        (
            "status_jogador",
            "isqueiro_barulho_extra",
            "ISQUEIRO_BARULHO",
            "i32",
        ),
        (
            "barulho_e_ia",
            "reducao_passiva_turno",
            "REDUCAO_BARULHO_TURNO",
            "i32",
        ),
        ("barulho_e_ia", "limiar_caca", "LIMIAR_CACA", "i32"),
        ("barulho_e_ia", "limiar_alerta", "LIMIAR_ALERTA", "i32"),
        (
            "barulho_e_ia",
            "chance_mover_patrulha",
            "CHANCE_MOVER_PATRULHA",
            "f64",
        ),
        ("barulho_e_ia", "barulho_correr", "BARULHO_CORRER", "i32"),
        (
            "barulho_e_ia",
            "turnos_enjoado_correr",
            "TURNOS_ENJOADO_CORRER",
            "i32",
        ),
        (
            "barulho_e_ia",
            "chance_perder_item_correr",
            "CHANCE_PERDER_ITEM_CORRER",
            "f64",
        ),
        (
            "barulho_e_ia",
            "barulho_ir_abrir",
            "BARULHO_IR_ABRIR",
            "i32",
        ),
        (
            "barulho_e_ia",
            "reducao_olhar_esperar",
            "REDUCAO_OLHAR_ESPERAR",
            "i32",
        ),
        (
            "barulho_e_ia",
            "barulho_acao_comum",
            "BARULHO_ACAO_COMUM",
            "i32",
        ),
        (
            "barulho_e_ia",
            "barulho_erros_consecutivos",
            "BARULHO_ERROS_CONSECUTIVOS",
            "i32",
        ),
        (
            "barulho_e_ia",
            "limite_erros_dica",
            "LIMITE_ERROS_DICA",
            "i32",
        ),
        (
            "barulho_e_ia",
            "limite_barulho_desafio_fantasma",
            "LIMITE_BARULHO_FANTASMA",
            "i32",
        ),
        (
            "exploracao",
            "chance_tropeco_escuro_pct",
            "CHANCE_TROPECO_ESCURO_PCT",
            "i32",
        ),
        (
            "exploracao",
            "chance_evento_aleatorio",
            "CHANCE_EVENTO_ALEATORIO",
            "f64",
        ),
        ("exploracao", "cofre_senha", "COFRE_SENHA", "&str"),
        (
            "exploracao",
            "max_tentativas_cofre",
            "MAX_TENTATIVAS_COFRE",
            "i32",
        ),
        (
            "exploracao",
            "tempo_speedrun_verdadeiro_seg",
            "TEMPO_SPEEDRUN_VERDADEIRO",
            "f64",
        ),
        (
            "exploracao",
            "tempo_speedrun_neutro_seg",
            "TEMPO_SPEEDRUN_NEUTRO",
            "f64",
        ),
    ] {
        let value = overrides
            .iter()
            .find(|(constant, _)| *constant == name)
            .and_then(|(_, field)| game["constantes"].get(field))
            .or_else(|| balance[group].get(key))
            .unwrap_or_else(|| panic!("Configuração ausente: {group}.{key}"));
        // Atribuir o literal a um tipo Rust também valida o valor na compilação.
        writeln!(output, "pub const {name}: {ty} = {value};").expect("escrever constante");
    }
    // Regras novas de balanceamento (issue #7): esconderijos, tropeço e a "Ela".
    let exploracao = &balance["exploracao"];
    let ia = &balance["barulho_e_ia"];
    let texto = |v: &Value, chave: &str| -> String {
        v.get(chave)
            .unwrap_or_else(|| panic!("Configuração ausente: {chave}"))
            .to_string()
    };
    let lista = |v: &Value, chave: &str| -> String {
        let itens: Vec<String> = v[chave]
            .as_array()
            .unwrap_or_else(|| panic!("Lista ausente: {chave}"))
            .iter()
            .map(|s| format!("{:?}", s.as_str().expect("nome de sala")))
            .collect();
        format!("&[{}]", itens.join(", "))
    };
    let real = |v: &Value, chave: &str| -> String {
        format!(
            "{:?}",
            v[chave]
                .as_f64()
                .unwrap_or_else(|| panic!("Número ausente: {chave}"))
        )
    };
    writeln!(
        output,
        "pub const TROPECO_PODE_MATAR: bool = {};",
        texto(exploracao, "tropeco_pode_matar")
    )
    .expect("escrever constante");
    for (chave, nome) in [
        ("barulho_tropeco_nao_letal", "BARULHO_TROPECO_NAO_LETAL"),
        ("max_esconder_seguidos", "MAX_ESCONDER_SEGUIDOS"),
    ] {
        writeln!(
            output,
            "pub const {nome}: i32 = {};",
            texto(exploracao, chave)
        )
        .expect("escrever constante");
    }
    writeln!(
        output,
        "pub const SALAS_ESCONDERIJO: &[&str] = {};",
        lista(exploracao, "salas_esconderijo")
    )
    .expect("escrever constante");
    writeln!(
        output,
        "pub const DUTOS_ELA: &[&str] = {};",
        lista(ia, "dutos_ela")
    )
    .expect("escrever constante");
    writeln!(
        output,
        "pub const SALA_INICIAL_ELA: &str = {};",
        texto(ia, "sala_inicial_ela")
    )
    .expect("escrever constante");

    for modo in ["NORMAL", "PESADELO"] {
        let preset = &balance["dificuldades"][modo];
        for (chave, sufixo) in [
            ("furia_noite", "FURIA_NOITE"),
            ("energia_min_noite", "ENERGIA_MIN_NOITE"),
            ("energia_max_noite", "ENERGIA_MAX_NOITE"),
        ] {
            writeln!(
                output,
                "pub const {modo}_{sufixo}: i32 = {};",
                texto(preset, chave)
            )
            .expect("escrever constante");
        }
        let ela = &preset["ela"];
        writeln!(
            output,
            "pub const PERFIL_ELA_{modo}: PerfilEla = PerfilEla {{ \
             turnos_dormencia: {}, chance_mover: {}, chance_rastrear: {}, \
             chance_usar_duto: {}, alcance_duto: {}, turnos_atordoada: {}, \
             dano_fuga: {}, aviso_distancia: {} }};",
            texto(ela, "turnos_dormencia"),
            real(ela, "chance_mover"),
            real(ela, "chance_rastrear"),
            real(ela, "chance_usar_duto"),
            texto(ela, "alcance_duto"),
            texto(ela, "turnos_atordoada"),
            texto(ela, "dano_fuga"),
            texto(ela, "aviso_distancia"),
        )
        .expect("escrever perfil da Ela");
    }

    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    fs::write(out.join("balance.rs"), output).expect("escrever balance.rs");
}
