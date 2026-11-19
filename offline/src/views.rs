use crate::audio::AudioManager;
use crate::data::{
    ARTE_ENTRADA, ARTE_FINAL_MEDIOCRE, ARTE_FINAL_NEUTRO, ARTE_FINAL_VERDADEIRO, CAVEIRA_MORTE,
};
use crate::media::MediaManager;
use crate::state::GameState;
use crate::ui::{DOS_AMARELO, DOS_BRANCO, DOS_VERDE, DOS_VERMELHO, RESET};
use crate::utils::corromper_texto;
use crate::TerminalBuffer;
use rand::seq::SliceRandom;
use rand::Rng;

/// Port 1:1 de `imprimir_tela_boot` do views.py
pub fn imprimir_tela_boot(ui: &mut TerminalBuffer) {
    ui.limpar();
    ui.animar_com_intervalo("VILLAS-BOAS INDUSTRIES (C) 1982", DOS_BRANCO, 0.04);
    ui.animar_com_intervalo("BIOS VERSION 1.04 - RELEASE 02/11/1982", DOS_BRANCO, 0.04);
    ui.animar_com_intervalo("RAM CHECK: 640KB OK", DOS_VERDE, 0.03);
    ui.animar_com_intervalo("DRIVE A: READY", DOS_VERDE, 0.03);
    ui.animar_com_intervalo("CARREGANDO 'COMMAND.COM'....... OK\n", DOS_VERDE, 0.06);
    ui.exibir(&format!(
        "{DOS_VERDE}Digite {DOS_BRANCO}dir{DOS_VERDE} para acessar os diretórios:{RESET}"
    ));
}

/// Port 1:1 de `imprimir_menu_dificuldade` do views.py
pub fn imprimir_menu_dificuldade(
    ui: &mut TerminalBuffer,
    tem_autosave: bool,
    finais_alcancados: &[String],
) {
    ui.animar(
        "==================================================",
        DOS_VERDE,
    );
    ui.animar(
        "__     _____ _     _        _ ____   ___   ____ ",
        DOS_VERDE,
    );
    ui.animar(
        r"\ \   / /_ _| |   | |      / / ___| / _ \ / ___|",
        DOS_VERDE,
    );
    ui.animar(
        r" \ \ / / | || |   | |     / /\___ \| | | |\___ \\",
        DOS_VERDE,
    );
    ui.animar(
        r"  \ V /  | || |___| |___ / /  ___) | |_| | ___) |",
        DOS_VERDE,
    );
    ui.animar(
        r"   \_/  |___|_____|_____/_/  |____/ \___/ |____/",
        DOS_VERDE,
    );
    ui.animar(
        "==================================================",
        DOS_VERDE,
    );
    ui.animar(
        "        SISTEMA DE SEGURANÇA INTEGRADO v1.0       \n",
        DOS_BRANCO,
    );

    let c_med = if finais_alcancados.iter().any(|c| c == "mediocre") {
        "[X]"
    } else {
        "[ ]"
    };
    let c_son = if finais_alcancados.iter().any(|c| c == "bons_sonhos") {
        "[X]"
    } else {
        "[ ]"
    };
    let c_bom = if finais_alcancados
        .iter()
        .any(|c| c == "bom" || c == "final_bom")
    {
        "[X]"
    } else {
        "[ ]"
    };
    let c_ver = if finais_alcancados
        .iter()
        .any(|c| c == "verdadeiro" || c == "final_verdadeiro")
    {
        "[X]"
    } else {
        "[ ]"
    };

    let mut qtd = 0;
    if c_med == "[X]" {
        qtd += 1;
    }
    if c_son == "[X]" {
        qtd += 1;
    }
    if c_bom == "[X]" {
        qtd += 1;
    }
    if c_ver == "[X]" {
        qtd += 1;
    }

    ui.animar(
        &format!("{}♔ FINAIS ALCANÇADOS: {}/4{}", DOS_AMARELO, qtd, RESET),
        DOS_BRANCO,
    );
    ui.animar(
        &format!(
            "{}{} Medíocre  {} Bons Sonhos  {} neutro  {} Verdadeiro{}\n",
            DOS_BRANCO, c_med, c_son, c_bom, c_ver, RESET
        ),
        DOS_BRANCO,
    );

    ui.animar(
        &format!(
            "{}[1] INICIAR: MODO NORMAL (Velocidade MS-DOS Padrão){}",
            DOS_BRANCO, RESET
        ),
        DOS_BRANCO,
    );
    ui.animar(
        &format!(
            "{}[2] INICIAR: MODO NORMAL (Texto Rápido — Sem Delays){}",
            DOS_AMARELO, RESET
        ),
        DOS_BRANCO,
    );
    ui.animar(
        &format!(
            "{}[3] INICIAR: MODO PESADELO (Velocidade MS-DOS Padrão){}",
            DOS_VERMELHO, RESET
        ),
        DOS_BRANCO,
    );
    ui.animar(
        &format!(
            "{}[4] INICIAR: MODO PESADELO (Texto Rápido — Sem Delays){}",
            DOS_AMARELO, RESET
        ),
        DOS_BRANCO,
    );
    ui.animar(
        &format!("{}[5] INICIAR: MODOS DE DESAFIO{}", DOS_AMARELO, RESET),
        DOS_BRANCO,
    );
    ui.animar(
        &format!("{}[6] INICIAR: MODO PERSONALIZADO{}", DOS_AMARELO, RESET),
        DOS_BRANCO,
    );

    if tem_autosave {
        ui.animar(
            &format!(
                "{}[7] CONTINUAR JOGO (Autosave Encontrado){}\n",
                DOS_VERDE, RESET
            ),
            DOS_BRANCO,
        );
        ui.animar(
            &format!("{}SELECIONE UMA OPÇÃO (1-7): {}", DOS_VERDE, RESET),
            DOS_BRANCO,
        );
    } else {
        ui.animar(
            &format!("\n{}SELECIONE UMA OPÇÃO (1-6): {}", DOS_VERDE, RESET),
            DOS_BRANCO,
        );
    }
}

