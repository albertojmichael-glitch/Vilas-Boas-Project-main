use crate::audio::AudioManager;
use crate::data::{ARTE_INDIO, ARTE_MESA_SEGURANCA, ARTE_RADAR, VIDA_GOD_MODE};
use crate::media::MediaManager;
use crate::state::{CaptchaData, GameState, SegurancaData};
use crate::ui::{DOS_AMARELO, DOS_BRANCO, DOS_VERDE, DOS_VERMELHO, RESET};
use crate::{utils, views, TerminalBuffer};
use rand::Rng;

const TURNO_FINAL: i32 = 24;
const TURNO_INICIO_CALOR: i32 = 12;
const TURNO_CRITICO: i32 = 22;
const RICK_PORTA: i32 = 3;
const RICK_ATAQUE: i32 = 4;
const JON_DUTO: i32 = 4;
const JON_ATAQUE: i32 = 5;
const CAROLINE_PORTA: i32 = 5;
const CAROLINE_ATAQUE: i32 = 6;

pub fn iniciar(jogo: &mut GameState, ui: &mut TerminalBuffer) {
    let mut rng = rand::thread_rng();
    let luz = (jogo.turnos_luz - 3).max(0) * 5;
    let baterias = jogo
        .inventario
        .iter()
        .filter(|item| *item == "bateria nova")
        .count() as i32
        * 10;
    let energia_base = if jogo.god_mode {
        VIDA_GOD_MODE
    } else {
        // Saves antigos podem trazer o intervalo invertido.
        let min = jogo.energia_min_noite.min(jogo.energia_max_noite);
        let max = jogo.energia_max_noite.max(jogo.energia_min_noite);
        rng.gen_range(min..=max)
    };
    let seguranca = SegurancaData {
        energia: energia_base + luz + baterias,
        bonus_energia: luz + baterias,
        caroline_no_duto: rng.gen(),
        ..SegurancaData::default()
    };
    ui.exibir(&format!("{DOS_BRANCO}{ARTE_MESA_SEGURANCA}{RESET}"));
    ui.exibir(&format!("\n{}", "=".repeat(50)));
    ui.exibir("Você senta na cadeira da sala de segurança.");
    ui.pausa(1.0);
    jogo.estado_atual = "MINIGAME_SEGURANCA".to_string();
    imprimir_status(&seguranca, jogo.god_mode, ui, &mut rng);
    jogo.seguranca_dados = Some(seguranca);
}

