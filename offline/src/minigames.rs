use crate::audio::AudioManager;
use crate::data::{
    COFRE_SENHA, MAX_TENTATIVAS_COFRE, MSGS_PERIGO_DIREITA, MSGS_PERIGO_ESQUERDA,
    MSGS_PERIGO_ESTATICA, MSGS_PERIGO_FRENTE, MSGS_PERIGO_TRAS,
};
use crate::media::MediaManager;
use crate::state::{CaptchaData, GameState, MinotauroData, SegurancaData};
use crate::ui::{DOS_AMARELO, DOS_BRANCO, DOS_CIANO, DOS_VERDE, DOS_VERMELHO, RESET};
use crate::utils::{corromper_texto, desbloquear_conquista, texto_cintilante};
use crate::views;
use crate::TerminalBuffer;
use rand::seq::SliceRandom;
use rand::Rng;

const PISTAS_COFRE: [&str; 3] = [
    "[1º dígito] Um único ponto de luz acende sozinho na unidade, o primeiro som da noite.",
    "[2º e 3º dígitos] Nove marcas de unha, em 2 vezes, na fileira da lateral do metal.",
    "[4º dígito] Quatro cadeiras na sala do piano. Só uma nunca foi ocupada de novo.",
];

fn entregar_item_ou_chao(
    item: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
) {
    if jogo.inventario.len() < jogo.obter_limite_inventario() {
        jogo.inventario.push(item.to_string());
        ui.animar(
            &format!("Item adicionado à mochila: '{}'.", item),
            DOS_VERDE,
        );
    } else {
        if let Some(sala) = jogo.mapa.get_mut(&jogo.sala_atual) {
            sala.itens.push(item.to_string());
        }
        ui.animar(
            &format!(
                "Mochila cheia! '{}' caiu no chão da sala.",
                item
            ),
            DOS_AMARELO,
        );
    }
}

// =========================================================================
// 1. MINIGAME DO COFRE
// =========================================================================

pub fn imprimir_painel_cofre(
    jogo: &GameState,
    ui: &mut TerminalBuffer,
    media: &mut MediaManager,
    primeira_vez: bool,
) {
    let _ = media.acionar_imagem("cofre", false, false);
    if jogo.cofre_tentativas == 0 {
        if let Some(arte) = jogo.ascii_arts.get("ARTE_COFRE") {
            ui.animar(&format!("{}{}{}", DOS_BRANCO, arte, RESET), DOS_BRANCO);
        }
    } else if let Some(arte) = jogo.ascii_arts.get("ARTE_COFRE_TENSO") {
        ui.animar(&format!("{}{}{}", DOS_VERMELHO, arte, RESET), DOS_VERMELHO);
    }

    if primeira_vez {
        ui.animar(
            "O cofre tem arranhões profundos ao redor do teclado numérico:",
            DOS_AMARELO,
        );
        let mut pistas = PISTAS_COFRE.to_vec();
        pistas.shuffle(&mut rand::thread_rng());
        for p in pistas {
            ui.animar(&format!("  {}", p), DOS_BRANCO);
        }
    }

    ui.exibir(&format!(
        "\n{}Digite a senha de 4 dígitos (ou 'sair' para se afastar):{}",
        DOS_AMARELO, RESET
    ));
}

fn processar_cofre(
    comando: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) {
    if ["sair", "voltar", "cancelar"].contains(&comando) {
        jogo.estado_atual = "JOGO".to_string();
        ui.exibir(&format!("{}Você se afasta do cofre.{}", DOS_BRANCO, RESET));
        views::imprimir_contexto_sala(jogo, ui, media);
        return;
    }

    if comando == COFRE_SENHA {
        ui.limpar();
        audio.som_porta();
        ui.animar(
            "BEEP! A luz verde pisca e a pesada porta do cofre se abre com um rangido.",
            DOS_VERDE,
        );
        jogo.cofre_aberto = true;

        if jogo.cofre_tentativas == 0 {
            desbloquear_conquista("mente_brilhante", jogo, ui, audio);
        }

        entregar_item_ou_chao("chave dos fundos", jogo, ui);

        if let Some(sala) = jogo.mapa.get_mut("01") {
            sala.inspecionaveis.insert(
                "cofre".to_string(),
                "O cofre agora está aberto e vazio.".to_string(),
            );
        }

        jogo.estado_atual = "JOGO".to_string();
        views::imprimir_contexto_sala(jogo, ui, media);
    } else {
        audio.som_metal();
        jogo.cofre_tentativas += 1;
        let restantes = MAX_TENTATIVAS_COFRE - jogo.cofre_tentativas;

        if restantes <= 0 {
            ui.limpar();
            ui.animar(
                "BEEP! BEEP! BEEP! O painel pisca em vermelho e desliga. O mecanismo de segurança travou o cofre permanentemente!",
                DOS_VERMELHO,
            );
            ui.animar(
                "O alarme silencioso disparou um estalo magnético no corredor... Ela sabe onde você está.",
                DOS_VERMELHO,
            );
            jogo.posicao_perseguidor = "perto".to_string();
            jogo.estado_atual = "JOGO".to_string();
            views::imprimir_contexto_sala(jogo, ui, media);
        } else {
            ui.limpar();
            ui.exibir(&format!(
                "{}SENHA INCORRETA! Tentativas restantes antes do bloqueio: {}{}",
                DOS_VERMELHO, restantes, RESET
            ));
            imprimir_painel_cofre(jogo, ui, media, false);
        }
    }
}

// =========================================================================
// 2. MINIGAME: A FOME DE JON
// =========================================================================

pub fn iniciar_minigame_jon(jogo: &mut GameState, ui: &mut TerminalBuffer) {
    jogo.estado_atual = "MINIGAME_JON".to_string();
    let mut rng = rand::thread_rng();
    let opcoes = ["f", "e", "d"];
    jogo.jon_passos = (0..4)
        .map(|_| (*opcoes.choose(&mut rng).unwrap()).to_string())
        .collect();
    jogo.jon_passo_atual = 0;

    ui.limpar();
    if let Some(arte) = jogo.ascii_arts.get("ARTE_PORCO") {
        ui.animar(&format!("{}{}{}", DOS_VERMELHO, arte, RESET), DOS_VERMELHO);
    }
    ui.animar(
        "=== ARCADE: A FOME DE JON (1983) ===",
        DOS_AMARELO,
    );
    ui.animar(
        "Você está nos dutos de ventilação. Escute os sinais sensoriais e escolha a direção OPOSTA ou SEGURA para avançar!",
        DOS_BRANCO,
    );
    ui.animar(
        "Atenção: 'frente' = ar gelado | 'esquerda' = metal arranhando | 'direita' = cheiro podre.",
        DOS_AMARELO,
    );
    ui.exibir(&format!(
        "{}Comandos: 'frente' (f), 'esquerda' (e), 'direita' (d) ou 'sair'.{}",
        DOS_VERDE, RESET
    ));

    let primeiro_passo = jogo.jon_passos[0].clone();
    views::dar_dica_jon(&primeiro_passo, ui);
}

