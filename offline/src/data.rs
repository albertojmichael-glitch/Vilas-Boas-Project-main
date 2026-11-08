use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const SERVER_BUNDLE_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/server_bundle.json"));
pub const EMBEDDED_GAME_DATA: &str = include_str!("../../server/data/game_data.json");
pub const EMBEDDED_BALANCE_CFG: &str = include_str!("../../server/data/balance_config.json");

// Constantes de Inventário e Status (1:1 com server/data.py + balance_config.json)
pub const MAX_INVENTARIO: usize = 3;
pub const SLOTS_POR_BOLSA: usize = 3;
pub const META_ACUMULADOR: usize = 9;

pub const VIDA_NORMAL: i32 = 3;
pub const VIDA_PESADELO: i32 = 2;
pub const VIDA_GOD_MODE: i32 = 9999;
pub const TURNOS_BATERIA: i32 = 12;
pub const TURNOS_LUZ_INICIAL: i32 = 3;
pub const ISQUEIRO_TURNOS_BONUS: i32 = 2;
pub const ISQUEIRO_LIMITE_LUZ: i32 = 2;
pub const ISQUEIRO_BARULHO: i32 = 10;

// Constantes de Barulho e IA
pub const REDUCAO_BARULHO_TURNO: i32 = 5;
pub const LIMIAR_CACA: i32 = 80;
pub const LIMIAR_ALERTA: i32 = 50;
pub const CHANCE_MOVER_PATRULHA: f64 = 0.5;
pub const BARULHO_CORRER: i32 = 100;
pub const TURNOS_ENJOADO_CORRER: i32 = 2;
pub const CHANCE_PERDER_ITEM_CORRER: f64 = 0.25;
pub const BARULHO_IR_ABRIR: i32 = 15;
pub const REDUCAO_OLHAR_ESPERAR: i32 = 10;
pub const BARULHO_ACAO_COMUM: i32 = 5;
pub const BARULHO_ERROS_CONSECUTIVOS: i32 = 30;
pub const LIMITE_ERROS_DICA: i32 = 3;
pub const LIMITE_BARULHO_FANTASMA: i32 = 50;

// Constantes de Exploração
pub const CHANCE_TROPECO_ESCURO_PCT: i32 = 10;
pub const CHANCE_EVENTO_ALEATORIO: f64 = 0.25;
pub const SENHA_COFRE: &str = "1994";
pub const COFRE_SENHA: &str = "1994";
pub const MAX_TENTATIVAS_COFRE: i32 = 3;
pub const TEMPO_SPEEDRUN_VERDADEIRO: f64 = 360.0;
pub const TEMPO_SPEEDRUN_NEUTRO: f64 = 300.0;

// Artes ASCII 1:1 com server/data.py e server/villas_boas/engine/core.py
pub const CAVEIRA_MORTE: &str = r#"
                     .ed"""" """$$$$be.
                   -"           ^""**$$$e.
                 ."                   '$$$c
                /                      "4$$b
               d  3                     $$$$
               $  * .$$$$$$
              .$  ^c           $$$$$e$$$$$$$$.
              d$L  4.         4$$$$$$$$$$$$$$b
              $$$$b ^ceeeee.  4$$ECL.F*$$$$$$$
  e$""=.      $$$$P d$$$$F $ $$$$$$$$$- $$$$$$
 z$$b. ^c     3$$$F "$$$$b   $"$$$$$$$  $$$$*"      .=""$c
4$$$$L   \     $$P"  "$$b   .$ $$$$$...e$$        .=  e$$$.
^*$$$$$c  %..   *c    ..    $$ 3$$$$$$$$$$eF     zP  d$$$$$
  "**$$$ec   "\   %ce""    $$$  $$$$$$$$$$* .r" =$$$$P""
        "*$b.  "c  *$e.    *** d$$$$$"L$$    .d"  e$$***"
          ^*$$c ^$c $$$      4J$$$$$% $$$ .e*".eeP"
             "$$$$$$"'$=e....$*$$**$cz$$" "..d$*"
               "*$$$  *=%4.$ L L$ P3$$$F $$$P"
                  "$   "%*ebJLzb$e$$$$$b $P"
                    %..      4$$$$$$$$$$ "
                     $$$e   z$$$$$$$$$$%
                      "*$c  "$$$$$$$P"
                       ."""*$$$$$$$$bc
                    .-"    .$***$$$"""*e.
                 .-"    .e$"     "*$c  ^*b.
          .=*""""    .e$*"          "*bc  "*$e..
        .$"        .z*"               ^*$e.   "*****e.
        $$ee$c   .d"                     "*$.        3.
        ^*$E")$..$"                         * .ee==d%
           $.d$$$* * J$$$e*
            """""                             "$$$"    