fn imprimir_status(s: &SegurancaData, god_mode: bool, ui: &mut TerminalBuffer, rng: &mut impl Rng) {
    ui.limpar();
    ui.buffer_efeito(match s.energia {
        ..=0 => "@@BLACKOUT@@",
        1..=14 => "@@LOW_POWER@@",
        _ => "@@NORMAL_POWER@@",
    });
    ui.exibir(&format!("\n{}", "=".repeat(50)));
    if s.turno == 0 && s.bonus_energia > 0 {
        ui.exibir(&format!(
            "{DOS_VERDE}[SISTEMA] Baterias extras conectadas. Energia redirecionada: +{}%{RESET}",
            s.bonus_energia
        ));
        ui.exibir(&"=".repeat(50));
    }
    let mut hora = if s.apagao {
        "[SISTEMA DESLIGADO]".to_string()
    } else if s.relogio_quebrado {
        format!("{:02}:??", s.turno / 4)
    } else {
        format!("{:02}:{:02}", s.turno / 4, s.turno % 4 * 15)
    };
    let mut energia = if god_mode {
        "∞".to_string()
    } else {
        format!("{}%", s.energia)
    };
    if s.energia < 15 && !god_mode {
        energia = utils::corromper_texto(&energia, 0.6);
        hora = utils::corromper_texto(&hora, 0.4);
    }
    let chance = (s.caroline_pos * 15).clamp(0, 100);
    let mut bug = |texto: &str| -> String {
        texto
            .chars()
            .map(|c| {
                if c.is_alphabetic() && rng.gen_range(1..=100) <= chance {
                    c.to_uppercase().next().unwrap_or(c)
                } else {
                    c
                }
            })
            .collect()
    };
    ui.exibir(&bug(&format!("RELOGIO: {hora}")));
    ui.exibir(&bug(&format!("ENERGIA: {energia}")));
    ui.exibir(&bug(if s.porta_fechada {
        "PORTA CENTRAL: Fechada"
    } else {
        "PORTA CENTRAL: Aberta"
    }));
    let erros: Vec<_> = [
        (s.camera_quebrada, "CÂMERAS"),
        (s.relogio_quebrado, "RELÓGIO"),
        (s.deteccao_quebrada, "DETECÇÃO"),
    ]
    .into_iter()
    .filter_map(|(quebrado, nome)| quebrado.then_some(nome))
    .collect();
    if erros.is_empty() {
        ui.exibir(&bug("ERROS: Nenhum"));
    } else {
        ui.exibir(&format!("ERROS ATIVOS: {}", erros.join(", ")));
    }
    if s.turnos_gerador > 0 {
        ui.exibir(&format!(
            "{DOS_VERDE}Gerador reserva: Ativo({} turnos restantes){RESET}",
            s.turnos_gerador
        ));
    } else if !s.gerador_usado {
        ui.exibir(&format!("{DOS_AMARELO}Gerador Reserva: Disponível{RESET}"));
    }
    if s.alberto_msg_falsa {
        ui.exibir("\n[MENSAGEM]: ERRO CRÍTICO! FECHAR PORTA AGORA!");
    }
    if s.indio_ativo && !s.deteccao_quebrada {
        ui.exibir(&bug(
            "\nVocê sente como se algo estivesse te olhando pelo vidro...",
        ));
    }
    ui.exibir("\nAção (ouvir | cameras | ver tubulacao | iluminar tubulacao | fechar porta | abrir porta | olhar vidro | ligar gerador | consertar [sistema] | esperar)");
    if let Some(captcha) = &s.captcha_ativo {
        ui.buffer_efeito("@@LOW_POWER@@");
        ui.exibir(&format!(
            "\n{DOS_AMARELO}================ OVERRIDE MANUAL REQUERIDO ================{RESET}"
        ));
        ui.exibir(&format!("{DOS_AMARELO}{}{RESET}", captcha.pergunta));
        ui.exibir(&format!(
            "{DOS_AMARELO}==========================================================={RESET}"
        ));
        ui.exibir("\n[Digite a resposta para continuar]");
    }
}

fn gerar_captcha(rng: &mut impl Rng) -> CaptchaData {
    let (pergunta, resposta) = match rng.gen_range(0..4) {
        0 => {
            let simbolos = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ123456789";
            let codigo: String = (0..6)
                .map(|_| char::from(simbolos[rng.gen_range(0..simbolos.len())]))
                .collect();
            let resposta = codigo
                .chars()
                .rev()
                .collect::<String>()
                .to_ascii_lowercase();
            (format!("Criptografia pesada. Digite {DOS_VERMELHO}{codigo}{DOS_AMARELO} de trás pra frente."), resposta)
        }
        1 => {
            let mut valores: [i32; 4] = std::array::from_fn(|_| rng.gen_range(11..=99));
            let texto = valores.map(|n| n.to_string()).join(" ");
            valores.sort();
            (format!("Falha no sequenciador. Ordene crescentemente: {DOS_VERMELHO}{texto}{DOS_AMARELO} (sem espaços)."), valores.map(|n| n.to_string()).concat())
        }
        2 => {
            let (a, b, c) = (
                rng.gen_range(2..=6),
                rng.gen_range(3..=9),
                rng.gen_range(10..=40),
            );
            (format!("Pico de voltagem. Resolva rápido: {DOS_VERMELHO}({a} * {b}) + {c}{DOS_AMARELO}."), (a * b + c).to_string())
        }
        _ => {
            let simbolos = ['#', '@', '%', '&', '$'];
            let alvo = simbolos[rng.gen_range(0..simbolos.len())];
            let texto: String = (0..16)
                .map(|_| simbolos[rng.gen_range(0..simbolos.len())])
                .collect();
            (format!("Ruído de dados. Quantos símbolos '{DOS_VERMELHO}{alvo}{DOS_AMARELO}' existem em: {DOS_VERMELHO}{texto}{DOS_AMARELO}?"), texto.matches(alvo).count().to_string())
        }
    };
    CaptchaData { pergunta, resposta }
}

impl SegurancaData {
    fn custo_extra(&self) -> i32 {
        match self.turno {
            TURNO_CRITICO.. => 3,
            TURNO_INICIO_CALOR.. => 1,
            _ => 0,
        }
    }

