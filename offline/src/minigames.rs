use crate::audio::AudioManager;
use crate::data::{COFRE_SENHA, MAX_TENTATIVAS_COFRE};
use crate::media::MediaManager;
use crate::state::{GameState, MinotauroData};
use crate::ui::{DOS_AMARELO, DOS_BRANCO, DOS_VERDE, DOS_VERMELHO, RESET};
use crate::utils::desbloquear_conquista;
use crate::views;
use crate::TerminalBuffer;
use rand::seq::SliceRandom;
use rand::Rng;

const PISTAS_COFRE: [&str; 3] = [
    "[1º dígito] Um único ponto de luz acende sozinho na unidade, o primeiro som da noite.",
    "[2º e 3º dígitos] Nove marcas de unha, em 2 vezes, na fileira da lateral do metal.",
    "[4º dígito] Quatro cadeiras na sala do piano. Só uma nunca foi ocupada de novo.",
];

fn entregar_item_ou_chao(item: &str, jogo: &mut GameState, ui: &mut TerminalBuffer) {
    if jogo.adicionar_item_inventario(item) {
        ui.animar(
            &format!("Item adicionado à mochila: '{}'.", item),
            DOS_VERDE,
        );
    } else {
        if let Some(sala) = jogo.mapa.get_mut(&jogo.sala_atual) {
            sala.itens.push(item.to_string());
        }
        ui.animar(
            &format!("Mochila cheia! '{}' caiu no chão da sala.", item),
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
    _media: &mut MediaManager,
    primeira_vez: bool,
) {
    let (arte, cor) = if jogo.cofre_tentativas == 0 {
        ("ARTE_COFRE", DOS_BRANCO)
    } else {
        ("ARTE_COFRE_TENSO", DOS_VERMELHO)
    };
    if let Some(painel) = jogo.ascii_arts.get(arte) {
        ui.animar_com_intervalo(painel, cor, 0.015);
    }
    if primeira_vez {
        ui.exibir("O cofre de ferro possui um teclado numérico antigo, gasto pelo tempo.");
        ui.exibir("Gravado a canivete embaixo do teclado, quatro frases soltas:");
        let mut pistas = PISTAS_COFRE;
        pistas.shuffle(&mut rand::thread_rng());
        for pista in pistas {
            ui.exibir(&format!("  {pista}"));
        }
    }
    if jogo.cofre_tentativas > 0 {
        let restantes = MAX_TENTATIVAS_COFRE - jogo.cofre_tentativas;
        ui.exibir(&format!(
            "{DOS_VERMELHO}Tentativas restantes antes do painel travar: {restantes}{RESET}"
        ));
    }
    ui.exibir(&format!("{DOS_VERDE}Digite a senha de 4 dígitos: {RESET}"));
}

fn processar_cofre(
    comando: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) {
    if ["cls", "limpar", "clear", "clean"].contains(&comando) {
        ui.limpar();
        imprimir_painel_cofre(jogo, ui, media, true);
        return;
    }
    if comando == COFRE_SENHA {
        ui.exibir(&format!(
            "{DOS_VERDE} Um som de 'click'. A pesada porta de metal se abre.{RESET}"
        ));
        if jogo.cofre_tentativas == 0 {
            desbloquear_conquista("mente_brilhante", jogo, ui, audio);
        }
        let chave = "chave dos fundos";
        if item_presente(jogo, chave) {
            ui.exibir("O cofre está vazio. Apenas poeira.");
        } else {
            entregar_item_ou_chao(chave, jogo, ui);
        }
    } else {
        jogo.cofre_tentativas += 1;
        if jogo.cofre_tentativas < MAX_TENTATIVAS_COFRE {
            let mensagem = if jogo.cofre_tentativas == 1 {
                " ⛝ Senha incorreta. Painel pisca em vermelho.⛝"
            } else {
                " ⛝ Errado. Um zumbido baixo sai de dentro do cofre.⛝"
            };
            ui.exibir(&format!("{DOS_VERMELHO}{mensagem}{RESET}"));
            imprimir_painel_cofre(jogo, ui, media, false);
            return;
        }
        ui.exibir(&format!("{DOS_VERMELHO} ⛝ Senha incorreta. O painel solta uma fagulha e trava com um estalo seco.⛝{RESET}"));
        ui.exibir(&format!("{DOS_AMARELO}O teclado esfria, morto. Talvez ele aceite tentar de novo mais tarde.{RESET}"));
    }
    jogo.cofre_tentativas = 0;
    voltar_ao_jogo(jogo, ui, media);
}

fn item_presente(jogo: &GameState, item: &str) -> bool {
    jogo.inventario.iter().any(|i| i == item)
        || jogo
            .mapa
            .get(&jogo.sala_atual)
            .is_some_and(|s| s.itens.iter().any(|i| i == item))
}

fn voltar_ao_jogo(jogo: &mut GameState, ui: &mut TerminalBuffer, media: &mut MediaManager) {
    jogo.estado_atual = "JOGO".to_string();
    views::imprimir_contexto_sala(jogo, ui, media);
}

pub fn iniciar_minigame_jon(jogo: &mut GameState, ui: &mut TerminalBuffer) {
    jogo.estado_atual = "MINIGAME_JON".to_string();
    jogo.jon_passos_dados = 0;
    let mut rng = rand::thread_rng();
    jogo.jon_caminho_certo = (0..4)
        .map(|_| ["f", "e", "d"][rng.gen_range(0..3)].to_string())
        .collect();
    ui.limpar();
    if let Some(arte) = jogo.ascii_arts.get("ARTE_PORCO") {
        ui.animar_com_intervalo(arte, DOS_BRANCO, 0.015);
    }
    ui.animar_com_intervalo("--- A FOME DE JON ---", DOS_VERDE, 0.03);
    ui.exibir("Guie o Porco Jon pelos dutos baseando-se nos seus sentidos.");
    ui.exibir("Comandos: [F] Frente | [E] Esquerda | [D] Direita");
    views::dar_dica_jon(&jogo.jon_caminho_certo[0], ui);
    ui.exibir("Passo 1/4 - Direção (F/E/D): ");
}

fn processar_jon(
    comando: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) {
    let direcao = match comando {
        "f" | "frente" => "f",
        "e" | "esquerda" => "e",
        "d" | "direita" => "d",
        _ => {
            ui.exibir("Direção inválida. Use F, E ou D.");
            return;
        }
    };
    let passo = jogo.jon_passos_dados;
    if jogo
        .jon_caminho_certo
        .get(passo)
        .is_some_and(|esperado| esperado == direcao)
    {
        let progresso = [
            "Jon rasteja em silêncio pelos dutos...",
            "O metal range baixinho sob o peso de Jon.",
            "Jon para, fareja o ar, e continua adiante.",
            "Quase lá. Jon acelera o passo nos dutos apertados.",
        ];
        ui.exibir(progresso[passo]);
        jogo.jon_passos_dados += 1;
        if let Some(proximo) = jogo.jon_caminho_certo.get(passo + 1) {
            views::dar_dica_jon(proximo, ui);
            ui.exibir(&format!("Passo {}/4 - Direção (F/E/D): ", passo + 2));
            return;
        }
        ui.exibir(&format!(
            "\n{DOS_VERDE}Jon encontrou a 'comida'. A tela pinga um pixel vermelho.{RESET}"
        ));
        ui.exibir(&format!(
            "{DOS_VERMELHO}MENSAGEM: 'Eles ainda estão aqui.'{RESET}"
        ));
        jogo.inventario.push("moeda velha".to_string());
        ui.exibir(&format!("{DOS_AMARELO}A maquina cospe uma ficha velha, de 1982, você guarda na sua bolsa.{RESET}"));
    } else {
        let erros = [
            "Jon caiu num triturador ativo, você leva um choque!",
            "Errado. Um fio desencapado solta faísca — você sente o choque na mão.",
            "Jon trava numa curva fechada, e a máquina te dá um aviso doloroso.",
        ];
        ui.exibir(&format!(
            "\n{DOS_VERMELHO}{}{RESET}",
            erros[jogo.jon_erros.min(erros.len() - 1)]
        ));
        jogo.jon_erros += 1;
        if jogo.aplicar_dano(1) {
            views::dar_tela_de_morte(jogo, ui, audio, media);
            return;
        }
    }
    jogo.consumir_luz(1);
    voltar_ao_jogo(jogo, ui, media);
}

pub fn iniciar_minigame_consertos(jogo: &mut GameState, ui: &mut TerminalBuffer) {
    if let Some(indice) = jogo
        .inventario
        .iter()
        .position(|item| item == "moeda velha")
    {
        jogo.inventario.remove(indice);
    }
    jogo.consertos_escolhas.clear();
    jogo.estado_atual = "MINIGAME_CONSERTOS_CABECA".to_string();
    ui.limpar();
    if let Some(arte) = jogo.ascii_arts.get("ARTE_ROBO") {
        ui.animar_com_intervalo(arte, DOS_BRANCO, 0.015);
    }
    ui.animar_com_intervalo("--- CONSERTOS & SORRISOS ---", DOS_VERDE, 0.03);
    ui.exibir("Bem-vindo, novo Mecânico. Vamos montar nosso novo Festeiro.");
    ui.exibir(&format!(
        "\n{DOS_AMARELO}[ FASE 1: SELEÇÃO DE PEÇAS ]{RESET}"
    ));
    ui.exibir("Escolha a Cabeça (1- Urso | 2- Coelho): ");
}

fn processar_consertos(
    comando: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    media: &mut MediaManager,
) {
    match jogo.estado_atual.as_str() {
        "MINIGAME_CONSERTOS_CABECA" => {
            jogo.consertos_escolhas
                .insert("cabeca".to_string(), comando.to_string());
            jogo.estado_atual = "MINIGAME_CONSERTOS_TRONCO".to_string();
            let nome = if comando == "1" { "Urso" } else { "Coelho" };
            ui.exibir(&format!("{DOS_VERDE}Você encaixa a cabeça de {nome}. Os olhos de vidro piscam uma vez, sozinhos.{RESET}"));
            ui.exibir(&format!(
                "\n{DOS_AMARELO}[ FASE 2: SELEÇÃO DE PEÇAS ]{RESET}"
            ));
            ui.exibir("Escolha o Tronco (1- Fino | 2- Robusto): ");
        }
        "MINIGAME_CONSERTOS_TRONCO" => {
            jogo.estado_atual = "MINIGAME_CONSERTOS_PERNAS".to_string();
            let nome = if comando == "2" { "Robusto" } else { "Fino" };
            ui.exibir(&format!(
                "{DOS_VERDE}O tronco {nome} se encaixa com um estalo metálico.{RESET}"
            ));
            ui.exibir(&format!(
                "\n{DOS_AMARELO}[ FASE 3: SELEÇÃO DE PEÇAS ]{RESET}"
            ));
            ui.exibir("Escolha as Pernas (1- Aço | 2- Pelúcia): ");
        }
        _ => {
            let nome = if comando == "2" { "Pelúcia" } else { "Aço" };
            ui.exibir(&format!(
                "{DOS_VERDE}As pernas de {nome} travam no lugar. A montagem está completa.{RESET}"
            ));
            ui.exibir(&format!(
                "\n{DOS_VERDE}CONSERTO CONCLUÍDO. O ANIMATRÔNICO SORRI PARA VOCÊ!{RESET}"
            ));
            ui.exibir(&format!(
                "\n{DOS_VERMELHO}ele começa a sangrar, e a gritar no seu ouvido.{RESET}"
            ));
            ui.pausa(2.0);
            ui.buffer_efeito("@@JUMPSCARE@@");
            ui.pausa(1.0);
            if !item_presente(jogo, "chave da cozinha") {
                if jogo.tem_espaco_inventario() {
                    jogo.inventario.extend([
                        "chave da cozinha".to_string(),
                        "partitura rasgada".to_string(),
                    ]);
                    ui.exibir(&format!(
                        "{DOS_VERDE}⛋ Você obteve: CHAVE DA COZINHA{RESET}"
                    ));
                    ui.exibir(&format!(
                        "{DOS_VERDE}⛋ Você obteve: Partitura Rasgada.{RESET}"
                    ));
                } else if let Some(sala) = jogo.mapa.get_mut(&jogo.sala_atual) {
                    sala.itens.extend([
                        "chave da cozinha".to_string(),
                        "partitura rasgada".to_string(),
                    ]);
                }
            }
            if comando == "2"
                && jogo
                    .consertos_escolhas
                    .get("cabeca")
                    .is_some_and(|p| p == "2")
            {
                entregar_item_ou_chao("remedio", jogo, ui);
            }
            jogo.consumir_luz(1);
            voltar_ao_jogo(jogo, ui, media);
        }
    }
}

pub fn iniciar_minigame_julgamento(jogo: &mut GameState, ui: &mut TerminalBuffer) {
    jogo.estado_atual = "MINIGAME_JULGAMENTO_Q1".to_string();
    jogo.julgamento_pontos = 0;
    jogo.vitimas_restantes = ["angela", "joao", "renato"].map(str::to_string).to_vec();
    ui.limpar();
    if let Some(arte) = jogo.ascii_arts.get("ARTE_PIANO") {
        ui.animar_com_intervalo(arte, DOS_BRANCO, 0.015);
    }
    ui.animar_com_intervalo("--- O JULGAMENTO DO PIANISTA ---", DOS_VERDE, 0.03);
    ui.exibir("O animatrônico desperta. Ele detém todas as respostas.");
    ui.exibir(&format!(
        "\n{DOS_AMARELO}PERGUNTA 1: Em que ano a nossa música parou para sempre?{RESET}"
    ));
}

fn processar_julgamento(
    comando: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    media: &mut MediaManager,
) {
    let (acertou, proximo, pergunta) = match jogo.estado_atual.as_str() {
        "MINIGAME_JULGAMENTO_Q1" => (
            comando == "1995",
            "MINIGAME_JULGAMENTO_Q2",
            "PERGUNTA 2: Qual animatrônico está atrás de você agora?",
        ),
        "MINIGAME_JULGAMENTO_Q2" => (
            comando.contains("caroline") || comando.contains("ela"),
            "MINIGAME_JULGAMENTO_Q3",
            "PERGUNTA 3: Em que ano tudo isso começou?",
        ),
        "MINIGAME_JULGAMENTO_Q3" => (
            comando == "1982",
            "MINIGAME_JULGAMENTO_Q4",
            "PERGUNTA 4: Quem é você?",
        ),
        "MINIGAME_JULGAMENTO_Q4" => (
            comando.contains("rogerio"),
            "MINIGAME_JULGAMENTO_V1",
            "PERGUNTA 5: Quem são as três vítimas? Digite o 1º nome:",
        ),
        _ => {
            processar_nomes_julgamento(comando, jogo, ui, media);
            return;
        }
    };
    jogo.julgamento_pontos += i32::from(acertou);
    views::falar_pianista(acertou, ui);
    jogo.estado_atual = proximo.to_string();
    ui.exibir(&format!("\n{DOS_AMARELO}{pergunta}{RESET}"));
}

fn processar_nomes_julgamento(
    comando: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    media: &mut MediaManager,
) {
    let antes = jogo.vitimas_restantes.len();
    jogo.vitimas_restantes
        .retain(|vitima| !comando.contains(vitima));
    views::falar_pianista(jogo.vitimas_restantes.len() < antes, ui);

    match jogo.estado_atual.as_str() {
        "MINIGAME_JULGAMENTO_V1" => {
            jogo.estado_atual = "MINIGAME_JULGAMENTO_V2".to_string();
            ui.exibir("Digite o 2º nome: ");
        }
        "MINIGAME_JULGAMENTO_V2" => {
            jogo.estado_atual = "MINIGAME_JULGAMENTO_V3".to_string();
            ui.exibir("Digite o 3º nome: ");
        }
        _ if jogo.vitimas_restantes.is_empty() && jogo.julgamento_pontos == 4 => {
            ui.animar_com_intervalo(
                "Obrigado por voltar pela gente, Rogério...",
                DOS_VERDE,
                0.08,
            );
            ui.exibir(&format!("{DOS_VERDE}⛋ Um compartimento do piano se abre. Você obteve o CARTÃO DE SEGURANÇA{RESET}"));
            jogo.inventario.push("cartao de seguranca".to_string());
            if !item_presente(jogo, "bateria nova") {
                entregar_item_ou_chao("bateria nova", jogo, ui);
            }
            voltar_ao_jogo(jogo, ui, media);
        }
        _ => {
            ui.buffer_efeito("@@JUMPSCARE@@");
            ui.pausa(2.0);
            ui.animar_com_intervalo(
                "Quem é você? *A tela desliga* Você não merece nossa ajuda.",
                DOS_VERMELHO,
                0.05,
            );
            ui.pausa(1.0);
            views::dar_tela_kernel_panic(jogo, ui);
        }
    }
}

// =========================================================================
// 5. MINIGAME: LABIRINTO DO MINOTAURO (SALA DE ENERGIA)
// =========================================================================

const MSGS_PERIGO_ESQUERDA: [&str; 3] = [
    "⚠ Você sente um ar pesado em sua esquerda.",
    "⚠ Algo se arrasta lentamente à sua esquerda.",
    "⚠ Um cheiro de metal queimado vem da esquerda.",
];
const MSGS_PERIGO_DIREITA: [&str; 3] = [
    "⚠ Você enxerga um vulto a sua direita.",
    "⚠ Um brilho metálico reflete rapidamente à direita.",
    "⚠ Você ouve unhas riscando a parede à direita.",
];
const MSGS_PERIGO_FRENTE: [&str; 3] = [
    "⚠ Uma mancha negra bloqueia sua frente.",
    "⚠ O ar fica mais frio à sua frente.",
    "⚠ Uma respiração pesada ecoa logo à frente.",
];
const MSGS_PERIGO_TRAS: [&str; 3] = [
    "⚠ Passos pesados são ouvidos atrás de você.",
    "⚠ Algo pesado se arrasta bem atrás de você.",
    "⚠ Uma respiração fria bate na sua nuca.",
];
const MSGS_ESTATICA: [&str; 3] = [
    "[!] O radar está falhando... ruídos de estática por todo lado.",
    "[!] Estática. O radar pisca e não mostra nada por um instante.",
    "[!] Um zumbido elétrico cobre qualquer outro som ao redor.",
];
const MSGS_COLISAO_JOGADOR: [&str; 3] = [
    "⚠ Você dá um passo e bate o rosto em uma carcaça de metal fria. Você recua pelo susto.⚠",
    "⚠ Sua mão encontra algo úmido e frio na escuridão. Você recua na hora.⚠",
    "⚠ Você tropeça em algo pesado no chão e quase cai. Recua assustado.⚠",
];
const MSGS_COLISAO_MONSTRO: [&str; 3] = [
    "⚠ VOCÊ TROMBA COM ALGO METÁLICO NO ESCURO. ELE ESTÁ NA SUA FRENTE⚠",
    "⚠ ALGO ENORME PASSA BEM AO SEU LADO NO ESCURO⚠",
    "⚠ VOCÊ SENTE O CHÃO TREMER PERTO DEMAIS DE VOCÊ⚠",
];

const GRID_X: std::ops::RangeInclusive<i32> = -2..=2;
const GRID_Y: std::ops::RangeInclusive<i32> = 0..=4;
const FUSIVEL: (i32, i32) = (0, 4);
const SAIDA: (i32, i32) = (0, 0);
const DISTANCIA_PERIGO: i32 = 1;

pub fn iniciar_minigame_minotauro(jogo: &mut GameState, ui: &mut TerminalBuffer) {
    let mut rng = rand::thread_rng();
    let mut pontos: Vec<_> = GRID_X
        .flat_map(|x| GRID_Y.map(move |y| (x, y)))
        .filter(|&p| p != FUSIVEL && p != SAIDA)
        .collect();
    pontos.shuffle(&mut rng);
    pontos.truncate(4);
    let labirinto = MinotauroData {
        px: SAIDA.0,
        py: SAIDA.1,
        mx: [-2, 0, 2][rng.gen_range(0..3)],
        my: rng.gen_range(2..=3),
        fusivel: FUSIVEL,
        saida: SAIDA,
        armadilhas: pontos,
        bateria: if jogo.god_mode { 9999 } else { 30 },
        tem_fios: false,
        tesoura_no_chao: true,
        celular_inspecionado: false,
    };
    jogo.estado_atual = "MINIGAME_MINOTAURO".to_string();
    ui.exibir(&format!("\n{}", "=".repeat(50)));
    ui.exibir("Você entra na Sala de Energia... e a pesada porta de metal bate atrás de você.");
    ui.exibir("Você escuta uma respiração pesada.");
    ui.exibir("Ele está aqui.");
    ui.pausa(2.0);
    imprimir_status_minotauro(&labirinto, ui);
    jogo.minotauro_dados = Some(labirinto);
}

fn distancia_minotauro(m: &MinotauroData) -> i32 {
    (m.px - m.mx).abs() + (m.py - m.my).abs()
}

fn imprimir_status_minotauro(m: &MinotauroData, ui: &mut TerminalBuffer) {
    ui.limpar();
    let distancia = distancia_minotauro(m);
    let mut rng = rand::thread_rng();
    let glitch = distancia == DISTANCIA_PERIGO && rng.gen::<f64>() < 0.2;
    let borda = format!("{DOS_VERDE}  +{}{RESET}", "=======+".repeat(5));
    ui.exibir(&borda);
    for y in GRID_Y.rev() {
        let mut linha = format!("{DOS_VERDE}  /{RESET}");
        for x in GRID_X {
            let pos = (x, y);
            let adjacente = (x - m.px).abs() + (y - m.py).abs() == 1;
            let (simbolo, cor) = if pos == (m.px, m.py) {
                ("@", DOS_BRANCO)
            } else if (glitch && adjacente)
                || (!glitch && distancia <= DISTANCIA_PERIGO && pos == (m.mx, m.my))
            {
                ("X", DOS_VERMELHO)
            } else if m.armadilhas.contains(&pos) {
                ("*", DOS_AMARELO)
            } else if !m.tem_fios && pos == m.fusivel {
                ("F", DOS_AMARELO)
            } else if pos == m.saida {
                ("S", DOS_VERDE)
            } else {
                (" ", DOS_BRANCO)
            };
            linha.push_str(&format!("   {cor}{simbolo}{RESET}   {DOS_VERDE}/{RESET}"));
        }
        ui.exibir(&linha);
        ui.exibir(&format!("{DOS_VERDE}  /{}{RESET}", "       /".repeat(5)));
        ui.exibir(&borda);
    }
    ui.exibir(&format!("\n{}", "-".repeat(30)));
    let bateria = if m.bateria > 100 {
        "∞".to_string()
    } else {
        m.bateria.to_string()
    };
    ui.exibir(&format!(" Bateria da Lanterna: {bateria} turnos restantes"));

    if distancia > DISTANCIA_PERIGO {
        ui.animar(
            "[v] O radar não detecta nada próximo. Silêncio.",
            DOS_BRANCO,
        );
    } else if distancia == DISTANCIA_PERIGO {
        let mensagens = if glitch {
            &MSGS_ESTATICA
        } else if m.mx < m.px {
            &MSGS_PERIGO_ESQUERDA
        } else if m.mx > m.px {
            &MSGS_PERIGO_DIREITA
        } else if m.my > m.py {
            &MSGS_PERIGO_FRENTE
        } else {
            &MSGS_PERIGO_TRAS
        };
        let mensagem = mensagens[rng.gen_range(0..mensagens.len())];
        if glitch {
            ui.exibir(&format!("{DOS_VERMELHO}{mensagem}{RESET}"));
        } else {
            ui.animar(mensagem, DOS_VERMELHO);
        }
    }

    let mut opcoes =
        "ir frente (f) | ir trás (a) | ir esquerda (e) | ir direita (d) | esperar".to_string();
    if (m.px, m.py) == m.fusivel && !m.tem_fios {
        ui.exibir(&format!(
            "\n{DOS_AMARELO} ↯ Você está de frente para a caixa de fusíveis!{RESET}"
        ));
        if m.tesoura_no_chao {
            ui.animar(" ✂ Há uma tesoura caída no chão.", DOS_BRANCO);
            opcoes.push_str(" | pegar tesoura");
        }
        opcoes.push_str(" | cortar fios");
    }
    if m.tem_fios {
        ui.exibir(&format!("\n{DOS_VERMELHO} ↯ FIOS CORTADOS! A SALA ESTÁ DESMORONANDO! FUJA PARA A SAÍDA (S)!{RESET}"));
        if (m.px, m.py) == m.saida {
            ui.exibir(&format!(
                "{DOS_VERDE} ⍍ Você chegou na porta de entrada!{RESET}"
            ));
            opcoes.push_str(" | sair");
        }
    }
    ui.exibir(&format!("\n[{opcoes}]"));
}

fn passo_minotauro(m: &mut MinotauroData, chance_erro: f64, rng: &mut impl Rng) {
    if rng.gen::<f64>() < chance_erro {
        let dx = rng.gen_range(-1..=1);
        let dy = rng.gen_range(-1..=1);
        m.mx += dx;
        m.my += if dx == 0 && dy == 0 { 1 } else { dy };
        return;
    }
    let dx = m.px - m.mx;
    let dy = m.py - m.my;
    if dx != 0 && (dy == 0 || rng.gen::<bool>()) {
        m.mx += dx.signum();
    } else {
        m.my += dy.signum();
    }
}

fn mover_minotauro(m: &mut MinotauroData, comando: &str, chance_sprint: f64, rng: &mut impl Rng) {
    let chance_erro = match comando {
        "esperar" => 0.75,
        "pegar tesoura" | "cortar fios" if distancia_minotauro(m) <= 2 => 0.90,
        _ => 0.25,
    };
    passo_minotauro(m, chance_erro, rng);
    if distancia_minotauro(m) > 1 && rng.gen::<f64>() < chance_sprint {
        passo_minotauro(m, chance_erro / 2.0, rng);
    }
    m.mx = m.mx.clamp(*GRID_X.start(), *GRID_X.end());
    m.my = m.my.clamp(*GRID_Y.start(), *GRID_Y.end());
}

enum ResultadoLabirinto {
    Continuar,
    Vitoria,
    Morte,
}

fn resolver_colisao_minotauro(
    m: &mut MinotauroData,
    distancia_antes: i32,
    posicao_anterior: (i32, i32),
    jogador_moveu: bool,
    god_mode: bool,
    ui: &mut TerminalBuffer,
    rng: &mut impl Rng,
) -> Option<ResultadoLabirinto> {
    if (m.px, m.py) != (m.mx, m.my) {
        return None;
    }
    if distancia_antes > DISTANCIA_PERIGO {
        let mensagens = if jogador_moveu {
            (m.px, m.py) = posicao_anterior;
            &MSGS_COLISAO_JOGADOR
        } else {
            (m.mx, m.my) = posicao_anterior;
            &MSGS_COLISAO_MONSTRO
        };
        ui.exibir(&format!(
            "\n{DOS_VERMELHO}{}{RESET}",
            mensagens[rng.gen_range(0..mensagens.len())]
        ));
        ui.pausa(2.0);
        return Some(ResultadoLabirinto::Continuar);
    }
    if god_mode {
        ui.exibir(&format!("\n{DOS_AMARELO}[GOD MODE] O Minotauro tenta te atacar, mas é repelido por um escudo de energia! Ele desiste e foge.{RESET}"));
        ui.pausa(1.5);
        return Some(ResultadoLabirinto::Vitoria);
    }
    if jogador_moveu {
        ui.exibir("\n Você andou direto para as mãos do monstro no escuro...");
    } else {
        ui.exibir(&format!("\n{DOS_VERMELHO} ☠ O Minotauro te encontrou no escuro. Mãos frias de metal te rasgam por inteiro ☠{RESET}"));
    }
    ui.buffer_efeito("@@JUMPSCARE@@");
    ui.pausa(1.5);
    if jogador_moveu {
        ui.exibir("\n No vazio, você morre sozinho, sem poder salvar ninguém. ");
    }
    Some(ResultadoLabirinto::Morte)
}

fn turno_minotauro(
    comando: &str,
    m: &mut MinotauroData,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
) -> ResultadoLabirinto {
    if [
        "celular quebrado",
        "ver celular quebrado",
        "olhar celular quebrado",
        "examinar celular quebrado",
        "investigar celular quebrado",
        "celular",
    ]
    .contains(&comando)
    {
        if (m.px, m.py) == m.fusivel {
            let descricao = jogo
                .mapa
                .get("sala de energia")
                .and_then(|s| s.inspecionaveis.get("celular quebrado"))
                .map(String::as_str)
                .unwrap_or("Parece ser dela...");
            ui.exibir(&format!("\n{DOS_AMARELO}🔎 {descricao}{RESET}"));
        } else {
            ui.exibir("O celular quebrado está no fundo da sala (na parede central).");
        }
        return ResultadoLabirinto::Continuar;
    }
    if jogo.god_mode && matches!(comando, "atacar" | "bater" | "chutar" | "lutar") {
        ui.exibir(&format!("{DOS_AMARELO}[GOD MODE] Você corre na direção do Minotauro e dá uma voadora com os dois pés no peito dele!{RESET}"));
        ui.exibir(&format!("{DOS_AMARELO}A fera despenca para trás, choraminga em som de estática e foge rompendo as paredes.{RESET}"));
        ui.pausa(1.5);
        return ResultadoLabirinto::Vitoria;
    }

    let distancia = distancia_minotauro(m);
    let jogador_antes = (m.px, m.py);
    let mut gastou_turno = true;
    match comando {
        "ir esquerda" | "e" if m.px > *GRID_X.start() => m.px -= 1,
        "ir direita" | "d" if m.px < *GRID_X.end() => m.px += 1,
        "ir esquerda" | "e" | "ir direita" | "d" => ui.exibir("Você bate a cara na parede..."),
        "ir frente" | "f" if m.py < *GRID_Y.end() => m.py += 1,
        "ir frente" | "f" => ui.exibir("Você bateu na parede do fundo..."),
        "ir tras" | "ir atras" | "a" | "t" if m.py > *GRID_Y.start() => m.py -= 1,
        "ir tras" | "ir atras" | "a" | "t" => ui.exibir("Você bate as costas na porta de metal. Ela não abre..."),
        "esperar" => ui.exibir("Você fica imóvel aguardando..."),
        "pegar tesoura" if (m.px, m.py) == m.fusivel && m.tesoura_no_chao => {
            m.tesoura_no_chao = false;
            jogo.inventario.push("tesoura".to_string());
            ui.exibir(" ✂ Você derruba a tesoura sem querer, fazendo um barulho, mas guarda na sua bolsa");
        }
        "pegar tesoura" => {
            ui.exibir("Não tem tesoura aqui.");
            gastou_turno = false;
        }
        "cortar fios" if (m.px, m.py) == m.fusivel && !m.tem_fios => {
            if let Some(indice) = jogo.inventario.iter().position(|item| item == "tesoura") {
                ui.exibir(&format!("\n{DOS_VERMELHO}Você corta os fios principais. Faíscas voam no seu rosto, mas não causam queimaduras.{RESET}"));
                ui.exibir(&format!("{DOS_VERMELHO} ✂ Sua tesoura quebra com a força do choque elétrico{RESET}"));
                ui.exibir(&format!("{DOS_VERMELHO}Ele sabe onde você está.{RESET}"));
                ui.exibir(&format!("{DOS_VERMELHO}CORRA DE VOLTA PARA A PORTA{RESET}"));
                jogo.inventario.remove(indice);
                jogo.inventario.extend(["tesoura quebrada".to_string(), "fios cortados".to_string()]);
                m.tem_fios = true;
                jogo.fios_cortados_inventario = true;
                ui.exibir(&format!("{DOS_AMARELO}Você guarda os 'fios cortados' na mochila.{RESET}"));
            } else {
                ui.exibir("Você precisa de uma tesoura inteira para cortar os fios.");
            }
        }
        "cortar fios" => ui.exibir("Não há mais fios aqui."),
        "sair" if (m.px, m.py) == m.saida => {
            if m.tem_fios && jogo.inventario.iter().any(|i| i == "fios cortados") {
                ui.exibir(&format!("\n{DOS_VERDE}Você se joga contra a maçaneta, abre a porta e a tranca com toda a força! Você sobreviveu!{RESET}"));
                ui.pausa(1.5);
                return ResultadoLabirinto::Vitoria;
            }
            ui.exibir("Você está na porta de saída. Você precisa cortar e pegar os fios elétricos.");
        }
        "sair" => {
            ui.exibir("A porta de saída não fica aqui. Tente voltar para trás.");
            gastou_turno = false;
        }
        _ => ui.exibir(&format!("{DOS_AMARELO}Comando não reconhecido no escuro. Você gasta segundos tropeçando...{RESET}")),
    }

    if gastou_turno {
        if let Some(indice) = m.armadilhas.iter().position(|&p| p == (m.px, m.py)) {
            m.armadilhas.remove(indice);
            m.bateria -= 3;
            ui.buffer_efeito("@@GLITCH_LUZ@@");
            ui.exibir(&format!("\n{DOS_VERMELHO}Você pisou em fios eletricos. A lanterna pisca e perde muita energia.{RESET}"));
            ui.pausa(1.5);
        }
    }
    let mut rng = rand::thread_rng();
    if let Some(resultado) = resolver_colisao_minotauro(
        m,
        distancia,
        jogador_antes,
        true,
        jogo.god_mode,
        ui,
        &mut rng,
    ) {
        return resultado;
    }
    if !gastou_turno {
        return ResultadoLabirinto::Continuar;
    }
    if !jogo.god_mode {
        m.bateria -= 1;
        if m.bateria <= 0 {
            ui.exibir("\n A sua lanterna apaga, você entra em desespero e bate na bateria fazendo barulho.");
            ui.pausa(2.0);
            ui.buffer_efeito("@@JUMPSCARE@@");
            ui.pausa(2.0);
            ui.exibir(
                "\n Você sente uma mão atravessando seu estômago por trás, não há nada a se fazer.",
            );
            return ResultadoLabirinto::Morte;
        }
    }
    let monstro_antes = (m.mx, m.my);
    mover_minotauro(
        m,
        comando,
        f64::from(jogo.chance_sprint_minotauro) / 100.0,
        &mut rng,
    );
    resolver_colisao_minotauro(
        m,
        distancia,
        monstro_antes,
        false,
        jogo.god_mode,
        ui,
        &mut rng,
    )
    .unwrap_or(ResultadoLabirinto::Continuar)
}

fn processar_minotauro(
    comando: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) {
    let Some(mut m) = jogo.minotauro_dados.take() else {
        iniciar_minigame_minotauro(jogo, ui);
        return;
    };
    match turno_minotauro(comando, &mut m, jogo, ui) {
        ResultadoLabirinto::Continuar => {
            imprimir_status_minotauro(&m, ui);
            jogo.minotauro_dados = Some(m);
        }
        ResultadoLabirinto::Morte => views::dar_tela_de_morte(jogo, ui, audio, media),
        ResultadoLabirinto::Vitoria => {
            desbloquear_conquista("toureiro", jogo, ui, audio);
            jogo.sala_atual = "sala dos fundos".to_string();
            if jogo.god_mode && !jogo.fios_cortados_inventario {
                jogo.adicionar_item_inventario("fios cortados");
                ui.exibir(&format!(
                    "{DOS_AMARELO}[GOD MODE] Você arrancou os 'fios cortados' da parede!{RESET}"
                ));
            }
            if let Some(sala) = jogo.mapa.get_mut("sala dos fundos") {
                sala.conexoes.insert(
                    "energia".to_string(),
                    Some(
                        "A pesada porta da sala de energia está totalmente destruída.".to_string(),
                    ),
                );
            }
            ui.exibir(&format!(
                "{DOS_VERDE}A porta cedeu atrás de você. Você sobreviveu.{RESET}"
            ));
            voltar_ao_jogo(jogo, ui, media);
        }
    }
}

// =========================================================================
// 6. MINIGAME: SALA DE SEGURANÇA (00:00 ÀS 06:00 AM)
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
        "MINIGAME_CONSERTOS_CABECA" | "MINIGAME_CONSERTOS_TRONCO" | "MINIGAME_CONSERTOS_PERNAS" => {
            processar_consertos(comando, jogo, ui, media);
            true
        }
        "MINIGAME_JULGAMENTO_Q1"
        | "MINIGAME_JULGAMENTO_Q2"
        | "MINIGAME_JULGAMENTO_Q3"
        | "MINIGAME_JULGAMENTO_Q4"
        | "MINIGAME_JULGAMENTO_V1"
        | "MINIGAME_JULGAMENTO_V2"
        | "MINIGAME_JULGAMENTO_V3" => {
            processar_julgamento(comando, jogo, ui, media);
            true
        }
        "MINIGAME_MINOTAURO" => {
            processar_minotauro(comando, jogo, ui, audio, media);
            true
        }
        "MINIGAME_SEGURANCA" => {
            crate::seguranca::processar(comando, jogo, ui, audio, media);
            true
        }
        _ => false,
    }
}