fn processar_jon(
    comando: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) {
    if ["sair", "parar", "desistir"].contains(&comando) {
        jogo.estado_atual = "JOGO".to_string();
        ui.exibir(&format!(
            "{}Você solta os controles enferrujados do fliperama.{}",
            DOS_BRANCO, RESET
        ));
        views::imprimir_contexto_sala(jogo, ui, media);
        return;
    }

    let escolha = match comando {
        "frente" | "f" | "w" => "f",
        "esquerda" | "e" | "a" => "e",
        "direita" | "d" => "d",
        _ => {
            ui.exibir(&format!(
                "{}Use apenas 'frente' (f), 'esquerda' (e), 'direita' (d) ou 'sair'.{}",
                DOS_AMARELO, RESET
            ));
            return;
        }
    };

    let passo_certo = jogo
        .jon_passos
        .get(jogo.jon_passo_atual)
        .cloned()
        .unwrap_or_else(|| "f".to_string());

    if escolha == passo_certo {
        ui.animar(
            "Você avança pelo duto escuro... O som de mastigação fica para trás.",
            DOS_VERDE,
        );
        jogo.jon_passo_atual += 1;

        if jogo.jon_passo_atual >= 4 {
            ui.limpar();
            ui.animar(
                "VITÓRIA! Na tela de 8-bits, o pequeno personagem escapa da tubulação!",
                DOS_VERDE,
            );
            ui.animar(
                "CLACK! O compartimento de fichas do fliperama cospe uma 'moeda velha'!",
                DOS_AMARELO,
            );
            entregar_item_ou_chao("moeda velha", jogo, ui);
            jogo.jon_vencido = true;
            jogo.consumir_luz(1);
            jogo.estado_atual = "JOGO".to_string();
            views::imprimir_contexto_sala(jogo, ui, media);
        } else {
            let prox = jogo.jon_passos[jogo.jon_passo_atual].clone();
            views::dar_dica_jon(&prox, ui);
        }
    } else {
        ui.limpar();
        ui.animar(
            "CRUNCH! Na tela, um porco humanoide gigante abocanha o personagem!",
            DOS_VERMELHO,
        );
        ui.animar(
            "BZZZT! O controle do fliperama dá um choque elétrico violento nas suas mãos! (-1 HP)",
            DOS_VERMELHO,
        );
        jogo.aplicar_dano(1);
        jogo.consumir_luz(1);

        if jogo.hp <= 0 {
            views::dar_tela_de_morte(jogo, ui, audio, media);
        } else {
            jogo.estado_atual = "JOGO".to_string();
            views::imprimir_contexto_sala(jogo, ui, media);
        }
    }
}

// =========================================================================
// 3. MINIGAME: CONSERTOS & SORRISOS
// =========================================================================

pub fn iniciar_minigame_consertos(jogo: &mut GameState, ui: &mut TerminalBuffer) {
    jogo.inventario.retain(|i| i != "moeda velha");
    jogo.consertos_escolhas.clear();
    jogo.estado_atual = "MINIGAME_CONSERTOS_CABECA".to_string();

    ui.limpar();
    if let Some(arte) = jogo.ascii_arts.get("ARTE_ROBO") {
        ui.animar(&format!("{}{}{}", DOS_AMARELO, arte, RESET), DOS_AMARELO);
    }
    ui.animar(
        "Você insere a moeda velha. A máquina 'Consertos & Sorrisos' ganha vida!",
        DOS_VERDE,
    );
    ui.animar(
        "FASE 1/3 - ESCOLHA A CABEÇA DO ANIMATRÔNICO:",
        DOS_AMARELO,
    );
    ui.exibir(&format!(
        "{}[1] Cabeça de Urso\n[2] Cabeça de Coelho\n[3] Cabeça de Pássaro{}",
        DOS_BRANCO, RESET
    ));
}

fn processar_consertos(
    comando: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) {
    if !["1", "2", "3"].contains(&comando) {
        ui.exibir(&format!(
            "{}Escolha uma peça digitando 1, 2 ou 3.{}",
            DOS_AMARELO, RESET
        ));
        return;
    }

    if jogo.estado_atual == "MINIGAME_CONSERTOS_CABECA" {
        jogo.consertos_escolhas
            .insert("cabeca".to_string(), comando.to_string());
        jogo.estado_atual = "MINIGAME_CONSERTOS_TRONCO".to_string();
        ui.animar(
            "\nFASE 2/3 - ESCOLHA O TRONCO DO ANIMATRÔNICO:",
            DOS_AMARELO,
        );
        ui.exibir(&format!(
            "{}[1] Tronco com Gravata Borboleta\n[2] Tronco com Colete Rasgado\n[3] Tronco Enferrujado{}",
            DOS_BRANCO, RESET
        ));
    } else if jogo.estado_atual == "MINIGAME_CONSERTOS_TRONCO" {
        jogo.consertos_escolhas
            .insert("tronco".to_string(), comando.to_string());
        jogo.estado_atual = "MINIGAME_CONSERTOS_PERNAS".to_string();
        ui.animar(
            "\nFASE 3/3 - ESCOLHA AS PERNAS DO ANIMATRÔNICO:",
            DOS_AMARELO,
        );
        ui.exibir(&format!(
            "{}[1] Pernas de Metal Exposto\n[2] Pernas de Pelúcia Azul\n[3] Pernas com Rodinhas{}",
            DOS_BRANCO, RESET
        ));
    } else if jogo.estado_atual == "MINIGAME_CONSERTOS_PERNAS" {
        jogo.consertos_escolhas
            .insert("pernas".to_string(), comando.to_string());

        ui.limpar();
        ui.buffer_efeito("@@JUMPSCARE@@");
        audio.som_grito();

        ui.animar(
            "Na tela do fliperama, o boneco montado abre os olhos vermelhos e encara você fora da tela!",
            DOS_VERMELHO,
        );
        ui.animar(
            "A gaveta de prêmios do arcade se abre com força, derrubando itens!",
            DOS_AMARELO,
        );

        entregar_item_ou_chao("chave da cozinha", jogo, ui);
        entregar_item_ou_chao("partitura rasgada", jogo, ui);

        let cabeca = jogo
            .consertos_escolhas
            .get("cabeca")
            .cloned()
            .unwrap_or_default();
        let pernas = jogo
            .consertos_escolhas
            .get("pernas")
            .cloned()
            .unwrap_or_default();

        if cabeca == "2" && pernas == "2" {
            ui.animar(
                "BÔNUS DE MONTAGEM PERFEITA! Um frasco de 'remedio' também caiu pela abertura!",
                DOS_VERDE,
            );
            entregar_item_ou_chao("remedio", jogo, ui);
        }

        jogo.consertos_vencido = true;
        jogo.consumir_luz(1);
        jogo.estado_atual = "JOGO".to_string();
        views::imprimir_contexto_sala(jogo, ui, media);
    }
}

// =========================================================================
// 4. MINIGAME: O JULGAMENTO DO PIANISTA
// =========================================================================

pub fn iniciar_minigame_julgamento(jogo: &mut GameState, ui: &mut TerminalBuffer) {
    jogo.estado_atual = "MINIGAME_JULGAMENTO_Q1".to_string();
    jogo.julgamento_pontos = 0;
    jogo.vitimas_restantes = vec![
        "angela".to_string(),
        "joao".to_string(),
        "renato".to_string(),
    ];

    ui.limpar();
    if let Some(arte) = jogo.ascii_arts.get("ARTE_PIANO") {
        ui.animar(&format!("{}{}{}", DOS_BRANCO, arte, RESET), DOS_BRANCO);
    }
    ui.animar(
        "Você coloca a 'partitura rasgada' no suporte do piano. As teclas começam a se mover sozinhas...",
        DOS_AMARELO,
    );
    ui.animar(
        "Uma voz sintética e distorcida ecoa de dentro da caixa acústica:",
        DOS_BRANCO,
    );
    ui.animar(
        "\"Responda com a verdade, Rogério... ou junte-se à orquestra.\"",
        DOS_VERMELHO,
    );
    ui.exibir(&format!(
        "\n{}1ª PERGUNTA: Em que ano as portas deste restaurante foram fechadas para sempre após a tragédia?{}",
        DOS_AMARELO, RESET
    ));
}