/// Port 1:1 de `imprimir_tutorial` do views.py
pub fn imprimir_tutorial(ui: &mut TerminalBuffer, _jogo: &GameState) {
    ui.animar_com_intervalo(
        &format!("\n{DOS_AMARELO}=================================================={RESET}"),
        DOS_BRANCO,
        0.005,
    );
    ui.animar_com_intervalo(
        &format!("{DOS_AMARELO}   [ MANUAL DE SOBREVIVÊNCIA DO SISTEMA v1.0 ]    {RESET}"),
        DOS_BRANCO,
        0.01,
    );
    ui.animar_com_intervalo(
        &format!("{DOS_AMARELO}=================================================={RESET}"),
        DOS_BRANCO,
        0.005,
    );
    ui.animar_com_intervalo(
        &format!("{DOS_BRANCO}>> NAVEGAÇÃO:{RESET}"),
        DOS_BRANCO,
        0.01,
    );
    ui.animar_com_intervalo(&format!("   Mova-se usando direções ({DOS_VERDE}ir frente{RESET}, {DOS_VERDE}ir esquerda{RESET})"), DOS_BRANCO, 0.01);
    ui.animar_com_intervalo(
        &format!("   ou digite o destino exato: {DOS_VERDE}ir sala de jantar{RESET}."),
        DOS_BRANCO,
        0.01,
    );
    ui.animar_com_intervalo(
        &format!("\n{DOS_BRANCO}>> INTERAÇÃO:{RESET}"),
        DOS_BRANCO,
        0.01,
    );
    ui.animar_com_intervalo(
        &format!("   Examine os detalhes ao redor com {DOS_VERDE}olhar [objeto]{RESET}."),
        DOS_BRANCO,
        0.01,
    );
    ui.animar_com_intervalo(&format!("   Para interagir, use {DOS_VERDE}pegar [item]{RESET} ou {DOS_VERDE}usar [item]{RESET}."), DOS_BRANCO, 0.01);
    ui.animar_com_intervalo(
        &format!("\n{DOS_BRANCO}>> GERENCIAMENTO:{RESET}"),
        DOS_BRANCO,
        0.01,
    );
    ui.animar_com_intervalo(&format!("   Digite {DOS_VERDE}inventario{RESET} (ou apenas {DOS_VERDE}i{RESET}) para checar seus bolsos."), DOS_BRANCO, 0.01);
    ui.animar_com_intervalo(
        &format!("   Dica: Atalhos funcionam. Digite {DOS_VERDE}p chave{RESET} para pegar rápido."),
        DOS_BRANCO,
        0.01,
    );
    ui.animar_com_intervalo(
        &format!("\n{DOS_VERMELHO}>> AVISO DE SEGURANÇA:{RESET}"),
        DOS_BRANCO,
        0.01,
    );
    ui.animar_com_intervalo(
        &format!("   Tropeçar no escuro ou errar comandos gera {DOS_VERMELHO}barulho{RESET}."),
        DOS_BRANCO,
        0.01,
    );
    ui.animar_com_intervalo(&format!("   Ela escuta. Se precisar fugir de uma sala, digite {DOS_VERDE}correr [direção]{RESET}."), DOS_BRANCO, 0.01);
    ui.animar_com_intervalo(&format!("   Ouça os passos e o ranger das grades. Para sumir, {DOS_VERDE}esconder{RESET}; para baixar o barulho, {DOS_VERDE}esperar{RESET}."), DOS_BRANCO, 0.01);
    ui.animar_com_intervalo(&format!("\n{DOS_AMARELO}Caso precise, digite {DOS_VERDE}ajuda{DOS_AMARELO} para o guia completo novamente.{RESET}"), DOS_BRANCO, 0.01);
    ui.animar_com_intervalo(
        &format!("{DOS_AMARELO}=================================================={RESET}\n"),
        DOS_BRANCO,
        0.005,
    );
    ui.animar_com_intervalo(ARTE_ENTRADA, DOS_BRANCO, 0.005);
}