    fn caroline_na_porta(&self) -> bool {
        !self.caroline_no_duto && self.caroline_pos >= CAROLINE_PORTA
    }

    fn resetar_caroline(&mut self, rng: &mut impl Rng) {
        self.caroline_pos = 0;
        self.caroline_no_duto = rng.gen();
    }

    fn falhar_sistema(&mut self, rng: &mut impl Rng) {
        match rng.gen_range(0..3) {
            0 => self.camera_quebrada = true,
            1 => self.relogio_quebrado = true,
            _ => self.deteccao_quebrada = true,
        }
    }

    fn acionar_porta(&mut self, fechar: bool, ui: &mut TerminalBuffer) {
        let custo = 2 + self.custo_extra();
        if self.apagao || self.energia <= custo {
            ui.exibir(if fechar {
                "Sem energia! O botão faz um clique morto."
            } else {
                "Sem energia! A porta não responde."
            });
            return;
        }
        if self.porta_fechada == fechar {
            ui.exibir(if fechar {
                "A porta já está fechada."
            } else {
                "A porta já está aberta."
            });
            return;
        }
        self.porta_fechada = fechar;
        self.energia -= custo;
        ui.buffer_efeito("@@PORTA@@");
        let relato = if fechar {
            "A pesada porta de metal desce com um estrondo."
        } else {
            "A porta de metal se ergue lentamente."
        };
        ui.exibir(&format!("{relato} (-{custo}% Energia)"));
        if fechar && self.alberto_msg_falsa {
            ui.exibir("\n Como você é tão tolo? Hahahaha");
            self.camera_quebrada = true;
            self.deteccao_quebrada = true;
            self.alberto_msg_falsa = false;
        }
    }

    fn iniciar_consulta(&mut self, comando: &str, ui: &mut TerminalBuffer) -> Option<i32> {
        let custo = if comando != "iluminar tubulacao" && self.turnos_gerador > 0 {
            0
        } else {
            1 + self.custo_extra()
        };
        let (offline, fraca, sobrecarga) = match comando {
            "cameras" => (
                "⊠ [SINAL PERDIDO]",
                "Câmeras offline (Bateria fraca).",
                "Monitor superaquecido. A tela exibe apenas estática!",
            ),
            "ouvir" => (
                "No apagão, você ouve sua própria respiração...",
                "Sistema de áudio offline (Bateria fraca).",
                "Placa de áudio em curto. Passe o turno para resfriar!",
            ),
            "ver tubulacao" => (
                "◯ [SENSORES OFFLINE]",
                "Sensores offline (Bateria fraca).",
                "Painel de detecção travado!",
            ),
            _ => (
                "Sem força nas luzes.",
                "Sem força nas luzes.",
                "Muitas requisições simultâneas. Hardware travado",
            ),
        };
        if self.apagao
            || (comando == "cameras" && self.camera_quebrada)
            || (comando == "ver tubulacao" && self.deteccao_quebrada)
        {
            ui.exibir(offline);
        } else if comando == "ouvir" && self.deteccao_quebrada {
            ui.exibir(&format!("{DOS_VERMELHO}⚠ ⚠ ⚠ O alarme estridente de falha nos sensores ecoa na sala. Você não consegue ouvir nada além disso ⚠ ⚠ ⚠{RESET}"));
        } else if self.energia <= custo {
            ui.exibir(fraca);
        } else if self.usos_sistema_turno >= 2 {
            ui.exibir(&format!(
                "{DOS_VERMELHO}⚠ [SISTEMA SOBRECARREGADO]: {sobrecarga}{RESET}"
            ));
            self.energia -= custo;
        } else {
            self.usos_sistema_turno += 1;
            self.energia -= custo;
            return Some(custo);
        }
        None
    }