fn processar_julgamento(
    comando: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) {
    match jogo.estado_atual.as_str() {
        "MINIGAME_JULGAMENTO_Q1" => {
            if comando == "1995" {
                jogo.julgamento_pontos += 1;
                views::falar_pianista(true, ui);
            } else {
                views::falar_pianista(false, ui);
            }
            jogo.estado_atual = "MINIGAME_JULGAMENTO_Q2".to_string();
            ui.exibir(&format!(
                "\n{}2ª PERGUNTA: Qual é o nome DELA? A alma que comanda a frequência principal?{}",
                DOS_AMARELO, RESET
            ));
        }
        "MINIGAME_JULGAMENTO_Q2" => {
            if comando.contains("caroline") || comando == "ela" {
                jogo.julgamento_pontos += 1;
                views::falar_pianista(true, ui);
            } else {
                views::falar_pianista(false, ui);
            }
            jogo.estado_atual = "MINIGAME_JULGAMENTO_Q3".to_string();
            ui.exibir(&format!(
                "\n{}3ª PERGUNTA: Em que ano a Villas-Boas Industries fundou este sistema (olhe para o início de tudo)?{}",
                DOS_AMARELO, RESET
            ));
        }
        "MINIGAME_JULGAMENTO_Q3" => {
            if comando == "1982" {
                jogo.julgamento_pontos += 1;
                views::falar_pianista(true, ui);
            } else {
                views::falar_pianista(false, ui);
            }
            jogo.estado_atual = "MINIGAME_JULGAMENTO_Q4".to_string();
            ui.exibir(&format!(
                "\n{}4ª PERGUNTA: Quem veio buscá-la esta noite?{}",
                DOS_AMARELO, RESET
            ));
        }
        "MINIGAME_JULGAMENTO_Q4" => {
            if comando.contains("rogerio") || comando.contains("eu") {
                jogo.julgamento_pontos += 1;
                views::falar_pianista(true, ui);
            } else {
                views::falar_pianista(false, ui);
            }
            jogo.estado_atual = "MINIGAME_JULGAMENTO_V1".to_string();
            ui.exibir(&format!(
                "\n{}ÚLTIMA PARTE: Diga o nome das outras três vítimas presas nas máquinas (uma por vez).\nPrimeira vítima:{}",
                DOS_AMARELO, RESET
            ));
        }
        "MINIGAME_JULGAMENTO_V1" | "MINIGAME_JULGAMENTO_V2" | "MINIGAME_JULGAMENTO_V3" => {
            let mut encontrado: Option<String> = None;
            for v in &jogo.vitimas_restantes {
                if comando.contains(v) {
                    encontrado = Some(v.clone());
                    break;
                }
            }

            if let Some(vitima) = encontrado {
                jogo.vitimas_restantes.retain(|v| v != &vitima);
                views::falar_pianista(true, ui);
            } else {
                views::falar_pianista(false, ui);
            }

            if jogo.estado_atual == "MINIGAME_JULGAMENTO_V1" {
                jogo.estado_atual = "MINIGAME_JULGAMENTO_V2".to_string();
                ui.exibir(&format!("\n{}Segunda vítima:{}", DOS_AMARELO, RESET));
            } else if jogo.estado_atual == "MINIGAME_JULGAMENTO_V2" {
                jogo.estado_atual = "MINIGAME_JULGAMENTO_V3".to_string();
                ui.exibir(&format!("\n{}Terceira vítima:{}", DOS_AMARELO, RESET));
            } else {
                if jogo.vitimas_restantes.is_empty() {
                    jogo.julgamento_pontos += 1;
                }

                if jogo.julgamento_pontos >= 5 {
                    ui.limpar();
                    ui.animar(
                        "O piano toca um último acorde majestoso e melancólico. A tampa do teclado se tranca sozinha.",
                        DOS_VERDE,
                    );
                    ui.animar(
                        "Um compartimento secreto na lateral de madeira se abre!",
                        DOS_VERDE,
                    );
                    jogo.inventario.retain(|i| i != "partitura rasgada");
                    entregar_item_ou_chao("cartao de seguranca", jogo, ui);
                    entregar_item_ou_chao("bateria nova", jogo, ui);
                    jogo.julgamento_vencido = true;
                    jogo.consumir_luz(1);
                    jogo.estado_atual = "JOGO".to_string();
                    views::imprimir_contexto_sala(jogo, ui, media);
                } else {
                    ui.buffer_efeito("@@JUMPSCARE@@");
                    audio.som_grito();
                    views::dar_tela_kernel_panic(jogo, ui);
                }
            }
        }
        _ => {}
    }
}

// =========================================================================
// 5. MINIGAME: LABIRINTO DO MINOTAURO (SALA DE ENERGIA)
// =========================================================================

pub fn iniciar_minigame_minotauro(jogo: &mut GameState, ui: &mut TerminalBuffer) {
    jogo.estado_atual = "MINIGAME_MINOTAURO".to_string();
    let mut rng = rand::thread_rng();
    let mut armadilhas = Vec::new();
    while armadilhas.len() < 4 {
        let tx = rng.gen_range(-2..=2);
        let ty = rng.gen_range(0..=4);
        if (tx, ty) != (0, 0) && (tx, ty) != (0, 4) && !armadilhas.contains(&(tx, ty)) {
            armadilhas.push((tx, ty));
        }
    }

    jogo.minotauro_dados = Some(MinotauroData {
        px: 0,
        py: 0,
        mx: 0,
        my: 4,
        fusivel: (0, 4),
        saida: (0, 0),
        armadilhas,
        bateria: if jogo.god_mode { 9999 } else { 30 },
        tem_fios: false,
        tesoura_no_chao: true,
        celular_inspecionado: false,
    });

    ui.limpar();
    ui.animar(
        "==================================================",
        DOS_VERMELHO,
    );
    ui.animar(
        "      [ LABIRINTO DE ALTA TENSÃO - ESCURIDÃO ]    ",
        DOS_VERMELHO,
    );
    ui.animar(
        "==================================================",
        DOS_VERMELHO,
    );
    ui.animar(
        "Você entra na grade de manutenção da Sala de Energia. As grades se fecham atrás de você!",
        DOS_BRANCO,
    );
    ui.animar(
        "Há algo imenso respirando entre os transformadores no fundo (0, 4)...",
        DOS_VERMELHO,
    );
    ui.exibir(&format!(
        "{}OBJETIVO: Pegue a 'tesoura', chegue até a Caixa de Fusíveis [F] no norte (0, 4), use 'cortar fios' e volte para a Saída [S] em (0, 0) e digite 'sair'.{}",
        DOS_AMARELO, RESET
    ));
    ui.exibir(&format!(
        "{}COMANDOS: 'ir frente' (f), 'ir atras' (a), 'ir esquerda' (e), 'ir direita' (d), 'esperar', 'pegar tesoura', 'cortar fios', 'olhar celular', 'sair'.{}",
        DOS_VERDE, RESET
    ));

    desenhar_radar_ascii(jogo, ui);
    dar_dicas_minotauro(jogo, ui);
}

fn desenhar_radar_ascii(jogo: &GameState, ui: &mut TerminalBuffer) {
    let Some(ref m) = jogo.minotauro_dados else {
        return;
    };
    let dist = (m.px - m.mx).abs() + (m.py - m.my).abs();
    let mut rng = rand::thread_rng();
    let glitch = dist <= 1 && !jogo.god_mode && rng.gen::<f64>() < 0.20;

    ui.exibir(&format!(
        "\n{}[ RADAR DE CURTO ALCANCE | BATERIA: {}% ]{}",
        DOS_VERDE, m.bateria, RESET
    ));
    ui.exibir(&format!("{}+---+---+---+---+---+{}", DOS_VERDE, RESET));

    for y in (0..=4).rev() {
        let mut linha = format!("{}|", DOS_VERDE);
        for x in -2..=2 {
            let simbolo = if glitch && rng.gen::<f64>() < 0.4 {
                format!("{}?{}", DOS_VERMELHO, DOS_VERDE)
            } else if (x, y) == (m.px, m.py) {
                format!("{}@{}", DOS_BRANCO, DOS_VERDE)
            } else if (jogo.god_mode || jogo.dev_radar_ativo) && (x, y) == (m.mx, m.my) {
                format!("{}M{}", DOS_VERMELHO, DOS_VERDE)
            } else if (x, y) == m.fusivel && !m.tem_fios {
                format!("{}F{}", DOS_AMARELO, DOS_VERDE)
            } else if (x, y) == m.saida {
                format!("{}S{}", DOS_CIANO, DOS_VERDE)
            } else if (jogo.god_mode || jogo.dev_radar_ativo) && m.armadilhas.contains(&(x, y)) {
                format!("{}*{}", DOS_VERMELHO, DOS_VERDE)
            } else {
                ".".to_string()
            };
            linha.push_str(&format!(" {} |", simbolo));
        }
        linha.push_str(RESET);
        ui.exibir(&linha);
        ui.exibir(&format!("{}+---+---+---+---+---+{}", DOS_VERDE, RESET));
    }

    if m.px == 0 && m.py == 0 && m.tesoura_no_chao {
        ui.exibir(&format!(
            "{}Você vê uma 'tesoura' no chão ao lado de um 'celular quebrado'.{}",
            DOS_AMARELO, RESET
        ));
    }
}