/// Port 1:1 de `dar_dica_jon` do views.py
/// Exibe a dica do passo; retorna `true` se a dica exibida foi falsa.
pub fn dar_dica_jon(passo_certo: &str, ui: &mut TerminalBuffer) -> bool {
    let dicas = [
        ("f", "Uma corrente de ar gelado bate direto no seu rosto."),
        (
            "e",
            "Um som agudo de metal arranhando reverbera pela parede canhota do duto.",
        ),
        (
            "d",
            "O cheiro podre de carne estragada fica mais forte no caminho destro.",
        ),
    ];
    let mut rng = rand::thread_rng();
    if rng.gen::<f64>() <= 0.25 {
        let falsas: Vec<&str> = dicas
            .iter()
            .filter(|(k, _)| *k != passo_certo)
            .map(|(_, v)| *v)
            .collect();
        if let Some(escolhida) = falsas.choose(&mut rng) {
            ui.exibir(&format!(
                "\n{}[SENSÓRIO CONFUSO]: {}{}",
                DOS_VERMELHO, escolhida, RESET
            ));
        }
        true
    } else {
        if let Some((_, certa)) = dicas.iter().find(|(k, _)| *k == passo_certo) {
            ui.exibir(&format!("\n{}[SENSÓRIO]: {}{}", DOS_AMARELO, certa, RESET));
        }
        false
    }
}

/// Port 1:1 de `falar_pianista` do views.py
pub fn falar_pianista(acertou: bool, ui: &mut TerminalBuffer) {
    let mut rng = rand::thread_rng();
    if acertou {
        ui.exibir(&format!(
            "{}A máquina toca uma nota suave e agradável.{}",
            DOS_BRANCO, RESET
        ));
        let falas = [
            "Você lembra bem,",
            "Você ainda tem ouvido para isso.",
            "Correto. Nós sempre soubemos que você voltaria.",
            "Sim... como aconteceu.",
        ];
        if let Some(f) = falas.choose(&mut rng) {
            ui.animar_com_intervalo(&format!("\"{}\"", f), DOS_AMARELO, 0.04);
        }
    } else {
        ui.exibir(&format!("{}Acorde dissonante.{}", DOS_VERMELHO, RESET));
        let falas = [
            "Errado. Eles não perdoam mentiras.",
            "Você deveria lembrar melhor do que isso, Rogério.",
            "Uma nota fora do lugar... como você.",
            "Isso não é o que consta no registro.",
        ];
        if let Some(f) = falas.choose(&mut rng) {
            ui.animar_com_intervalo(&format!("\"{}\"", f), DOS_AMARELO, 0.04);
        }
    }
}