"#;

pub const ARTE_INDIO: &str = r####"
                               ,,..,
                             ,@$$$$$.
                           .,$$$$$$$$i
                     .,z$""')$$$$$$$$C`^#`-..
                  ,zF'        `""#*"'       "*o.
               ,zXe>        u:..        ..      "c
             ,' zP'    ,:`"          .            "N.
           ,d",d$   ,'"   ,uB" .,uee..,?R.  ,  .    ^$.
         ,@P d$"     .:$$$$$$$$$$$$$@$CJN.,"    `     #b
        z$" d$P    :SM$$$$$$$$$$$$$$$$$$$Nf.           ^$.
       J$" J$P  , ,@$$$$$$$$$$$$$$$$$$$$$$$$$k.         "$r
      z$   $$.   ,$$$$$$$$$$$$$$$$$$$$$$$$$$f'   .    .   $b
     ,$"  $$u,-.x'^""$$$$$$$$$$$$$$$$$$$$$$$$$.        `.  $k
     $"  :$$$$> 8.   `#$$$$$$$$$$$$$$$$$$$$$"\  d  .    F   $.
    $P  .$$$$$N `$b.  $$$$$$$$$$$$$$$$$$$$$k.$  $"  :   '   `$
   <$'  4$$k $$c `*$.,Q$$$$$$$$$$$$$$$$$$$$$$$ ..            $L
   $P   4$$$$$F:   `"$$$$$$$$$$$$$$$$$$$$$$'`$"     .   ,    `$
  ,$'  ,$$$$$d$$    '##$$c3$$$$$$$$$$$$$$$$. '      :   L.    $.
  J$  u$$$$$$$$$.,oed$*$$$$N "#$$$$$$$$$$***$@$N. , $  ,B$$N.,9L
  $F,$$$$$$$$$$,@*"'  `J$$$$$#h$$$$$$P"`     `"*$$. $4W$' "$$uJF
  4$$$$$$$$$$$$F'      $*'`$$RR@$$$$$R        ,' "$d$4"    '$$$R
 ,$$$$$$$$$$$$$F     ,'    @$.3$$$$ R>            `$F$  dN.4$$$$.