fn dar_dicas_minotauro(jogo: &GameState, ui: &mut TerminalBuffer) {
    let Some(ref m) = jogo.minotauro_dados else {
        return;
    };
    let dx = m.mx - m.px;
    let dy = m.my - m.py;
    let dist = dx.abs() + dy.abs();
    let mut rng = rand::thread_rng();

    if dist <= 1 {
        if let Some(msg) = MSGS_PERIGO_ESTATICA.choose(&mut rng) {
            ui.exibir(&format!(
                "{}[ALERTA DE PROXIMIDADE CRÍTICA]: {}{}",
                DOS_VERMELHO,
                texto_cintilante(msg),
                RESET
            ));
        }
    }

    if dx < 0 {
        if let Some(msg) = MSGS_PERIGO_ESQUERDA.choose(&mut rng) {
            ui.exibir(&format!("{}[SOM À ESQUERDA]: {}{}", DOS_AMARELO, msg, RESET));
        }
    } else if dx > 0 {
        if let Some(msg) = MSGS_PERIGO_DIREITA.choose(&mut rng) {
            ui.exibir(&format!("{}[SOM À DIREITA]: {}{}", DOS_AMARELO, msg, RESET));
        }
    }

    if dy > 0 {
        if let Some(msg) = MSGS_PERIGO_FRENTE.choose(&mut rng) {
            ui.exibir(&format!("{}[SOM À FRENTE]: {}{}", DOS_AMARELO, msg, RESET));
        }
    } else if dy < 0 {
        if let Some(msg) = MSGS_PERIGO_TRAS.choose(&mut rng) {
            ui.exibir(&format!("{}[SOM ATRÁS]: {}{}", DOS_AMARELO, msg, RESET));
        }
    }
}

fn mover_minotauro(jogo: &mut GameState) {
    let Some(ref mut m) = jogo.minotauro_dados else {
        return;
    };
    let mut rng = rand::thread_rng();
    if rng.gen::<f64>() < 0.70 {
        let dx = m.px - m.mx;
        let dy = m.py - m.my;
        if dx.abs() >= dy.abs() && dx != 0 {
            m.mx += dx.signum();
        } else if dy != 0 {
            m.my += dy.signum();
        }
    } else {
        let dirs = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        if let Some((ox, oy)) = dirs.choose(&mut rng) {
            m.mx = (m.mx + ox).clamp(-2, 2);
            m.my = (m.my + oy).clamp(0, 4);
        }
    }
}

fn processar_minotauro(
    comando: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) {
    if jogo.god_mode && ["atacar", "bater", "chutar", "voadora"].contains(&comando) {
        if let Some(ref mut m) = jogo.minotauro_dados {
            m.mx = 0;
            m.my = 4;
        }
        ui.animar(
            "MODO DEUS: Você dá uma voadora de dois pés no peito do Minotauro mecânico, jogando-o de volta para o fundo da sala (0, 4)!",
            DOS_VERDE,
        );
        desenhar_radar_ascii(jogo, ui);
        return;
    }

    let Some(mut m) = jogo.minotauro_dados.clone() else {
        return;
    };

    let dist_antes = (m.px - m.mx).abs() + (m.py - m.my).abs();
    let old_px = m.px;
    let old_py = m.py;
    let mut gastou_turno = false;

    if ["pegar tesoura", "p tesoura", "tesoura"].contains(&comando) {
        if m.px == 0 && m.py == 0 && m.tesoura_no_chao {
            m.tesoura_no_chao = false;
            if !jogo.inventario.contains(&"tesoura".to_string()) {
                jogo.inventario.push("tesoura".to_string());
            }
            ui.animar("Você pegou a 'tesoura' do chão!", DOS_VERDE);
            gastou_turno = true;
        } else {
            ui.exibir(&format!(
                "{}Não há tesoura no chão aqui.{}",
                DOS_VERMELHO, RESET
            ));
        }
    } else if [
        "olhar celular",
        "examinar celular",
        "olhar celular quebrado",
        "celular",
    ]
    .contains(&comando)
    {
        if m.px == 0 && m.py == 0 {
            m.celular_inspecionado = true;
            ui.animar(
                "[CELULAR QUEBRADO]: A tela rachada mostra uma última mensagem não enviada: 'O código da porta de manutenção... ele trocou para o ano do acidente...'",
                DOS_AMARELO,
            );
        } else {
            ui.exibir(&format!(
                "{}O celular quebrado ficou lá na entrada (0, 0).{}",
                DOS_AMARELO, RESET
            ));
        }
    } else if ["cortar fios", "usar tesoura", "cortar"].contains(&comando) {
        if (m.px, m.py) == m.fusivel {
            if jogo.inventario.iter().any(|i| i == "tesoura") || jogo.god_mode {
                m.tem_fios = true;
                jogo.fios_cortados_inventario = true;
                jogo.inventario.retain(|i| i != "tesoura");
                jogo.inventario.push("tesoura quebrada".to_string());
                if !jogo.inventario.contains(&"fios cortados".to_string()) {
                    jogo.inventario.push("fios cortados".to_string());
                }
                audio.som_metal();
                ui.animar(
                    "SNAP! BZZZT! Você corta os cabos de alta tensão da caixa de fusíveis! A tesoura quebra com o arco voltaico, mas você guarda os 'fios cortados'!",
                    DOS_VERDE,
                );
                ui.animar(
                    "AGORA VOLTE PARA A SAÍDA [S] EM (0, 0) E DIGITE 'sair'!",
                    DOS_AMARELO,
                );
                gastou_turno = true;
            } else {
                ui.exibir(&format!(
                    "{}Você precisa da 'tesoura' intacta para cortar estes cabos grossos!{}",
                    DOS_VERMELHO, RESET
                ));
            }
        } else {
            ui.exibir(&format!(
                "{}Você precisa estar na Caixa de Fusíveis [F] em (0, 4) para cortar os fios!{}",
                DOS_VERMELHO, RESET
            ));
        }
    } else if comando == "sair" {
        if (m.px, m.py) == m.saida {
            if m.tem_fios {
                ui.limpar();
                ui.animar(
                    "Você se joga pela porta da Sala de Energia e bate a grade bem na cara da criatura mecânica!",
                    DOS_VERDE,
                );
                desbloquear_conquista("toureiro", jogo, ui, audio);
                jogo.estado_atual = "JOGO".to_string();
                jogo.sala_atual = "sala dos fundos".to_string();
                jogo.minotauro_dados = Some(m);
                views::imprimir_contexto_sala(jogo, ui, media);
                return;
            } else {
                ui.limpar();
                ui.animar(
                    "Você recua de volta para a Sala dos Fundos antes de cortar os fios.",
                    DOS_AMARELO,
                );
                jogo.estado_atual = "JOGO".to_string();
                jogo.sala_atual = "sala dos fundos".to_string();
                jogo.minotauro_dados = Some(m);
                views::imprimir_contexto_sala(jogo, ui, media);
                return;
            }
        } else {
            ui.exibir(&format!(
                "{}Você precisa voltar até a Saída [S] em (0, 0) para sair da grade!{}",
                DOS_VERMELHO, RESET
            ));
        }
    } else if comando == "esperar" {
        ui.animar(
            "Você prende a respiração no escuro e aguarda...",
            DOS_AMARELO,
        );
        gastou_turno = true;
    } else {
        let dir = match comando {
            "ir frente" | "frente" | "f" | "w" => Some((0, 1)),
            "ir atras" | "ir atrás" | "atras" | "atrás" | "a" | "t" | "s" => Some((0, -1)),
            "ir esquerda" | "esquerda" | "e" => Some((-1, 0)),
            "ir direita" | "direita" | "d" => Some((1, 0)),
            _ => None,
        };

        if let Some((dx, dy)) = dir {
            let nx = m.px + dx;
            let ny = m.py + dy;
            if (-2..=2).contains(&nx) && (0..=4).contains(&ny) {
                m.px = nx;
                m.py = ny;
                gastou_turno = true;
            } else {
                ui.exibir(&format!(
                    "{}Uma grade de ferro eletrificada bloqueia essa direção!{}",
                    DOS_VERMELHO, RESET
                ));
            }
        } else {
            ui.exibir(&format!(
                "{}Comando inválido no labirinto. Use 'ir frente/atras/esquerda/direita', 'esperar', 'pegar tesoura', 'cortar fios' ou 'sair'.{}",
                DOS_AMARELO, RESET
            ));
        }
    }

    jogo.minotauro_dados = Some(m);

    if gastou_turno {
        ui.limpar();

        if let Some(ref mut md) = jogo.minotauro_dados {
            if !jogo.god_mode {
                md.bateria -= 1;
            }
            if let Some(pos_arm) = md.armadilhas.iter().position(|&p| p == (md.px, md.py)) {
                md.armadilhas.remove(pos_arm);
                if !jogo.god_mode {
                    md.bateria = (md.bateria - 3).max(0);
                }
                ui.buffer_efeito("@@GLITCH_LUZ@@");
                ui.animar(
                    "BZZZT! Você pisa numa poça de óleo eletrificada! Sua bateria drena rapidamente (-3%)!",
                    DOS_VERMELHO,
                );
            }
        }

        mover_minotauro(jogo);

        let colisao = jogo
            .minotauro_dados
            .as_ref()
            .map(|md| (md.px, md.py) == (md.mx, md.my))
            .unwrap_or(false);

        if colisao && !jogo.god_mode {
            if dist_antes > 1 {
                if let Some(ref mut md) = jogo.minotauro_dados {
                    md.px = old_px;
                    md.py = old_py;
                }
                ui.buffer_efeito("@@GLITCH_LUZ@@");
                ui.animar(
                    "IMPACTO! Você esbarra no peito metálico da criatura no escuro, mas consegue se jogar para trás no último segundo!",
                    DOS_VERMELHO,
                );
            } else {
                ui.animar(
                    "Os chifres de metal enferrujado atravessam você na escuridão da Sala de Energia!",
                    DOS_VERMELHO,
                );
                views::dar_tela_de_morte(jogo, ui, audio, media);
                return;
            }
        }

        let sem_bateria = jogo
            .minotauro_dados
            .as_ref()
            .map(|md| md.bateria <= 0)
            .unwrap_or(false);

        if sem_bateria && !jogo.god_mode {
            ui.animar(
                "A bateria do seu radar zera completamente. No breu total, passos pesados correm na sua direção...",
                DOS_VERMELHO,
            );
            views::dar_tela_de_morte(jogo, ui, audio, media);
            return;
        }

        desenhar_radar_ascii(jogo, ui);
        dar_dicas_minotauro(jogo, ui);
    }
}