/// Port 1:1 de `dar_tela_kernel_panic` do views.py
pub fn dar_tela_kernel_panic(jogo: &mut GameState, ui: &mut TerminalBuffer) {
    jogo.estado_atual = "FIM".to_string();
    ui.limpar();
    ui.buffer_efeito("@@PANIC@@");
    const ARTE_PANIC: &str = "\n    .--.\n   |0_0 |     -\n   |:_/ |    | |  \n  //    \\\\   | |\n (|      |)  | |\n/'\\_   _/`\\  ( )\n\\___)=(___/  \n\n    ";
    ui.animar_com_intervalo(ARTE_PANIC, DOS_BRANCO, 0.0);
    ui.animar_com_intervalo(
        "KERNEL PANIC - NOT SYNCING: Fatal exception in interrupt",
        DOS_BRANCO,
        0.0,
    );
    ui.animar_com_intervalo(
        "Hardware CPU 0: corrupted by external psychological entity.",
        DOS_BRANCO,
        0.0,
    );
    ui.animar_com_intervalo(
        "Process CAROLINE.SYS (pid: 1982, stackpage=0x00000000)",
        DOS_BRANCO,
        0.0,
    );
    ui.animar_com_intervalo("Call Trace:", DOS_BRANCO, 0.0);
    ui.animar_com_intervalo(" [<c010a245>] show_stack+0x7a/0x90", DOS_BRANCO, 0.0);
    ui.animar_com_intervalo(" [<c010a3c4>] show_registers+0x15a/0x1dc", DOS_BRANCO, 0.0);
    ui.animar_com_intervalo(
        " [<c010c140>] die_if_no_escape+0x12a/0x240",
        DOS_BRANCO,
        0.0,
    );
    ui.animar_com_intervalo(
        " [<c0000000>] DO_NOT_LOOK_BEHIND_YOU+0x666/0x666",
        DOS_BRANCO,
        0.0,
    );
    ui.exibir("\nMemory corruption detected. Dumping physical memory to disk...");
    ui.pausa(2.0);
    ui.exibir("Dump failed. The soul is too heavy.");
    ui.exibir("\nSystem halted. Press F5 to attempt a manual reboot.");
}

