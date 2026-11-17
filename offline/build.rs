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
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    fs::write(out.join("balance.rs"), output).expect("escrever balance.rs");
}