// =========================================================================
// 6. MINIGAME: SALA DE SEGURANÇA (00:00 ÀS 06:00 AM)
// =========================================================================

pub fn iniciar_minigame_seguranca(jogo: &mut GameState, ui: &mut TerminalBuffer) {
    jogo.estado_atual = "MINIGAME_SEGURANCA".to_string();

    let mut energia_inicial = 100;
    if jogo.turnos_luz > 0 {
        energia_inicial += jogo.turnos_luz * 2;
    }
    let qtd_baterias = jogo
        .inventario
        .iter()
        .filter(|i| *i == "bateria nova")
        .count() as i32;
    if qtd_baterias > 0 {
        energia_inicial += qtd_baterias * 25;
        jogo.inventario.retain(|i| i != "bateria nova");
    }
    if jogo.god_mode {
        energia_inicial = 9999;
    }

    jogo.seguranca_dados = Some(SegurancaData {
        turno: 0,
        energia: energia_inicial,
        porta_fechada: false,
        gerador_ligado: false,
        usos_sistema_turno: 0,
        camera_quebrada: false,
        deteccao_quebrada: false,
        relogio_quebrado: false,
        captcha_ativo: None,
        sistema_alvo_conserto: None,
        rick_pos: 0,
        jon_pos: 0,
        caroline_ativa: false,
        caroline_local: String::new(),
        indio_ativo: false,
        alberto_msg_falsa: false,
        calor_ativo: false,
        fios_cortados: jogo.fios_cortados_inventario
            || jogo.inventario.iter().any(|i| i == "fios cortados"),
    });

    ui.limpar();
    if let Some(arte) = jogo.ascii_arts.get("ARTE_MESA_SEGURANCA") {
        ui.animar(&format!("{}{}{}", DOS_VERDE, arte, RESET), DOS_VERDE);
    }
    ui.animar(
        "==================================================",
        DOS_VERDE,
    );
    ui.animar(
        "   [ TERMINAL CENTRAL DE SEGURANÇA - TURNO DA NOITE ]",
        DOS_VERDE,
    );
    ui.animar(
        "==================================================",
        DOS_VERDE,
    );
    ui.animar(
        "Você passa o Cartão de Segurança e se tranca no escritório. Sobreviva das 12:00 AM até as 06:00 AM (24 turnos)!",
        DOS_BRANCO,
    );
    if qtd_baterias > 0 {
        ui.animar(
            &format!(
                "Você conectou {} bateria(s) extra(s) ao nobreak da sala! (+{}% Energia)",
                qtd_baterias,
                qtd_baterias * 25
            ),
            DOS_VERDE,
        );
    }
    ui.exibir(&format!(
        "{}COMANDOS: 'cameras', 'ouvir', 'ver tubulacao', 'iluminar tubulacao', 'fechar porta', 'abrir porta', 'olhar vidro', 'ligar gerador', 'consertar [camera/deteccao/relogio]', 'esperar'.{}",
        DOS_AMARELO, RESET
    ));

    imprimir_hud_seguranca(jogo, ui);
}

fn formatar_hora_seguranca(turno: i32, relogio_quebrado: bool) -> String {
    if relogio_quebrado {
        return "??:?? AM [ERRO]".to_string();
    }
    let hora = turno / 4;
    let min = (turno % 4) * 15;
    let hora_display = if hora == 0 { 12 } else { hora };
    format!("{:02}:{:02} AM", hora_display, min)
}

fn aplicar_corrupcao_caroline(texto: &str, ativa: bool) -> String {
    if !ativa {
        return texto.to_string();
    }
    let mut rng = rand::thread_rng();
    let alternado: String = texto
        .chars()
        .map(|c| {
            if rng.gen::<f64>() < 0.35 {
                if c.is_uppercase() {
                    c.to_lowercase().collect::<String>()
                } else {
                    c.to_uppercase().collect::<String>()
                }
            } else {
                c.to_string()
            }
        })
        .collect();
    corromper_texto(&alternado, 0.25)
}