/// Port 1:1 de `imprimir_contexto_sala` do views.py
pub fn imprimir_contexto_sala(jogo: &GameState, ui: &mut TerminalBuffer, media: &mut MediaManager) {
    if jogo.estado_atual != "JOGO" {
        return;
    }
    let Some(sala) = jogo.mapa.get(&jogo.sala_atual) else {
        return;
    };

    if let Some(ref img_sala) = sala.imagem {
        let _ = media.acionar_imagem(img_sala, false, false);
    } else {
        let _ = media.acionar_imagem(&jogo.sala_atual, false, false);
    }

    ui.exibir(&format!("\n{}", "=".repeat(50)));
    ui.animar(
        &format!("⚇ VOCÊ ESTÁ EM: {}", jogo.sala_atual.to_uppercase()),
        DOS_VERDE,
    );

    if jogo.amanheceu {
        ui.animar(
            &format!(
                "{} ☀ A luz pálida da manhã ilumina a sala através das frestas.{}",
                DOS_BRANCO, RESET
            ),
            DOS_BRANCO,
        );
    }

    let mut descricao_colorida = sala.descricao.clone();
    let mut intensidade_glitch = 0.0;

    if jogo.hp <= 1 && !jogo.god_mode {
        intensidade_glitch += 0.3;
        ui.animar(
            &format!(
                "{}[SISTEMA NEUROLÓGICO COMPROMETIDO]{}",
                DOS_VERMELHO, RESET
            ),
            DOS_VERMELHO,
        );
    }

    if jogo.nivel_barulho > 50 {
        intensidade_glitch += (jogo.nivel_barulho - 50) as f64 / 125.0;
    }

    if jogo.turnos_luz <= 0 && !jogo.amanheceu {
        intensidade_glitch += 0.5;
        ui.animar(
            &format!(
                "{}Sua visão falha na escuridão absoluta...{}",
                DOS_VERMELHO, RESET
            ),
            DOS_BRANCO,
        );
    }

    if intensidade_glitch > 0.0 && !jogo.god_mode {
        descricao_colorida = corromper_texto(&descricao_colorida, intensidade_glitch.min(0.9));
    }

    for inspecionavel in sala.inspecionaveis.keys() {
        descricao_colorida = descricao_colorida.replace(
            inspecionavel,
            &format!("{}{}{}", DOS_AMARELO, inspecionavel, DOS_BRANCO),
        );
    }
    for item in &sala.itens {
        descricao_colorida =
            descricao_colorida.replace(item, &format!("{}{}{}", DOS_VERDE, item, DOS_BRANCO));
    }

    ui.animar(&format!("⏿ Visão: {}", descricao_colorida), DOS_BRANCO);

    let inspecionaveis: Vec<_> = sala.inspecionaveis.keys().map(String::as_str).collect();
    if !inspecionaveis.is_empty() {
        ui.animar(
            &format!(
                "☞ Investigar: {}{}{}",
                DOS_AMARELO,
                inspecionaveis.join(", "),
                RESET
            ),
            DOS_BRANCO,
        );
    }

    if !sala.itens.is_empty() {
        if jogo.turnos_luz > 0 || jogo.amanheceu {
            let itens_formatados: Vec<String> = sala
                .itens
                .iter()
                .map(|item| format!("{}{}{}", DOS_VERDE, item, DOS_BRANCO))
                .collect();
            ui.animar(
                &format!("[i] Itens no chão: {}", itens_formatados.join(", ")),
                DOS_BRANCO,
            );
        } else {
            ui.animar(
                &format!(
                    " {}Deve ter algo no chão, mas escuro demais para ver o quê.{}",
                    DOS_BRANCO, RESET
                ),
                DOS_BRANCO,
            );
        }
    }

    let saidas = sala.saidas_listadas();
    if !saidas.is_empty() {
        let nomes_saidas: Vec<String> = saidas
            .into_iter()
            .map(|(k, _)| {
                let mut c = k.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect();
        ui.animar(
            &format!(
                "⏱ Saídas: {}{}{}",
                DOS_AMARELO,
                nomes_saidas.join(", "),
                RESET
            ),
            DOS_BRANCO,
        );
    } else {
        ui.animar(
            &format!(
                "⏱ Saídas: {}Nenhuma saída aparente...{}",
                DOS_VERMELHO, RESET
            ),
            DOS_BRANCO,
        );
    }
}

/// Port 1:1 de `dar_tela_de_morte` do views.py
pub fn dar_tela_de_morte(
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) {
    jogo.sala_atual = "morte".to_string();
    jogo.estado_atual = "FIM".to_string();
    audio.parar_ambiente();
    media.imagem_atual = None;
    ui.buffer_efeito("@@JUMPSCARE@@");
    ui.pausa(2.0);
    ui.animar_com_intervalo(
        &format!("{DOS_VERMELHO}{CAVEIRA_MORTE}{RESET}"),
        DOS_BRANCO,
        0.002,
    );
    ui.animar_com_intervalo("☠ GAME OVER. A NOITE ENGOLIU VOCÊ.", DOS_VERMELHO, 0.05);
    ui.animar_com_intervalo(
        "=== SISTEMA CORROMPIDO. APERTE F5 PARA REINICIAR ===",
        DOS_AMARELO,
        0.05,
    );
}

/// Port 1:1 de `rodar_final` do views.py com todos os 4 finais originais
pub fn rodar_final(
    tipo_final: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) {
    jogo.estado_atual = "FIM".to_string();
    ui.limpar();
    audio.parar_ambiente();
    media.imagem_atual = None;
    let id = match tipo_final {
        "saida" => {
            ui.animar_com_intervalo(
                "Você empurra as pesadas portas de vidro e foge do restaurante...",
                DOS_AMARELO,
                0.08,
            );
            ui.animar_com_intervalo("Você acredita que vai conseguir superar tudo isso. Que pode recomeçar uma nova vida...", DOS_VERMELHO, 0.07);
            ui.pausa(1.5);
            ui.animar_com_intervalo(
                "Você entra no seu carro, liga o motor e acelera para longe dali.",
                DOS_BRANCO,
                0.08,
            );
            ui.animar_com_intervalo(
                "Você quer esquecer dela... Mesmo ela suplicando por ajuda no escuro.",
                DOS_VERMELHO,
                0.08,
            );
            ui.pausa(2.0);
            ui.buffer_efeito("@@GLITCH_LUZ@@");
            ui.animar_com_intervalo(
                "Você olha pelo retrovisor. Algo se mexe no banco de trás.",
                DOS_VERMELHO,
                0.05,
            );
            ui.pausa(1.0);
            ui.buffer_efeito("@@JUMPSCARE@@");
            ui.pausa(2.0);
            ui.buffer_efeito("@@BLACKOUT@@");
            let arte_corrigida = ARTE_FINAL_MEDIOCRE.replace(" ", "\u{00a0}");
            ui.animar_com_intervalo(
                &format!("{DOS_BRANCO}{arte_corrigida}{RESET}"),
                DOS_BRANCO,
                0.015,
            );
            ui.exibir(&format!(
                "\n{DOS_VERMELHO}[ FINAL MEDÍOCRE: ALGO TE SEGUIU... ]{RESET}"
            ));
            "mediocre"
        }
        "cama" => {
            ui.animar_com_intervalo(
                "Você deita na cama velha, ignorando o cheiro de mofo...",
                DOS_AMARELO,
                0.05,
            );
            ui.pausa(1.0);
            ui.buffer_efeito("@@LOW_POWER@@");
            ui.animar_com_intervalo(
                "O corredor fica em silêncio. Você sente o trinco da porta girar lentamente...",
                DOS_VERMELHO,
                0.09,
            );
            ui.animar_com_intervalo(
                "Você finalmente sente ela, o seu cheiro, sua vibração... Seu dispositivo apita...",
                DOS_VERDE,
                0.06,
            );
            ui.pausa(1.5);
            ui.animar_com_intervalo(
                "PRESENÇA ULTERIOR DETECTADA DENTRO DO QUARTO.",
                DOS_VERMELHO,
                0.05,
            );
            ui.pausa(2.0);
            ui.buffer_efeito("@@BLACKOUT@@");
            ui.animar_com_intervalo(
                "Você fecha os olhos. Pensando em não acordar nunca mais.",
                DOS_BRANCO,
                0.1,
            );
            ui.buffer_efeito("@@JUMPSCARE@@");
            ui.exibir(&format!("\n{DOS_BRANCO}[ FINAL BONS SONHOS ]{RESET}"));
            "bons_sonhos"
        }
        "final_bom" => {
            ui.limpar();
            ui.animar_com_intervalo(&format!("{DOS_VERDE} ======================================================================{RESET}"), DOS_BRANCO, 0.03);
            ui.animar_com_intervalo(&format!("{DOS_VERDE}                             [ FINAL NEUTRO ]                               {RESET}"), DOS_BRANCO, 0.04);
            ui.animar_com_intervalo(&format!("{DOS_VERDE} ======================================================================{RESET}"), DOS_BRANCO, 0.03);
            ui.animar_com_intervalo(
                "Você acende o isqueiro e ilumina o local. A luz do fogo traz calma...",
                DOS_VERDE,
                0.04,
            );
            ui.animar_com_intervalo(
                "- Por que não deu certo? O que eu fiz de errado?",
                DOS_AMARELO,
                0.05,
            );
            ui.animar_com_intervalo("- 'Ainda estou aqui...'", DOS_VERMELHO, 0.09);
            ui.animar_com_intervalo("- Amor? É voce? Mesmo???", DOS_AMARELO, 0.05);
            ui.animar_com_intervalo("- 'Eu espero que ainda seja eu...'", DOS_VERMELHO, 0.09);
            ui.animar_com_intervalo("- 'eu Não SinTo maIS nADA...'", DOS_VERMELHO, 0.1);
            ui.animar_com_intervalo(
                "- 'Eu não consegui te salvar, me desculpa...'",
                DOS_AMARELO,
                0.09,
            );
            ui.animar_com_intervalo(
                "- 'Não consegui salvar ninguém, o que eu fiz.'",
                DOS_AMARELO,
                0.09,
            );
            ui.animar_com_intervalo("- 'Me ajUde, EU SINTO FALTA DE CASA.'", DOS_VERMELHO, 0.1);
            ui.animar_com_intervalo("- Caroline... desista desse corpo que não lhe pertence. Siga o rumo das estrelas,f eu não posso fazer mais nada.", DOS_AMARELO, 0.05);
            ui.pausa(2.0);
            ui.animar_com_intervalo("- ... *Caroline abraça Rogério*", DOS_VERMELHO, 0.09);
            ui.animar_com_intervalo(
                "- 'Eu esperO tE eNContrar Lá, meu bem.'",
                DOS_VERMELHO,
                0.09,
            );
            ui.pausa(1.5);
            ui.buffer_efeito("@@GLITCH_LUZ@@");
            ui.pausa(2.0);
            ui.buffer_efeito("@@JUMPSCARE@@");
            ui.pausa(3.0);
            ui.animar_com_intervalo(
                "Você começa a cuspir sangue. A mão metálica dela atravessou seu estômago.",
                DOS_VERMELHO,
                0.05,
            );
            ui.pausa(1.0);
            ui.buffer_efeito("@@LOW_POWER@@");
            ui.animar_com_intervalo("Nada mais faz sentido, sua visão começa a tremer, mas pela última vez, você olha nos olhos dela.", DOS_VERMELHO, 0.04);
            ui.pausa(1.5);
            ui.animar_com_intervalo("- 'Me Perdoe por isso.'", DOS_VERMELHO, 0.09);
            ui.pausa(2.0);
            ui.buffer_efeito("@@BLACKOUT@@");
            ui.animar_com_intervalo(
                "Seu corpo é deixado ao chão, e você se junta a todos os outros naquele lugar.",
                DOS_BRANCO,
                0.09,
            );
            ui.pausa(2.0);
            ui.animar_com_intervalo(
                &format!("{DOS_BRANCO}{ARTE_FINAL_NEUTRO}{RESET}"),
                DOS_BRANCO,
                0.005,
            );
            ui.exibir(&format!("\n{DOS_BRANCO}[ FINAL NEUTRO: ESCURIDÃO ]{RESET}"));
            "bom"
        }
        "verdadeiro" => {
            ui.limpar();
            ui.animar_com_intervalo(&format!("{DOS_VERMELHO}=================================================================={RESET}"), DOS_BRANCO, 0.02);
            ui.animar_com_intervalo(&format!("{DOS_VERMELHO}                        [ FINAL VERDADEIRO ]                      {RESET}"), DOS_BRANCO, 0.03);
            ui.animar_com_intervalo(&format!("{DOS_VERMELHO}=================================================================={RESET}\n"), DOS_BRANCO, 0.02);
            ui.animar_com_intervalo("Você acende seu isqueiro...", DOS_BRANCO, 0.04);
            ui.buffer_efeito("@@LOW_POWER@@");
            ui.animar_com_intervalo(
                "Com a chama ardente, você a joga nas cortinas do hall. O fogo sobe rapidamente.",
                DOS_BRANCO,
                0.04,
            );
            ui.animar_com_intervalo("Você tosse um pouco com a fumaça. Ao retomar o olhar para o fim da sala, você a vê, esperando.", DOS_BRANCO, 0.04);
            ui.animar_com_intervalo("O estalo das chamas consome a madeira podre. A fumaça preta e espessa cobre o teto do restaurante.", DOS_BRANCO, 0.04);
            ui.animar_com_intervalo("O calor é sufocante, mas você não corre para a saída ainda. Você caminha até o centro do hall.\n", DOS_BRANCO, 0.04);
            ui.animar_com_intervalo(
                "Lá está a carcaça de pelúcia rosa, com o metal rangendo e o plástico derretendo.",
                DOS_BRANCO,
                0.04,
            );
            ui.animar_com_intervalo("Você se aproxima do animatrônico... dela. E encaixa os fios na sua fiação exposta...", DOS_BRANCO, 0.05);
            ui.animar_com_intervalo(
                "Os LEDs vermelhos piscam fracos. Ela tenta se mover... mas o mecanismo emperra.\n",
                DOS_BRANCO,
                0.05,
            );
            ui.animar_com_intervalo(
                "- Por que não deu certo? O que eu fiz de errado...?",
                DOS_AMARELO,
                0.05,
            );
            ui.animar_com_intervalo(
                "Sua voz treme. As lágrimas cortam a fuligem no seu rosto.",
                DOS_BRANCO,
                0.04,
            );
            ui.animar_com_intervalo(
                "\nUm zumbido eletrônico arranhado sai do alto-falante no peito da carcaça:",
                DOS_BRANCO,
                0.04,
            );
            ui.animar_com_intervalo(
                "- '... v-você... fez... dar... certo...'",
                DOS_VERMELHO,
                0.08,
            );
            ui.animar_com_intervalo("- '... R-Rogério...?'", DOS_VERMELHO, 0.08);
            ui.animar_com_intervalo("\n- Caro... Caroline? É você?", DOS_AMARELO, 0.05);
            ui.animar_com_intervalo(
                "*(Você abraça a carcaça de pelagem rosa)*",
                DOS_BRANCO,
                0.04,
            );
            ui.animar_com_intervalo(
                "Mesmo com o calor das chamas se espalhando, o metal que te toca é frio e rígido.",
                DOS_BRANCO,
                0.04,
            );
            ui.animar_com_intervalo(
                "Mas você não solta. Você a segura com toda a força que te resta.",
                DOS_BRANCO,
                0.04,
            );
            ui.animar_com_intervalo(
                "- Meu corpo ficou em silêncio... O barulho da minha cabeça, simplesmente parou.",
                DOS_VERMELHO,
                0.07,
            );
            ui.animar_com_intervalo(
                "- A dor nas minhas costas... o ferro rasgando minha pele... sumiu.",
                DOS_VERMELHO,
                0.07,
            );
            ui.animar_com_intervalo("- Eu não sinto mais raiva.", DOS_VERMELHO, 0.07);
            ui.animar_com_intervalo(
                "\n- Eu sinto muito, Caroline... Me desculpa por não ter chegado a tempo...",
                DOS_AMARELO,
                0.05,
            );
            ui.animar_com_intervalo(
                "- Eu... Eu li seus e-mails. Eu tenho te procurado em todo lugar...",
                DOS_AMARELO,
                0.05,
            );
            ui.pausa(1.0);
            ui.animar_com_intervalo("\n*(O fogo se alastra pelo restaurante, aquele lugar maldito vai arder em chamas)*", DOS_BRANCO, 0.04);
            ui.animar_com_intervalo("O crepitar do incêndio abafa os ruídos metálicos dos outros robôs gritando lá no fundo.", DOS_BRANCO, 0.04);
            ui.animar_com_intervalo(
                "O filtro de luz vermelha dos olhos de plástico dela se apaga para sempre.",
                DOS_BRANCO,
                0.04,
            );
            ui.animar_com_intervalo("\nNa penumbra dourada pelas chamas, a voz não vem mais do alto-falante. Ela ecoa suave na sua mente:", DOS_BRANCO, 0.04);
            ui.animar_com_intervalo(
                "- Não peça desculpas, meu amor... Você veio.",
                DOS_VERMELHO,
                0.07,
            );
            ui.animar_com_intervalo("- Durante meses no escuro dessa máquina, eu queria fazer 'eles' pagarem pelo que me fizeram...", DOS_VERMELHO, 0.07);
            ui.animar_com_intervalo(
                "- Mas quando você me abraçou... a escuridão simplesmente evaporou.",
                DOS_VERMELHO,
                0.07,
            );
            ui.animar_com_intervalo("\n- Não podia te deixar presa aqui, Caroline. Hoje todo esse terror feito de máquinas acaba.", DOS_AMARELO, 0.06);
            ui.animar_com_intervalo(
                "- A gente ainda vai viver juntos... Pena que seja em outra vida.",
                DOS_AMARELO,
                0.05,
            );
            ui.animar_com_intervalo(
                "\nUma brisa suave e fresca corta o ar quente do incêndio, envolvendo seu pescoço.",
                DOS_BRANCO,
                0.04,
            );
            ui.animar_com_intervalo("- Me sinta pela última vez.", DOS_VERMELHO, 0.07);
            ui.animar_com_intervalo("\n*(Você sente mãos invisíveis e macias em seus ombros. Toda a dor e a paranoia desaparecem)*", DOS_BRANCO, 0.04);
            ui.animar_com_intervalo(
                "\n- Vai embora agora, viva por nós dois. Eu sempre estarei contigo.",
                DOS_VERMELHO,
                0.07,
            );
            ui.animar_com_intervalo("- Eu te amo.", DOS_AMARELO, 0.06);
            ui.animar_com_intervalo("- Eu também te amo, amor. Pra sempre.", DOS_VERMELHO, 0.06);
            ui.pausa(2.0);
            ui.animar_com_intervalo("\n*(O animatrônico cai no chão, a carcaça é consumida pelas chamas enquanto o teto cede)*", DOS_BRANCO, 0.05);
            ui.animar_com_intervalo(
                &format!("\n{DOS_VERDE}[DISPOSITIVO]: NENHUMA PRESENÇA DETECTADA.{RESET}"),
                DOS_BRANCO,
                0.05,
            );
            ui.pausa(1.5);
            ui.buffer_efeito("@@NORMAL_POWER@@");
            ui.animar_com_intervalo("\nVocê se levanta e empurra as portas de entrada, saindo para o ar frio da madrugada.", DOS_BRANCO, 0.08);
            ui.animar_com_intervalo("Pela calçada de Curitiba, você vê a fumaça subindo ao amanhecer. O restaurante virou cinzas.", DOS_BRANCO, 0.06);
            ui.animar_com_intervalo(
                &format!("{DOS_BRANCO}{ARTE_FINAL_VERDADEIRO}{RESET}"),
                DOS_BRANCO,
                0.005,
            );
            ui.exibir(&format!(
                "\n{DOS_VERDE}[ FINAL VERDADEIRO: LIBERTAÇÃO ]{RESET}"
            ));
            "verdadeiro"
        }
        _ => return,
    };
    if audio.config.registrar_final(id) {
        ui.exibir(&format!(
            "\n{DOS_AMARELO}=================================================={RESET}"
        ));
        ui.animar_com_intervalo(
            "VOCÊ DESVENDOU TODAS AS VERDADES DESTA NOITE.",
            DOS_VERMELHO,
            0.05,
        );
        ui.exibir(&format!(
            "{DOS_AMARELO}DIGITE O ANO EM QUE TUDO ACABOU NA TELA DE MENU: {DOS_BRANCO}2007{RESET}"
        ));
        ui.exibir(&format!(
            "{DOS_AMARELO}=================================================={RESET}"
        ));
    }
    ui.buffer_efeito("@@NORMAL_POWER@@");
    ui.animar_com_intervalo("\n=== APERTE F5 PARA REINICIAR ===", DOS_AMARELO, 0.05);
}