    fn consultar(&mut self, comando: &str, ui: &mut TerminalBuffer, rng: &mut impl Rng) {
        let Some(custo) = self.iniciar_consulta(comando, ui) else {
            return;
        };
        if comando == "iluminar tubulacao" {
            ui.exibir(&format!(
                "Você liga o projetor nos dutos (-{custo}% Energia)"
            ));
            if self.jon_pos >= JON_DUTO {
                self.jon_pos = 0;
                ui.exibir("Jon recua apressado pela tubulação");
            }
            if self.caroline_no_duto && self.caroline_pos >= 5 {
                self.resetar_caroline(rng);
                ui.exibir("A Caroline fugiu do duto");
            }
            return;
        }

        ui.exibir(&format!("(-{custo}% Energia)"));
        match comando {
            "cameras" => {
                if rng.gen_range(1..=100) <= self.caroline_pos * 10 {
                    ui.exibir("⊠ [SINAL COM INTERFERÊNCIA] Imagens distorcidas...");
                    ui.exibir(&format!("Rick: Setor {}/4 (???)", rng.gen_range(0..=4)));
                    ui.exibir(&format!("Jon: Setor {}/5 (???)", rng.gen_range(0..=5)));
                } else {
                    ui.exibir(&format!(
                        "\n--- FEED DAS CÂMERAS ---\nRick: Setor {}/4",
                        self.rick_pos
                    ));
                    if self.jon_pos < 3 {
                        ui.exibir(&format!("Jon: Setor {}/5", self.jon_pos));
                    } else {
                        ui.exibir("Jon: [não é visivel nas cameras]");
                    }
                }
                ui.exibir("------------------------");
                if rng.gen_range(1..=100) == 1 {
                    ui.exibir(&format!("\n{DOS_VERMELHO}⊠ [ANOMALIA DETECTADA]: O feed pisca. Em uma das câmeras escuras, o rosto quebrado de Caroline encara diretamente a lente... e ela está... sorrindo?{RESET}"));
                }
            }
            "ouvir" => {
                let porta = self.rick_pos >= RICK_PORTA || self.caroline_na_porta();
                let duto =
                    self.jon_pos >= JON_DUTO || (self.caroline_no_duto && self.caroline_pos >= 5);
                if porta {
                    ui.buffer_efeito("@@PASSO@@");
                    ui.exibir(" Passos metálicos pesados são ouvidos do corredor");
                }
                if duto {
                    ui.buffer_efeito("@@PASSO@@");
                    ui.exibir(" Você escuta arranhões e batidas vindo da tubulação");
                }
                if !porta && !duto {
                    ui.exibir("Apenas o zumbido dos fios elétricos e da lâmpada quase apagada.");
                }
            }
            _ => {
                if self.jon_pos >= 3 || (self.caroline_no_duto && self.caroline_pos >= 4) {
                    ui.exibir("⭙ Sensor fica vermelho, há algo nos dutos ⭙");
                } else {
                    ui.exibir("◉ Sensor não detecta nada");
                }
            }
        }
    }

    fn olhar_vidro(&mut self, ui: &mut TerminalBuffer, rng: &mut impl Rng) {
        if self.indio_ativo {
            ui.exibir(&format!("{DOS_BRANCO}{ARTE_INDIO}{RESET}"));
            ui.pausa(2.0);
            ui.exibir("Você não enxerga nada, até que 2 olhos te encaram pela janela, a figura do indio jones faz você perder a cabeça");
            self.falhar_sistema(rng);
            if self.turno < 20 {
                self.indio_ativo = false;
            }
            return;
        }
        ui.exibir(match (self.rick_pos >= RICK_PORTA, self.caroline_na_porta()) {
            (true, true) => "Seu corpo treme. Você vê a carcaça maciça de Rick, o mosqueteiro, e a carcaça de coelho rosa retorcido de Caroline parados lado a lado no corredor, olhando diretamente para a câmera. O vazio nos olhos deles é aterrorizante.",
            (true, false) => " Você olha pelo vidro e vê a silhueta gigantesca do Rick, o mosqueteiro, parado nas sombras. Os olhos de plástico sem vida dele estão focados em você.",
            (false, true) => " Através da sujeira do vidro, você enxerga a carcaça do coelho rosa tentando se esconder nas sombras. Ela está encostada na parede do corredor",
            _ => "Você limpa o embaçado do vidro e força a vista para o corredor escuro. Consegue distinguir as portas fechadas das outras salas, os cartazes rasgados nas paredes e o chão de linóleo imundo refletindo a pouca luz que resta. Nenhum movimento... Além das sombras, há apenas o seu reflexo devolvendo o olhar.",
        });
    }