fn imprimir_hud_seguranca(jogo: &GameState, ui: &mut TerminalBuffer) {
    let Some(ref s) = jogo.seguranca_dados else {
        return;
    };
    let hora_str = formatar_hora_seguranca(s.turno, s.relogio_quebrado);
    let status_porta = if s.porta_fechada {
        format!("{}[FECHADA]{}", DOS_VERMELHO, DOS_VERDE)
    } else {
        format!("{}[ABERTA]{}", DOS_BRANCO, DOS_VERDE)
    };
    let status_gerador = if s.gerador_ligado {
        format!("{}[LIGADO - FAZENDO BARULHO]{}", DOS_AMARELO, DOS_VERDE)
    } else {
        format!("{}[DESLIGADO]{}", DOS_BRANCO, DOS_VERDE)
    };

    let linha_hud = format!(
        "\n{}[ HORA: {} | ENERGIA: {}% | PORTA: {} | GERADOR: {} ]{}",
        DOS_VERDE, hora_str, s.energia, status_porta, status_gerador, RESET
    );
    ui.exibir(&aplicar_corrupcao_caroline(&linha_hud, s.caroline_ativa));

    if s.calor_ativo {
        ui.exibir(&format!(
            "{}[AVISO TÉRMICO]: A ventilação está superaquecendo! Feche a porta ou aguente o calor!{}",
            DOS_VERMELHO, RESET
        ));
    }
    if s.alberto_msg_falsa {
        ui.exibir(&format!(
            "{}[MENSAGEM]: ERRO CRÍTICO! FECHAR PORTA AGORA!{}",
            DOS_VERMELHO, RESET
        ));
    }
    if s.camera_quebrada || s.deteccao_quebrada || s.relogio_quebrado {
        let mut falhas = Vec::new();
        if s.camera_quebrada {
            falhas.push("camera");
        }
        if s.deteccao_quebrada {
            falhas.push("deteccao");
        }
        if s.relogio_quebrado {
            falhas.push("relogio");
        }
        ui.exibir(&format!(
            "{}[SISTEMAS DEFEITUOSOS]: {} (Use 'consertar <sistema>'){}",
            DOS_AMARELO,
            falhas.join(", "),
            RESET
        ));
    }
}

fn gerar_captcha() -> CaptchaData {
    let mut rng = rand::thread_rng();
    let tipo = rng.gen_range(0..4);
    match tipo {
        0 => {
            let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();
            let codigo: String = (0..5).map(|_| *chars.choose(&mut rng).unwrap()).collect();
            CaptchaData {
                pergunta: format!(
                    "[REBOOT DE SEGURANÇA - CAPTCHA DE TEXTO]\nDigite exatamente o código de verificação: {}",
                    codigo
                ),
                resposta: codigo.to_lowercase(),
            }
        }
        1 => {
            let a = rng.gen_range(11..=29);
            let b = rng.gen_range(11..=29);
            CaptchaData {
                pergunta: format!(
                    "[REBOOT DE SEGURANÇA - VERIFICAÇÃO LÓGICA]\nResolva a soma de verificação: {} + {} =",
                    a, b
                ),
                resposta: (a + b).to_string(),
            }
        }
        2 => {
            let seqs = [
                ("2, 4, 8, 16, ?", "32"),
                ("3, 6, 9, 12, ?", "15"),
                ("1, 1, 2, 3, 5, ?", "8"),
                ("10, 20, 30, 40, ?", "50"),
            ];
            let (seq, resp) = *seqs.choose(&mut rng).unwrap();
            CaptchaData {
                pergunta: format!(
                    "[REBOOT DE SEGURANÇA - SEQUÊNCIA DE HARDWARE]\nComplete o próximo número da sequência: {}",
                    seq
                ),
                resposta: resp.to_string(),
            }
        }
        _ => {
            let dirs = [
                ("> > < >", "dded"),
                ("< > < <", "edee"),
                ("> < > >", "dedd"),
            ];
            let (setas, resp) = *dirs.choose(&mut rng).unwrap();
            CaptchaData {
                pergunta: format!(
                    "[REBOOT DE SEGURANÇA - CALIBRAÇÃO DIRECIONAL]\nTraduza as setas '{}' usando 'e' (esquerda) e 'd' (direita) sem espaços:",
                    setas
                ),
                resposta: resp.to_string(),
            }
        }
    }
}

fn vencer_noite_seguranca(
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    media: &mut MediaManager,
) {
    ui.limpar();
    ui.buffer_efeito("@@NORMAL_POWER@@");
    ui.animar(
        "==================================================",
        DOS_VERDE,
    );
    ui.animar(
        "      🔔 06:00 AM - FIM DO TURNO DE SEGURANÇA     ",
        DOS_VERDE,
    );
    ui.animar(
        "==================================================",
        DOS_VERDE,
    );
    ui.animar(
        "O relógio digital apita 06:00 AM! Os motores dos animatrônicos desligam um por um no corredor.",
        DOS_VERDE,
    );
    ui.animar(
        "Os primeiros raios de sol atravessam as janelas sujas do restaurante.",
        DOS_BRANCO,
    );

    jogo.noite_vencida = true;
    jogo.amanheceu = true;
    jogo.posicao_perseguidor = "desativado".to_string();
    jogo.estado_atual = "JOGO".to_string();

    if let Some(hall) = jogo.mapa.get_mut("hall") {
        hall.descricao = "A luz da manhã ilumina o Salão Principal. No centro do palco, a carcaça imóvel de Caroline parece encarar o vazio, esperando um desfecho. As portas para a 'saida' estão destrancadas.".to_string();
        hall.conexoes
            .insert("saida".to_string(), Some("saida".to_string()));
    }
    if let Some(sala04) = jogo.mapa.get_mut("04") {
        sala04.descricao = "O quarto silencioso agora recebe uma fresta de luz pálida. A 'cama' velha no canto parece estranhamente convidativa após uma noite inteira de pavor.".to_string();
        sala04
            .conexoes
            .insert("cama".to_string(), Some("cama".to_string()));
    }

    let tem_fios = jogo
        .seguranca_dados
        .as_ref()
        .map(|s| s.fios_cortados)
        .unwrap_or(false)
        || jogo.fios_cortados_inventario;

    if tem_fios {
        if let Some(arte) = jogo.ascii_arts.get("ARTE_RADAR") {
            ui.animar(&format!("{}{}{}", DOS_VERDE, arte, RESET), DOS_VERDE);
        }
        ui.animar(
            "[SINAL DETECTADO]: Com os fios cortados da Sala de Energia e seu isqueiro, você pode dar um fim verdadeiro a tudo isso no 'hall'...",
            DOS_AMARELO,
        );
    }

    views::imprimir_contexto_sala(jogo, ui, media);
}