:$$$$$$$$$$$$*$"          J$'$$$$$& $.             $'   $$$$$$$$$o
 ^$$$$$$$$$$B@$$          $P $$$"?N/$k             $r   $$P"*$$$$'
   $$i  .$$$$"$'         $$ ~R$P '$k^$$,'          $   "'  ,d$$'
   $$$$ J$$$$ `,'    .,z$P'd.$P   #$. #$$$u.       .$  eu. ,d$$$
   $^$$$$$$$$. `"=+=N#'.,d$M$$'   `$$@s.#$$$u.   ,$C  $$$@$$$"$
   "  `*$$$$$$bx..        ,M$"     `*$$$b/""$R"*"'d$ ,$$$$P"  '
   4     "$$k3$9$$B.e.  ,ud$F       `3$$$$b.      ,$,@R$*'    4
   <       *$$$$$$$b$$@$$$$$L   ,.  ,J$$.'**$$k$NX$"M"'       .
   $         "$#"  `" <$$$$$$c,z$N.,o$$$$   ,NW$*"'           $
   $.         ',    `$$$$$$$$$d$$$$$$$$$f ,$e*'                $
  ,$c         d.     `^$$$$$$$$$$$$$$$$$.u '"                :$.
  $$$         $\   .,  `"#$$$*$$$$$$$$$$$$ '                 4$F
  $$"         $ `  k.`.     ``"#`"""'      ,' ,'             `$$
  `"          $>,  `b.,ce(b:o uz CCLd$4$*F?\,o                "' 
              $&    $$k'*"$$$$$$#$$$$$$$$$$ d'
              $$.,$$$$$$$$,e,$#$.*$`""""'e4 $
              `$$$$  ^$$\$"$$$$$$$$$$$$$$$.eL
               $$$"  $$$$$$$e$.$.$$.$e$d$$$$k
               R`$$  '$$$$$$$$$$$$$$$$$$$$$$P
               `  $Nc'"$$N3$$$$$$$$$$$$$$$$$'  
                   *$  9$.`@$$$$$$$$$$R$$$#'
                    `$.  `"*$$$$$$$$$$P'' #
                      "$u.    `""""''   ,'
                        `"$Nu..  .,z
"####;

pub const ARTE_PORCO: &str = r#"
      _.._         ,.
    .' .-'`       (_|,.
   /  /      * ,' /, )_______   _
   |  |    * __j o``-'        `.'-)'
   \  \      * (")                 \'
    '._'-._       `-j               |
                      `-._(        /
                        |_\  |--^./
                          /_]' /_)
                            
        A tela apresenta problemas...
"#;

pub const ARTE_ROBO: &str = r#"
       .-T-.
      /     \
    }=) o o (={
      \_===_/
(_)  _.-"""-._
 |\/`/+' _ '+\`\
  \__\ +[_]+ /=|
      )====={\=\_
      |  .  | `( )
      |_/ \_|
     <__| |__>
      |=| |=|
      |_| |_|
     (___Y___)
"#;

pub const ARTE_PIANO: &str = r#"
 ____________________________________
|\                                    \
| \                                    \
|  \____________________________________\
|  |       __---_ _---__                |
|  |      |======|=====|                |
|  |      |======|=====|                |
|  |  ____|__---_|_---_|______________  |
|  | |                                | |
|   \ \                                \ \
|  \ ||\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\| |     
|  |\  ,--------------------------------  |
|  ||| |                               || |
 \ ||| |           -  -                || |
  \'|| |-----------\\-\\---------------|| |
    \|_|            "  "               \|_|
"#;

pub const ARTE_MESA_SEGURANCA: &str = r#"
                              ___________
                              |.---------.|
      ______________________  || o   [  ]||
     |  __________________  | ||---------||
     | |                  | | ||     [  ]||
     | |                  | | |'---------'|
     | |                  | | |  _______  |
     | |                  | | | | [   ] | |
     | |                  | | | |_______| |
     | |                  | | |     .     |
     | |__________________| | |     .     |
    (|______________________| |    / \    |
    )) |______________|_|_|   |    \ /    |
   ((    / \__________/ \     |     o     |
   _))__/________________\_____           |
  / - - - - - - - - - - - /__\ \__________|
 / - - - - - - - - - - - - - -  \
/________________________________\  
'--------------------------------'
"#;

pub const ARTE_PASTA: &str = r#"
      _____
      /     \_____
     |            |
     |   _________|__
     |  /           /
     | /           /
     |/___________/
"#;

pub const ARTE_RADAR: &str = r#"
         .-.
      _.-' '-._
    ,';-"```"-;'.
   /.'    |    '.\
  //   '. | .'   \\
  ||  --- o ---  ||
  \\   .' | '.   //
   \'.    |    .'/
    '.'-._^_.-'.'
      ``-----``
"#;

pub const ARTE_DISQUETE: &str = r#"
.------.
|######|
|######| 
|------| 
| [__] | 
'------'
"#;

pub const ARTE_FINAL_VERDADEIRO: &str = r#"

 .

              |
     .               /
      \       I       
                  /
        \  ,g88R_
          d888(`  ).                   _
 -  --==  888(     ).=--           .+(`  )`.
)         Y8P(       '`.          :(   .    )
        .+(`(      .   )     .--  `.  (    ) )
       ((    (..__.:'-'   .=(   )   ` _`  ) )
`.     `(       ) )       (   .  )     (   )  ._
  )      ` __.:'   )     (   (   ))     `-'.:(`  )
)  )  ( )       --'       `- __.'         :(      ))
.-'  (_.'          .')                    `(    )  ))
                  (_  )                     ` __.:'

--..,___.--,--'`,---..-.--+--.,,-,,..._.--..-._.-a:f--.
"#;