    fn ligar_gerador(&mut self, ui: &mut TerminalBuffer) -> bool {
        if self.apagao {
            ui.exibir("Tarde demais, o sistema principal já foi totalmente desligado");
            return false;
        }
        if self.gerador_usado {
            ui.exibir(
                "O combustivel do gerador reserva já foi queimado, ele só pode ser usado uma vez",
            );
            return false;
        }
        ui.exibir(&format!("\n{DOS_VERDE} Você aperta o botão do gerador reserva, ele cospe uma fumaça preta, sistemas basicos operando sem custo de energia.{RESET}"));
        self.gerador_usado = true;
        self.turnos_gerador = 2;
        true
    }

    fn iniciar_conserto(&mut self, sistema: &str, ui: &mut TerminalBuffer, rng: &mut impl Rng) {
        let normal = match sistema {
            "camera" if !self.camera_quebrada => {
                Some("Diagnóstico: O sistema de câmeras já está operacional e transmitindo.")
            }
            "deteccao" if !self.deteccao_quebrada => {
                Some("Diagnóstico: O sistema de detecção (radar) já está operando normalmente.")
            }
            "relogio" if !self.relogio_quebrado => {
                Some("Diagnóstico: O relógio já está sincronizado corretamente.")
            }
            _ => None,
        };
        if let Some(normal) = normal {
            ui.exibir(&format!("{DOS_AMARELO}{normal}{RESET}"));
        } else {
            ui.buffer_efeito("@@BSOD@@");
            ui.pausa(2.5);
            self.captcha_ativo = Some(gerar_captcha(rng));
            self.sistema_alvo_conserto = Some(sistema.to_string());
        }
    }

    fn responder_captcha(&mut self, resposta: &str, captcha: CaptchaData, ui: &mut TerminalBuffer) {
        let sistema = self.sistema_alvo_conserto.take().unwrap_or_default();
        if resposta == captcha.resposta {
            ui.exibir(&format!(
                "\n{DOS_VERDE}OVERRIDE ACEITO. Protocolos de segurança validados.{RESET}"
            ));
            ui.exibir(&format!(
                "{DOS_VERDE}Sistema {} reiniciado e operacional.{RESET}",
                sistema.to_uppercase()
            ));
            match sistema.as_str() {
                "camera" => self.camera_quebrada = false,
                "deteccao" => self.deteccao_quebrada = false,
                "relogio" => self.relogio_quebrado = false,
                _ => {}
            }
        } else {
            ui.buffer_efeito("@@GLITCH_LUZ@@");
            ui.exibir(&format!(
                "\n{DOS_VERMELHO}ACESSO NEGADO. Assinatura de erro detectada no input.{RESET}"
            ));
            ui.exibir(&format!("{DOS_VERMELHO}Reboot abortado. A falha de firewall drenou energia do sistema.{RESET}"));
            self.energia = (self.energia - 1).max(0);
        }
        ui.pausa(1.5);
    }

    fn interferencia(&mut self, ui: &mut TerminalBuffer, rng: &mut impl Rng) {
        if rng.gen::<f64>() > 0.10 {
            return;
        }
        match rng.gen_range(0..3) {
            0 => self.rick_pos += 1,
            1 => self.jon_pos += 1,
            _ => self.caroline_pos += 1,
        }
        ui.buffer_efeito("@@PASSO@@");
        ui.exibir(&format!("\n{DOS_VERMELHO}Você ouve um ruído metálico se aproximando enquanto mexe no sistema.{RESET}"));
        if self.turno >= TURNO_CRITICO && rng.gen::<f64>() < 0.3 {
            self.falhar_sistema(rng);
            ui.buffer_efeito("@@GLITCH_LUZ@@");
        }
    }