fn processar_seguranca(
    comando: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) {
    if jogo.god_mode && ["pular noite", "vencer", "6am"].contains(&comando) {
        vencer_noite_seguranca(jogo, ui, media);
        return;
    }

    let Some(mut s) = jogo.seguranca_dados.clone() else {
        return;
    };

    // Se estiver respondendo a um CAPTCHA de conserto
    if let Some(ref cap) = s.captcha_ativo.clone() {
        if comando == cap.resposta {
            ui.animar(
                "CÓDIGO ACEITO! Sistema reiniciado com sucesso!",
                DOS_VERDE,
            );
            if let Some(ref alvo) = s.sistema_alvo_conserto {
                match alvo.as_str() {
                    "camera" => s.camera_quebrada = false,
                    "deteccao" => s.deteccao_quebrada = false,
                    "relogio" => s.relogio_quebrado = false,
                    _ => {}
                }
            }
            s.captcha_ativo = None;
            s.sistema_alvo_conserto = None;
            jogo.seguranca_dados = Some(s);
            imprimir_hud_seguranca(jogo, ui);
        } else {
            ui.exibir(&format!(
                "{}RESPOSTA INCORRETA! Tente novamente:\n{}{}",
                DOS_VERMELHO, cap.pergunta, RESET
            ));
        }
        return;
    }

    let mut avancar_tempo = false;

    // Ações de sistema que não passam o turno (mas gastam usos_sistema_turno)
    if ["cameras", "camera", "cam", "olhar cameras"].contains(&comando) {
        s.usos_sistema_turno += 1;
        if s.usos_sistema_turno >= 5 && !jogo.god_mode {
            ui.animar(
                "SOBRECARGA DE HARDWARE! O excesso de requisições simultâneas derreteu a placa-mãe do terminal!",
                DOS_VERMELHO,
            );
            views::dar_tela_kernel_panic(jogo, ui);
            return;
        }
        if s.usos_sistema_turno > 2 && !jogo.god_mode {
            s.camera_quebrada = true;
            ui.exibir(&format!(
                "{}[ERRO]: O feed de vídeo travou por excesso de uso neste turno! Use 'consertar camera'.{}",
                DOS_VERMELHO, RESET
            ));
            jogo.seguranca_dados = Some(s);
            return;
        }
        if s.camera_quebrada {
            ui.exibir(&format!(
                "{}[SINAL PERDIDO]: As câmeras estão fora do ar! Digite 'consertar camera'.{}",
                DOS_VERMELHO, RESET
            ));
            jogo.seguranca_dados = Some(s);
            return;
        }

        if !jogo.god_mode {
            s.energia = (s.energia - 2).max(0);
        }

        let msg_rick = match s.rick_pos {
            0 => "CAM 01 (Palco): Rick está no palco.",
            1 => "CAM 02 (Corredor Leste): Rick está caminhando pelo corredor.",
            _ => "CAM 03 (Porta do Escritório): [ALERTA] RICK ESTÁ PARADO NA PORTA!",
        };
        let msg_jon = match s.jon_pos {
            0 => "CAM 04 (Cozinha): Jon está vasculhando panelas.",
            1 => "CAM 05 (Entrada do Duto): Jon entrou na tubulação de ar!",
            _ => "CAM 06 (Saída do Duto): [PERIGO] JON ESTÁ NA GRADE DA TUBULAÇÃO!",
        };
        let mut relatorio = format!("{}\n{}", msg_rick, msg_jon);
        if s.caroline_ativa {
            relatorio.push_str(&format!(
                "\n[ANOMALIA CRÍTICA]: CAROLINE DETECTADA NA {}!",
                s.caroline_local.to_uppercase()
            ));
        }
        if !s.deteccao_quebrada {
            ui.animar(
                &aplicar_corrupcao_caroline(&relatorio, s.caroline_ativa),
                DOS_CIANO,
            );
        } else {
            ui.animar(
                "As câmeras mostram estática pesada (Sensor de Detecção quebrado!). Você mal distingue vultos se mexendo.",
                DOS_AMARELO,
            );
        }
        jogo.seguranca_dados = Some(s);
        return;
    } else if ["ouvir", "escutar"].contains(&comando) {
        if s.rick_pos >= 2 || (s.caroline_ativa && s.caroline_local == "porta") {
            ui.animar(
                "Você ouve passos metálicos pesadíssimos bem atrás da porta de ferro!",
                DOS_VERMELHO,
            );
        } else if s.jon_pos >= 1 || (s.caroline_ativa && s.caroline_local == "tubulacao") {
            ui.animar(
                "Você ouve barulho de metal raspando dentro da tubulação de ar acima da sua cabeça!",
                DOS_AMARELO,
            );
        } else {
            ui.animar(
                "Apenas o zumbido distante dos ventiladores.",
                DOS_BRANCO,
            );
        }
        jogo.seguranca_dados = Some(s);
        return;
    } else if ["ver tubulacao", "olhar tubulacao", "tubulacao"].contains(&comando) {
        if s.jon_pos >= 2 || (s.caroline_ativa && s.caroline_local == "tubulacao") {
            ui.animar(
                "Há dois olhos brilhando na escuridão da grade da tubulação! Use 'iluminar tubulacao'!",
                DOS_VERMELHO,
            );
        } else {
            ui.animar("A grade da tubulação parece vazia no momento.", DOS_BRANCO);
        }
        jogo.seguranca_dados = Some(s);
        return;
    } else if ["olhar vidro", "ver vidro", "vidro", "janela"].contains(&comando) {
        if s.indio_ativo {
            if let Some(arte) = jogo.ascii_arts.get("ARTE_INDIO") {
                ui.animar(&format!("{}{}{}", DOS_AMARELO, arte, RESET), DOS_AMARELO);
            }
            ui.animar(
                "Índio Jones estava encarando você pelo vidro! Ao cruzar olhares com você, ele recua lentamente para as sombras.",
                DOS_VERDE,
            );
            s.indio_ativo = false;
        } else {
            ui.animar(
                "Você olha pelo vidro blindado. Apenas o corredor escuro lá fora.",
                DOS_BRANCO,
            );
        }
        jogo.seguranca_dados = Some(s);
        return;
    } else if let Some(alvo) = comando.strip_prefix("consertar ") {
        let sis = alvo.trim();
        let valido = match sis {
            "camera" | "cameras" => {
                s.sistema_alvo_conserto = Some("camera".to_string());
                s.camera_quebrada
            }
            "deteccao" | "sensor" => {
                s.sistema_alvo_conserto = Some("deteccao".to_string());
                s.deteccao_quebrada
            }
            "relogio" | "hora" => {
                s.sistema_alvo_conserto = Some("relogio".to_string());
                s.relogio_quebrado
            }
            _ => {
                ui.exibir(&format!(
                    "{}Especifique: 'consertar camera', 'consertar deteccao' ou 'consertar relogio'.{}",
                    DOS_AMARELO, RESET
                ));
                jogo.seguranca_dados = Some(s);
                return;
            }
        };

        if !valido {
            ui.exibir(&format!(
                "{}Esse sistema está funcionando normalmente.{}",
                DOS_VERDE, RESET
            ));
            s.sistema_alvo_conserto = None;
            jogo.seguranca_dados = Some(s);
            return;
        }

        ui.buffer_efeito("@@BSOD@@");
        let cap = gerar_captcha();
        ui.exibir(&format!(
            "\n{}=== VILAS-BOAS OS - RECOVERY CONSOLE ===\n{}{}",
            DOS_CIANO, cap.pergunta, RESET
        ));
        s.captcha_ativo = Some(cap);
        jogo.seguranca_dados = Some(s);
        return;
    }

    // Comandos que avançam o turno da noite
    if ["iluminar tubulacao", "luz tubulacao", "flashear tubulacao"].contains(&comando) {
        if !jogo.god_mode {
            s.energia = (s.energia - 4).max(0);
        }
        if s.jon_pos >= 2 {
            s.jon_pos = 0;
            ui.animar(
                "FLASH! O feixe de luz cega Jon na tubulação! Ele recua guinchando de volta para a cozinha!",
                DOS_VERDE,
            );
        } else if s.caroline_ativa && s.caroline_local == "tubulacao" {
            s.caroline_ativa = false;
            s.caroline_local.clear();
            ui.animar(
                "FLASH! A luz forte atinge o rosto derretido de Caroline no duto! Ela recua soltando um chiado estático!",
                DOS_VERDE,
            );
        } else {
            ui.animar(
                "Você ilumina a tubulação vazia, gastando bateria à toa.",
                DOS_AMARELO,
            );
        }
        avancar_tempo = true;
    } else if ["fechar porta", "porta fechar", "trancar porta"].contains(&comando) {
        if s.alberto_msg_falsa && !s.caroline_ativa && s.rick_pos < 2 {
            ui.animar(
                "ARMADILHA DE REDE (ALBERTO)! A mensagem de erro era falsa! Ao fechar a porta, um curto-circuito drena 15% da sua energia!",
                DOS_VERMELHO,
            );
            if !jogo.god_mode {
                s.energia = (s.energia - 15).max(0);
            }
            s.alberto_msg_falsa = false;
        }
        s.porta_fechada = true;
        audio.som_porta();
        ui.animar(
            "CLANG! A pesada porta de aço desce e tranca a entrada do escritório.",
            DOS_AMARELO,
        );
        avancar_tempo = true;
    } else if ["abrir porta", "porta abrir", "destrancar porta"].contains(&comando) {
        s.porta_fechada = false;
        audio.som_porta();
        ui.animar("A porta de aço sobe, economizando energia.", DOS_VERDE);
        avancar_tempo = true;
    } else if ["ligar gerador", "gerador", "usar gerador"].contains(&comando) {
        s.gerador_ligado = !s.gerador_ligado;
        if s.gerador_ligado {
            ui.animar(
                "Você aciona o gerador auxiliar! A energia vai subir (+8%/turno), mas o barulho do motor vai atrair os animatrônicos!",
                DOS_AMARELO,
            );
        } else {
            ui.animar("Você desliga o gerador auxiliar.", DOS_BRANCO);
        }
        avancar_tempo = true;
    } else if ["esperar", "aguardar", "passar"].contains(&comando) {
        ui.animar("Você aguarda tenso olhando os monitores...", DOS_BRANCO);
        avancar_tempo = true;
    } else {
        ui.exibir(&format!(
            "{}Comando desconhecido. Use: 'cameras', 'ouvir', 'ver tubulacao', 'iluminar tubulacao', 'fechar porta', 'abrir porta', 'olhar vidro', 'ligar gerador', 'consertar <sistema>' ou 'esperar'.{}",
            DOS_AMARELO, RESET
        ));
        jogo.seguranca_dados = Some(s);
        return;
    }

    if !avancar_tempo {
        jogo.seguranca_dados = Some(s);
        return;
    }

    // Avança 1 turno (15 minutos)
    s.turno += 1;
    s.usos_sistema_turno = 0;
    s.alberto_msg_falsa = false;

    // Consumo / Ganho de energia
    if !jogo.god_mode {
        let mut delta_energia = -2;
        if s.porta_fechada {
            delta_energia -= 4;
        }
        if s.gerador_ligado {
            delta_energia += 8;
        }
        s.energia = (s.energia + delta_energia).clamp(0, 999);
    }

    if s.energia <= 0 && !jogo.god_mode {
        s.porta_fechada = false;
        ui.buffer_efeito("@@BLACKOUT@@");
        ui.animar(
            "APAGÃO TOTAL! A energia do escritório chegou a 0%! As luzes se apagam e a porta magnética se abre sozinha...",
            DOS_VERMELHO,
        );
        views::dar_tela_de_morte(jogo, ui, audio, media);
        return;
    }

    // Verifica ataques fatais dos animatrônicos que já estavam em posição de ataque
    if !jogo.god_mode {
        if s.rick_pos >= 2 {
            if s.porta_fechada {
                ui.animar(
                    "BAM! BAM! Rick bate com força na porta fechada e vai embora frustrado!",
                    DOS_VERDE,
                );
                s.rick_pos = 0;
            } else {
                ui.animar(
                    "Rick invade o escritório pela porta aberta!",
                    DOS_VERMELHO,
                );
                views::dar_tela_de_morte(jogo, ui, audio, media);
                return;
            }
        }

        if s.jon_pos >= 2 {
            ui.animar(
                "Jon despenca da grade da tubulação direto em cima de você!",
                DOS_VERMELHO,
            );
            views::dar_tela_de_morte(jogo, ui, audio, media);
            return;
        }

        if s.caroline_ativa {
            if s.caroline_local == "porta" {
                if s.porta_fechada {
                    ui.animar(
                        "Um arranhão agudo percorre a porta de ferro... A presença de Caroline se dissipa.",
                        DOS_VERDE,
                    );
                    s.caroline_ativa = false;
                    s.caroline_local.clear();
                } else {
                    ui.animar(
                        "Caroline atravessa a porta aberta! O sistema entra em colapso!",
                        DOS_VERMELHO,
                    );
                    views::dar_tela_de_morte(jogo, ui, audio, media);
                    return;
                }
            } else if s.caroline_local == "tubulacao" {
                ui.animar(
                    "Caroline salta da tubulação de ar na escuridão!",
                    DOS_VERMELHO,
                );
                views::dar_tela_de_morte(jogo, ui, audio, media);
                return;
            }
        }

        if s.indio_ativo {
            ui.buffer_efeito("@@GLITCH_LUZ@@");
            ui.animar(
                "Você ignorou Índio Jones no vidro! Ele causa um pulso eletromagnético que quebra um sistema e drena 10% de energia!",
                DOS_VERMELHO,
            );
            s.energia = (s.energia - 10).max(0);
            s.camera_quebrada = true;
            s.indio_ativo = false;
        }
    } else {
        // No God Mode, se a porta estiver fechada rebate o Rick/Caroline
        if s.porta_fechada && s.rick_pos >= 2 {
            s.rick_pos = 0;
        }
        if s.porta_fechada && s.caroline_ativa && s.caroline_local == "porta" {
            s.caroline_ativa = false;
            s.caroline_local.clear();
        }
    }

    // Checa vitória às 06:00 AM (24 turnos)
    if s.turno >= 24 {
        jogo.seguranca_dados = Some(s);
        vencer_noite_seguranca(jogo, ui, media);
        return;
    }

    // Eventos de calor nos turnos 12 (03:00 AM) e 22 (05:30 AM)
    s.calor_ativo = s.turno == 12 || s.turno == 22;

    // Movimentação de IA para o próximo turno
    let mut rng = rand::thread_rng();
    let chance_base = if s.gerador_ligado { 0.55 } else { 0.30 };

    if s.rick_pos < 2 && rng.gen::<f64>() < chance_base {
        s.rick_pos += 1;
    }
    if s.jon_pos < 2 && rng.gen::<f64>() < chance_base {
        s.jon_pos += 1;
    }
    if !s.caroline_ativa && s.turno >= 6 && rng.gen::<f64>() < 0.20 {
        s.caroline_ativa = true;
        s.caroline_local = if rng.gen::<bool>() {
            "porta".to_string()
        } else {
            "tubulacao".to_string()
        };
    }
    if !s.indio_ativo && rng.gen::<f64>() < 0.18 {
        s.indio_ativo = true;
        ui.exibir(&format!(
            "{}Você nota uma sombra parada atrás do vidro blindado...{}",
            DOS_AMARELO, RESET
        ));
    }
    if rng.gen::<f64>() < 0.15 {
        s.alberto_msg_falsa = true;
    }
    if rng.gen::<f64>() < 0.12 {
        match rng.gen_range(0..3) {
            0 => s.camera_quebrada = true,
            1 => s.deteccao_quebrada = true,
            _ => s.relogio_quebrado = true,
        }
    }

    jogo.seguranca_dados = Some(s);
    imprimir_hud_seguranca(jogo, ui);
}