pub const ARTE_ENTRADA: &str = r#"
      _,                _.
     (  `)            (`  ).
  .=( ` ,_ `)    .-``(      ).
 (.__.:-`-_.'   (.,,(.       '`.
                  `--`--`'`

        ____.........___
      __/%%%%|%%%%%%%|%%%%\
 _ ()/%%|:II:|II:::II|:II:|_ _ _
|-(()|--|:II:|II:H:II|:II:|-|-|-|
`'.'"^  ^` "^ "^|"|^'"' `^`-.^~'
"#;

pub const ARTE_FINAL_MEDIOCRE: &str = r#"
                                 * .    .    .     +     .  * .     .  *
.   * . .   * .    .   .        *     .
       __________    * .    *   . sinto sua falta.    +  .   *  .   
      /`.---..----;\    .   * . . *     * +  *  * . . * . 
     / /____||_____\\__       *   .      *    +     .
    | "~      7        `'.
    (_ .--. ========.--. =|
    `"( () )       ( () )"`         
……………….'--'.........'--'.............................................
"#;

pub const ARTE_FINAL_NEUTRO: &str = r#"
 *    .  *       .            *

                         *

 *   .        *       .       .       *

   .     *     .       *             . 

       .     .  *        *
    Nós estamos Juntos Novamente.
   .                .        .

.  *           *                    *

         .               *    .

     *          .   *
"#;

pub const ARTE_ISQUEIRO: &str = r#"
          
            ,-`    \
            \       \
             \       \
               \       \
                \       \
    _.-------. \       \
  (o| o o o |  \    .-`
 __||o_o_o_o|_ad-``
|``````````````|' 
|              | 
|              |    
|              |   
|              |  
|__________1962| 
"#;

pub const ARTE_PELUCIA: &str = r#"
    
     _   ____  _.-"-._
    .' `;'    ':.      '-.
   /  ./     .'`\`-...-'`
  /  /|D_  .O    |
 /  / |=\/`=     |
|_.'  |  |       |
       \  \   _ /_
       /`---'}_()_{
      /`'---' //\\\
     /;      (/\ \)\
    / |         \   \
   /  |         |   |
  /  / .     _ /   /
  \_|   '. .'  '-'|
    \    .-|     /
 _.-'   /  |    /
(      /  / .--;
 '-.__/   |    /
          \__.
"#;

pub const ARTE_COFRE: &str = r#"
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
"#;

pub const ARTE_COFRE_TENSO: &str = r#"
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
"#;

pub const MSGS_PERIGO_ESQUERDA: [&str; 3] = [
    "Passos pesados de ferro raspam no chão à sua ESQUERDA.",
    "Um sopro quente e cheiro de óleo queimado vêm da ESQUERDA.",
    "Engrenagens estalam na escuridão ao seu lado ESQUERDO.",
];

pub const MSGS_PERIGO_DIREITA: [&str; 3] = [
    "O chão treme com o peso de algo enorme à sua DIREITA.",
    "Um chiado mecânico abafado ecoa pela sua DIREITA.",
    "Chifres de metal arranham a grade à sua DIREITA.",
];

pub const MSGS_PERIGO_FRENTE: [&str; 3] = [
    "Dois pontos vermelhos brilham brevemente no breu à sua FRENTE.",
    "Uma respiração metálica e pesada bloqueia o caminho à FRENTE.",
    "O som de cascos de ferro vem diretamente da sua FRENTE.",
];

pub const MSGS_PERIGO_TRAS: [&str; 3] = [
    "Você sente o ar deslocar bem ATRÁS da sua nuca!",
    "Passos apressados ecoam no corredor ATRÁS de você!",
    "Um rangido de mandíbula metálica soa logo ATRÁS de você!",
];

pub const MSGS_PERIGO_ESTATICA: [&str; 3] = [
    "O RADAR CHIA VIOLENTAMENTE! ELE ESTÁ DO SEU LADO!",
    "INTERFERÊNCIA MAGNÉTICA MÁXIMA! A CRIATURA ESTÁ COLADA EM VOCÊ!",
    "SINAL CORROMPIDO! VOCÊ SENTE O CALOR DO MOTOR DELE AO SEU LADO!",
];

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
pub struct EventoAleatorio {
    #[serde(default, alias = "mensagem")]
    pub texto: String,
    #[serde(default, alias = "item_drop")]
    pub item_recompensa: Option<String>,
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

#[derive(Debug, Clone, Serialize, Default, PartialEq)]
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
    #[serde(default)]
    pub conexoes: HashMap<String, Option<String>>,
}

impl<'de> Deserialize<'de> for Sala {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map = serde_json::Map::deserialize(deserializer)?;
        let mut sala = Sala::default();
        let ignorar = chaves_nao_saida();

        for (k, v) in map {
            match k.as_str() {
                "descrição" | "descricao" => {
                    if let Some(s) = v.as_str() {
                        sala.descricao = s.to_string();
                    }
                }
                "itens" => {
                    if let Ok(vec) = serde_json::from_value::<Vec<String>>(v) {
                        sala.itens = vec;
                    }
                }
                "inspecionaveis" => {
                    if let Ok(hm) = serde_json::from_value::<HashMap<String, String>>(v) {
                        sala.inspecionaveis = hm;
                    }
                }
                "energia" => sala.energia = Some(v),
                "cofre_important" => sala.cofre_important = Some(v),
                "cadeira" => sala.cadeira = Some(v),
                "imagem" => {
                    if let Some(s) = v.as_str() {
                        sala.imagem = Some(s.to_string());
                    }
                }
                "conexoes" => {
                    if let Ok(hm) =
                        serde_json::from_value::<HashMap<String, Option<String>>>(v)
                    {
                        sala.conexoes.extend(hm);
                    }
                }
                _ => {
                    if !ignorar.contains(&k.as_str()) {
                        match v {
                            Value::String(s) => {
                                sala.conexoes.insert(k, Some(s));
                            }
                            Value::Null => {
                                sala.conexoes.insert(k, None);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(sala)
    }
}

impl Sala {
    pub fn obter_saida(&self, direcao: &str) -> Option<String> {
        if direcao == "cadeira" {
            if let Some(ref c) = self.cadeira {
                if let Some(s) = c.as_str() {
                    return Some(s.to_string());
                }
            }
        }
        self.conexoes.get(direcao).and_then(|v| v.clone())
    }

    pub fn saidas_listadas(&self) -> Vec<(String, String)> {
        let mut res = Vec::new();
        for (k, v) in &self.conexoes {
            if let Some(dest) = v {
                res.push((k.clone(), dest.clone()));
            }
        }
        res.sort_by(|a, b| a.0.cmp(&b.0));
        res
    }

    pub fn saidas_validas(&self, mapa: &HashMap<String, Sala>) -> Vec<(String, String)> {
        let terminais = salas_terminais();
        let mut res = Vec::new();
        for (k, v) in &self.conexoes {
            if let Some(dest) = v {
                if mapa.contains_key(dest) || terminais.contains(&dest.as_str()) {
                    res.push((k.clone(), dest.clone()));
                }
            }
        }
        res.sort_by(|a, b| a.0.cmp(&b.0));
        res
    }
}

pub fn carregar_ascii_arts() -> HashMap<String, String> {
    let mut m = HashMap::new();
    let padroes = [
        ("CAVEIRA_MORTE", CAVEIRA_MORTE),
        ("ARTE_INDIO", ARTE_INDIO),
        ("ARTE_PORCO", ARTE_PORCO),
        ("ARTE_ROBO", ARTE_ROBO),
        ("ARTE_PIANO", ARTE_PIANO),
        ("ARTE_MESA_SEGURANCA", ARTE_MESA_SEGURANCA),
        ("ARTE_PASTA", ARTE_PASTA),
        ("ARTE_RADAR", ARTE_RADAR),
        ("ARTE_DISQUETE", ARTE_DISQUETE),
        ("ARTE_FINAL_VERDADEIRO", ARTE_FINAL_VERDADEIRO),
        ("ARTE_ENTRADA", ARTE_ENTRADA),
        ("ARTE_FINAL_MEDIOCRE", ARTE_FINAL_MEDIOCRE),
        ("ARTE_FINAL_NEUTRO", ARTE_FINAL_NEUTRO),
        ("ARTE_ISQUEIRO", ARTE_ISQUEIRO),
        ("ARTE_PELUCIA", ARTE_PELUCIA),
        ("ARTE_COFRE", ARTE_COFRE),
        ("ARTE_COFRE_TENSO", ARTE_COFRE_TENSO),
    ];
    for (nome, arte) in padroes {
        m.insert(nome.to_string(), arte.to_string());
    }
    m
}

pub fn sinonimos_padrao() -> HashMap<String, String> {
    let mut m = HashMap::new();
    for (k, v) in [
        ("f", "ir frente"),
        ("a", "ir atrás"),
        ("t", "ir atrás"),
        ("e", "ir esquerda"),
        ("d", "ir direita"),
        ("norte", "frente"),
        ("sul", "atrás"),
        ("oeste", "esquerda"),
        ("leste", "direita"),
    ] {
        m.insert(k.to_string(), v.to_string());
    }
    m
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DadosJogo {
    #[serde(default)]
    pub mapa: HashMap<String, Sala>,
    #[serde(default)]
    pub descricoes_itens: HashMap<String, String>,
    #[serde(default)]
    pub eventos_aleatorios: Vec<EventoAleatorio>,
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
        Self::mesclar_json_str(&mut dados, SERVER_BUNDLE_JSON);
        Self::mesclar_json_str(&mut dados, EMBEDDED_GAME_DATA);

        let pastas_data = [
            PathBuf::from("data"),
            PathBuf::from("../data"),
            PathBuf::from("../server/data"),
            PathBuf::from("../server/villas_boas/data"),
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

    fn mesclar_json_str(dados: &mut DadosJogo, raw_json: &str) {
        let Ok(val) = serde_json::from_str::<Value>(raw_json) else {
            return;
        };

        for map_key in ["mapa_original", "mapa"] {
            if let Some(mapa_obj) = val.get(map_key) {
                if let Ok(m) = serde_json::from_value::<HashMap<String, Sala>>(mapa_obj.clone()) {
                    for (k, v) in m {
                        dados.mapa.insert(k, v);
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

        if let Some(ev_val) = val.get("eventos_aleatorios") {
            if let Ok(evs) = serde_json::from_value::<Vec<EventoAleatorio>>(ev_val.clone()) {
                if !evs.is_empty() {
                    dados.eventos_aleatorios = evs;
                }
            } else if let Some(arr) = ev_val.as_array() {
                let mut evs = Vec::new();
                for item in arr {
                    if let Some(tuple) = item.as_array() {
                        if !tuple.is_empty() {
                            let texto = tuple[0].as_str().unwrap_or_default().to_string();
                            let rec = tuple
                                .get(1)
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            evs.push(EventoAleatorio {
                                texto,
                                item_recompensa: rec,
                            });
                        }
                    }
                }
                if !evs.is_empty() {
                    dados.eventos_aleatorios = evs;
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
        for (nome, arte) in carregar_ascii_arts() {
            self.ascii_arts.entry(nome).or_insert(arte);
        }
        if self.sinonimos.is_empty() {
            self.sinonimos = sinonimos_padrao();
        }
    }
}

pub fn obter_audio_embutido(nome_arquivo: &str) -> Option<&'static [u8]> {
    match nome_arquivo.to_lowercase().as_str() {
        "ambiente.mp3" => Some(include_bytes!("../../client/assets/audio/ambiente.mp3")),
        "chiado.mp3" => Some(include_bytes!(
            "../../client/assets/audio/sonsrandomicos.mp3"
        )),
        "grito.mp3" => Some(include_bytes!("../../client/assets/audio/grito.mp3")),
        "metalrangendo.mp3" => Some(include_bytes!(
            "../../client/assets/audio/metalrangendo.mp3"
        )),
        "portafechandoabrindo.mp3" => Some(include_bytes!(
            "../../client/assets/audio/portafechandoabrindo.mp3"
        )),
        "sonsrandomicos.mp3" => Some(include_bytes!(
            "../../client/assets/audio/sonsrandomicos.mp3"
        )),
        "sonsrandomicos2.mp3" => Some(include_bytes!(
            "../../client/assets/audio/sonsrandomicos2.mp3"
        )),
        _ => None,
    }
}

pub const IMAGENS_EMBUTIDAS: &[(&str, &[u8])] = &[
    ("tv.png", include_bytes!("../../client/assets/images/tv.png")),
    (
        "estatica.png",
        include_bytes!("../../client/assets/images/estatica.png"),
    ),
    (
        "tvcolorida.png",
        include_bytes!("../../client/assets/images/tvcolorida.png"),
    ),
    (
        "image.png",
        include_bytes!("../../client/assets/images/image.png"),
    ),
    (
        "JUMPSCARE1.jpg",
        include_bytes!("../../client/assets/static/images/JUMPSCARE1.jpg"),
    ),
    (
        "JUMPSCARE2.jpg",
        include_bytes!("../../client/assets/static/images/JUMPSCARE2.jpg"),
    ),
    (
        "JAPONESMEDO.jpg",
        include_bytes!("../../client/assets/static/images/JAPONESMEDO.jpg"),
    ),
    (
        "SHEISHERE1.jpg",
        include_bytes!("../../client/assets/static/images/SHEISHERE1.jpg"),
    ),
    (
        "SHEISHERE2.jpg",
        include_bytes!("../../client/assets/static/images/SHEISHERE2.jpg"),
    ),
];

pub const IMAGENS_SECRETAS_EMBUTIDAS: &[(&str, &[u8])] = &[
    (
        "HiddenImage1.webp",
        include_bytes!("../../client/assets/secret images/HiddenImage1.webp"),
    ),
    (
        "HiddenImage2.webp",
        include_bytes!("../../client/assets/secret images/HiddenImage2.webp"),
    ),
    (
        "HiddenImage3.webp",
        include_bytes!("../../client/assets/secret images/HiddenImage3.webp"),
    ),
    (
        "HiddenImage4.webp",
        include_bytes!("../../client/assets/secret images/HiddenImage4.webp"),
    ),
    (
        "HiddenImage5.webp",
        include_bytes!("../../client/assets/secret images/HiddenImage5.webp"),
    ),
    (
        "HiddenImage6.webp",
        include_bytes!("../../client/assets/secret images/HiddenImage6.webp"),
    ),
    (
        "HiddenImage7.webp",
        include_bytes!("../../client/assets/secret images/HiddenImage7.webp"),
    ),
    (
        "HiddenImage8.webp",
        include_bytes!("../../client/assets/secret images/HiddenImage8.webp"),
    ),
    (
        "HiddenImage10.webp",
        include_bytes!("../../client/assets/secret images/HiddenImage10.webp"),
    ),
    (
        "HiddenImage11.webp",
        include_bytes!("../../client/assets/secret images/HiddenImage11.webp"),
    ),
    (
        "HiddenImage12.webp",
        include_bytes!("../../client/assets/secret images/HiddenImage12.webp"),
    ),
    (
        "HiddenImage14.webp",
        include_bytes!("../../client/assets/secret images/HiddenImage14.webp"),
    ),
    (
        "HiddenImage15.webp",
        include_bytes!("../../client/assets/secret images/HiddenImage15.webp"),
    ),
    (
        "HiddenImage16.webp",
        include_bytes!("../../client/assets/secret images/HiddenImage16.webp"),
    ),
    (
        "HiddenImage17.webp",
        include_bytes!("../../client/assets/secret images/HiddenImage17.webp"),
    ),
    (
        "HiddenImage18.webp",
        include_bytes!("../../client/assets/secret images/HiddenImage18.webp"),
    ),
    (
        "HiddenImage20.webp",
        include_bytes!("../../client/assets/secret images/HiddenImage20.webp"),
    ),
    (
        "hidden image 19.png",
        include_bytes!("../../client/assets/secret images/hidden image 19.png"),
    ),
    (
        "hidden image 20.png",
        include_bytes!("../../client/assets/secret images/hidden image 20.png"),
    ),
    (
        "hidden image 21.png",
        include_bytes!("../../client/assets/secret images/hidden image 21.png"),
    ),
    (
        "mimic.webp",
        include_bytes!("../../client/assets/secret images/mimic.webp"),
    ),
];

pub fn obter_imagem_embutida(nome_arquivo: &str) -> Option<&'static [u8]> {
    let lower = nome_arquivo.to_lowercase();
    for (nome, bytes) in IMAGENS_EMBUTIDAS {
        if nome.to_lowercase() == lower {
            return Some(*bytes);
        }
    }
    for (nome, bytes) in IMAGENS_SECRETAS_EMBUTIDAS {
        if nome.to_lowercase() == lower {
            return Some(*bytes);
        }
    }
    if let Some(num_str) = lower.strip_suffix(".jpeg").or_else(|| lower.strip_suffix(".jpg")) {
        if let Ok(idx) = num_str.parse::<usize>() {
            if idx >= 1 && idx <= IMAGENS_SECRETAS_EMBUTIDAS.len() {
                return Some(IMAGENS_SECRETAS_EMBUTIDAS[idx - 1].1);
            }
        }
    }
    None
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