    fn resolver_invasao(
        &mut self,
        god_mode: bool,
        captcha: bool,
        ui: &mut TerminalBuffer,
        rng: &mut impl Rng,
    ) -> bool {
        if self.porta_fechada {
            if self.rick_pos == RICK_ATAQUE {
                self.rick_pos = 0;
                ui.exibir(
                    "\n Você escuta batidas na porta, e passos para fora do corredor logo depois.",
                );
            }
            if self.caroline_na_porta() {
                self.resetar_caroline(rng);
                ui.exibir("\n Você escuta um estrondo na porta, e depois passos apressados para a sala de jantar.");
            }
        }
        let rick =
            self.rick_pos >= RICK_ATAQUE || (self.rick_pos == RICK_PORTA && rng.gen::<f64>() < 0.3);
        let caroline = self.caroline_pos >= CAROLINE_ATAQUE
            || (!self.caroline_no_duto
                && self.caroline_pos == CAROLINE_PORTA
                && rng.gen::<f64>() < 0.3);
        let invasao = (rick && !self.porta_fechada)
            || (caroline && (self.caroline_no_duto || !self.porta_fechada))
            || self.jon_pos >= JON_ATAQUE;
        if !invasao {
            return false;
        }
        if god_mode {
            ui.exibir(&format!("\n{DOS_AMARELO}[GOD MODE] Um animatrônico entra na sala... mas você o encara. Ele pede desculpas e sai de fininho.{RESET}"));
            self.rick_pos = 0;
            self.jon_pos = 0;
            self.caroline_pos = 0;
            return false;
        }
        ui.exibir(if captcha {
            "\n Distraído com o override, você não percebe a sombra até ser tarde demais."
        } else {
            "\n Um animatronico conseguiu entrar."
        });
        ui.buffer_efeito("@@JUMPSCARE@@");
        ui.pausa(2.0);
        true
    }

    fn mover_animatronicos(&mut self, furia: i32, ui: &mut TerminalBuffer, rng: &mut impl Rng) {
        if self.rick_pos == RICK_PORTA && !self.porta_fechada && rng.gen::<f64>() < 0.25 {
            self.rick_pos = 1;
            ui.exibir(" Ouve passos pesados a se afastar da porta");
        } else if self.rick_pos < RICK_PORTA {
            self.rick_pos = (self.rick_pos
                + [0, 1, 1, 2][rng.gen_range(0..4)] * (furia + self.turno / 6))
                .min(RICK_PORTA);
        } else if self.rick_pos == RICK_PORTA && rng.gen::<f64>() < 0.80 {
            self.rick_pos += 1;
        }
        let passo_jon = rng.gen_range(0..=2) + i32::from(self.deteccao_quebrada);
        self.jon_pos = (self.jon_pos + passo_jon).min(JON_ATAQUE);
        self.caroline_pos = (self.caroline_pos + rng.gen_range(0..=3)).min(CAROLINE_ATAQUE);
    }

    fn avancar_turno(
        &mut self,
        jogo: &GameState,
        ui: &mut TerminalBuffer,
        rng: &mut impl Rng,
    ) -> bool {
        self.usos_sistema_turno = 0;
        if self.turnos_gerador > 0 {
            self.turnos_gerador -= 1;
            if self.turnos_gerador == 0 {
                ui.exibir(&format!("\n{DOS_AMARELO} O gerador reserva para de soltar fumaça, e começa a dar gargalos, e depois deliga. A energia volta a ser drenada normalmente{RESET}"));
            }
        }
        match self.turno {
            TURNO_INICIO_CALOR => ui.exibir(&format!("\n{DOS_AMARELO} [SISTEMA] O antigo gerador está superaquecendo, cada acão custará mais energia a partir de agora.{RESET}")),
            TURNO_CRITICO => ui.exibir(&format!("\n {DOS_AMARELO} [SISTEMA] [AVISO CRITICO!!!] O gerador superaqueceu! Geradores reservas ligados, dreno de energia aumentou!{RESET}")),
            _ => {}
        }
        if self.porta_fechada && self.energia > 0 {
            self.energia -= 2;
            ui.exibir(" A pesada porta de metal consome energia contínua... (-2% Energia)");
        }
        if self.energia <= 0 && !self.apagao && !jogo.god_mode {
            ui.exibir("\n [ ENERGIA ESGOTADA ] Tudo fica escuro.");
            ui.buffer_efeito("@@PORTA@@");
            ui.exibir("A porta abre sozinha...");
            self.porta_fechada = false;
            self.apagao = true;
            ui.pausa(2.0);
        }
        if self.resolver_invasao(jogo.god_mode, false, ui, rng) {
            return false;
        }
        self.mover_animatronicos(jogo.furia_noite, ui, rng);
        self.indio_ativo =
            self.turno >= TURNO_INICIO_CALOR && (self.turno >= 20 || rng.gen_range(1..=100) > 70);
        if !jogo.alberto_desativado && rng.gen_range(1..=100) > 80 {
            self.alberto_msg_falsa = true;
        }
        ui.exibir("\n[A atualizar sistema...]");
        ui.pausa(1.2);
        true
    }
}