// =========================================================================
// ROTEADOR CENTRAL DE MINIGAMES
// =========================================================================

pub fn processar_minigame(
    comando: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) -> bool {
    match jogo.estado_atual.as_str() {
        "MINIGAME_COFRE" => {
            processar_cofre(comando, jogo, ui, audio, media);
            true
        }
        "MINIGAME_JON" => {
            processar_jon(comando, jogo, ui, audio, media);
            true
        }
        "MINIGAME_CONSERTOS_CABECA"
        | "MINIGAME_CONSERTOS_TRONCO"
        | "MINIGAME_CONSERTOS_PERNAS" => {
            processar_consertos(comando, jogo, ui, audio, media);
            true
        }
        "MINIGAME_JULGAMENTO_Q1"
        | "MINIGAME_JULGAMENTO_Q2"
        | "MINIGAME_JULGAMENTO_Q3"
        | "MINIGAME_JULGAMENTO_Q4"
        | "MINIGAME_JULGAMENTO_V1"
        | "MINIGAME_JULGAMENTO_V2"
        | "MINIGAME_JULGAMENTO_V3" => {
            processar_julgamento(comando, jogo, ui, audio, media);
            true
        }
        "MINIGAME_MINOTAURO" => {
            processar_minotauro(comando, jogo, ui, audio, media);
            true
        }
        "MINIGAME_SEGURANCA" => {
            processar_seguranca(comando, jogo, ui, audio, media);
            true
        }
        _ => false,
    }
}