fn evento_ambiente(ui: &mut TerminalBuffer, rng: &mut impl Rng) {
    let (cor, mensagem) = match rng.gen_range(1..=100) {
        1..=3 => (DOS_AMARELO, " Toc.. Toc.. Você escuta batidas fracas na janela, você não sabe se há algo ali, o vidro está muito sujo."),
        4..=7 => (DOS_AMARELO, " Você escuta ruidos vindo da ventilação... Parece que algo está arranhando o aluminio. "),
        8..=9 => (DOS_VERMELHO, " 'Rogerio'... Você escuta algo chamar seu nome vindo do fundo do corredor."),
        10 => (DOS_VERMELHO, " Pelo canto do seu olho, você jura ter visto algo acenando da janela, você não sabe se há algo real ou não."),
        11..=12 => (DOS_VERMELHO, " Você jura ter visto algo na ventilação... Será que é coisa da sua cabeça?"),
        _ => return,
    };
    ui.exibir(&format!("\n{cor}{mensagem}{RESET}"));
}

fn vencer(jogo: &mut GameState, pulou: bool, ui: &mut TerminalBuffer, media: &mut MediaManager) {
    ui.limpar();
    if pulou {
        ui.animar_com_intervalo("Você se sente aliviado quando a luz do sol começa a invadir a janela do restaurante, e o relógio marca pontualmente '06:00' ", DOS_BRANCO, 0.03);
        ui.pausa(2.0);
        ui.animar_com_intervalo(
            "O sol começa a nascer. A energia retorna aos poucos.",
            DOS_BRANCO,
            0.03,
        );
        ui.animar_com_intervalo("A porta da sala destranca.", DOS_BRANCO, 0.03);
        for (nome, descricao) in [
            ("sala de jantar", "A luz da manhã invade as janelas sujas."),
            ("hall de entrada", "O hall está iluminado."),
            ("balcão", "A claridade revela o mofo nos doces."),
            ("entrada", "As luzes não piscam mais."),
        ] {
            if let Some(sala) = jogo.mapa.get_mut(nome) {
                sala.descricao = descricao.to_string();
            }
        }
        if jogo.fios_cortados_inventario {
            ui.pausa(2.0);
            ui.animar_com_intervalo("\nVocê saca o dispositivo.", DOS_AMARELO, 0.03);
            ui.exibir(&format!("{DOS_VERDE}{ARTE_RADAR}{RESET}"));
            ui.pausa(1.0);
            ui.animar_com_intervalo(
                "[DISPOSITIVO]: PRESENÇA ULTERIOR DETECTADA.",
                DOS_VERDE,
                0.03,
            );
            ui.animar_com_intervalo("Ela ainda está aqui...\n", DOS_AMARELO, 0.04);
            ui.pausa(3.0);
        }
    } else {
        ui.animar_com_intervalo("06:00 AM", DOS_VERDE, 0.1);
        ui.animar_com_intervalo(
            "Os sistemas entram em modo de espera diurno. Você sobreviveu.",
            DOS_BRANCO,
            0.05,
        );
        ui.pausa(2.0);
    }
    ui.buffer_efeito("@@NORMAL_POWER@@");
    jogo.noite_vencida = true;
    jogo.estado_atual = "JOGO".to_string();
    views::imprimir_contexto_sala(jogo, ui, media);
}

pub fn processar(
    comando: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) {
    let Some(mut s) = jogo.seguranca_dados.take() else {
        iniciar(jogo, ui);
        return;
    };
    let mut rng = rand::thread_rng();
    if let Some(captcha) = s.captcha_ativo.take() {
        s.responder_captcha(comando, captcha, ui);
        if s.resolver_invasao(jogo.god_mode, true, ui, &mut rng) {
            views::dar_tela_de_morte(jogo, ui, audio, media);
            return;
        }
        s.mover_animatronicos(jogo.furia_noite, ui, &mut rng);
        imprimir_status(&s, jogo.god_mode, ui, &mut rng);
        jogo.seguranca_dados = Some(s);
        return;
    }
    if jogo.god_mode && matches!(comando, "pular noite" | "pular" | "set time 06:00") {
        ui.exibir(&format!("\n{DOS_AMARELO}[GOD MODE ATIVADO]{RESET}"));
        ui.exibir(&format!(
            "{DOS_AMARELO}As leis da física se contorcem. O tempo acelera artificialmente.{RESET}"
        ));
        ui.exibir(&format!("{DOS_AMARELO}Os ponteiros do relógio digital giram freneticamente até parar às 06:00.{RESET}"));
        vencer(jogo, true, ui, media);
        return;
    }
    let mut turno_passou = false;
    let mut valido = true;
    match comando {
        "consertar camera" | "consertar deteccao" | "consertar relogio" => {
            s.iniciar_conserto(comando.trim_start_matches("consertar "), ui, &mut rng);
            imprimir_status(&s, jogo.god_mode, ui, &mut rng);
            jogo.seguranca_dados = Some(s);
            return;
        }
        "fechar porta" => s.acionar_porta(true, ui),
        "abrir porta" => s.acionar_porta(false, ui),
        "ouvir" | "cameras" | "ver tubulacao" | "iluminar tubulacao" => {
            s.consultar(comando, ui, &mut rng)
        }
        "olhar vidro" => s.olhar_vidro(ui, &mut rng),
        "ligar gerador" => turno_passou = s.ligar_gerador(ui),
        "esperar" | "aguardar" => {
            ui.exibir(&format!("{DOS_BRANCO}Você prende a respiração, recua da mesa e deixa os minutos se arrastarem no escuro...{RESET}"));
            turno_passou = true;
        }
        _ => {
            ui.exibir(&format!("{DOS_VERMELHO}Erro de Sintaxe: Comando inválido ou periférico não reconhecido pelo terminal.{RESET}"));
            valido = false;
        }
    }
    if turno_passou {
        s.turno += 1;
        s.alberto_msg_falsa = false;
    }
    if valido && !matches!(comando, "esperar" | "aguardar") {
        s.interferencia(ui, &mut rng);
        if s.porta_fechada && s.rick_pos >= RICK_ATAQUE {
            s.rick_pos = 0;
            ui.exibir(&format!(
                "\n{DOS_AMARELO} ALGO SOCA A PORTA COM VIOLÊNCIA E RECUA{RESET}"
            ));
        }
        if s.porta_fechada && !s.caroline_no_duto && s.caroline_pos >= CAROLINE_ATAQUE {
            s.resetar_caroline(&mut rng);
            ui.exibir(&format!(
                "\n{DOS_AMARELO} Um estrondo na porta. Ela recuou...{RESET}"
            ));
        }
    }
    ui.pausa(1.5);
    if valido {
        evento_ambiente(ui, &mut rng);
    }
    ui.pausa(2.0);
    if turno_passou && !s.avancar_turno(jogo, ui, &mut rng) {
        views::dar_tela_de_morte(jogo, ui, audio, media);
        return;
    }
    if !turno_passou && s.turno >= TURNO_FINAL {
        vencer(jogo, false, ui, media);
        return;
    }
    imprimir_status(&s, jogo.god_mode, ui, &mut rng);
    jogo.seguranca_dados = Some(s);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn reserva_avanca_um_turno_e_nao_pode_ser_reutilizada() {
        let mut jogo = GameState::new();
        let mut s = SegurancaData {
            energia: 100,
            ..SegurancaData::default()
        };
        jogo.god_mode = true;
        let mut ui = TerminalBuffer::default();
        let mut rng = StdRng::seed_from_u64(20);
        assert!(s.ligar_gerador(&mut ui));
        assert_eq!(s.turnos_gerador, 2);
        s.turno += 1;
        assert!(s.avancar_turno(&jogo, &mut ui, &mut rng));
        assert_eq!(s.turnos_gerador, 1);
        assert_eq!(s.energia, 100);
        assert_eq!(s.iniciar_consulta("ouvir", &mut ui), Some(0));
        assert!(!s.ligar_gerador(&mut ui));
    }

    #[test]
    fn fechar_porta_gasta_energia_sem_avancar_e_apagao_abre_porta() {
        let jogo = GameState::new();
        let mut s = SegurancaData {
            energia: 4,
            ..SegurancaData::default()
        };
        let mut ui = TerminalBuffer::default();
        let mut rng = StdRng::seed_from_u64(40);
        s.acionar_porta(true, &mut ui);
        assert_eq!(s.turno, 0);
        assert_eq!(s.energia, 2);
        assert!(s.porta_fechada);
        s.turno += 1;
        assert!(s.avancar_turno(&jogo, &mut ui, &mut rng));
        assert!(s.apagao);
        assert!(!s.porta_fechada);
    }
}
