import json
import os
from pathlib import Path

COFRE_SENHA = "1994"
MAX_INVENTARIO = 3

CAVEIRA_MORTE = r'''
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
'''

ARTE_INDIO = r'''
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
'''

ARTE_PORCO = r"""
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
"""

ARTE_ROBO = r'''
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
'''

ARTE_PIANO = r"""
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
"""

ARTE_MESA_SEGURANCA = r"""
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
"""

ARTE_PASTA = r"""
      _____
      /     \_____
     |            |
     |   _________|__
     |  /           /
     | /           /
     |/___________/
"""

ARTE_RADAR = r"""
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
"""

ARTE_DISQUETE = r"""
.------.
|######|
|######| 
|------| 
| [__] | 
'------'
"""

ARTE_FINAL_VERDADEIRO = r"""

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
"""

ARTE_ENTRADA = r"""
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
"""

ARTE_FINAL_MEDIOCRE = r"""
                                 * .    .    .     +     .  * .     .  *
.   * . .   * .    .   .        *     .
       __________    * .    *   . sinto sua falta.    +  .   *  .   
      /`.---..----;\    .   * . . *     * +  *  * . . * . 
     / /____||_____\\__       *   .      *    +     .
    | "~      7        `'.
    (_ .--. ========.--. =|
    `"( () )       ( () )"`         
……………….'--'.........'--'.............................................
"""

ARTE_FINAL_NEUTRO = r"""
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
"""
ARTE_ISQUEIRO = r"""
          
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
"""
ARTE_PELUCIA = r"""
    
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
"""


BASE_DIR = Path(__file__).parent
JSON_PATH = BASE_DIR / "data" / "game_data.json"
BALANCE_PATH = BASE_DIR / "data" / "balance_config.json"

try:
    with open(JSON_PATH, "r", encoding="utf-8") as f:
        _dados = json.load(f)
except FileNotFoundError:
    print(f"ERRO FATAL: Arquivo de dados não encontrado em {JSON_PATH}")
    _dados = {"mapa_original": {}, "descricoes_itens": {}}
except json.JSONDecodeError as e:
    print(f"ERRO FATAL: JSON malformado em game_data.json: {e}")
    _dados = {"mapa_original": {}, "descricoes_itens": {}}

try:
    with open(BALANCE_PATH, "r", encoding="utf-8") as f:
        BALANCE_CFG = json.load(f)
except (FileNotFoundError, json.JSONDecodeError):
    BALANCE_CFG = {}

MAPA_ORIGINAL = _dados.get("mapa_original", {})
descricoes_itens = _dados.get("descricoes_itens", {})
EVENTOS_ALEATORIOS = _dados.get("eventos_aleatorios", [])

_constantes_legadas = _dados.get("constantes", {})
_inv_cfg = BALANCE_CFG.get("inventario", {})
_status_cfg = BALANCE_CFG.get("status_jogador", {})
_ia_cfg = BALANCE_CFG.get("barulho_e_ia", {})
_exp_cfg = BALANCE_CFG.get("exploracao", {})

# Constantes de Inventário e Status
MAX_INVENTARIO = int(
    os.environ.get(
        "MAX_INVENTARIO",
        _inv_cfg.get("max_inicial", _constantes_legadas.get("max_inventario", 3)),
    )
)
SLOTS_POR_BOLSA = int(_inv_cfg.get("slots_por_bolsa", 3))
META_ACUMULADOR = int(_inv_cfg.get("meta_conquista_acumulador", 9))

VIDA_NORMAL = int(
    os.environ.get(
        "VIDA_NORMAL",
        _status_cfg.get("vida_normal", _constantes_legadas.get("vida_normal", 3)),
    )
)
VIDA_PESADELO = int(
    os.environ.get(
        "VIDA_PESADELO",
        _status_cfg.get("vida_pesadelo", _constantes_legadas.get("vida_pesadelo", 2)),
    )
)
VIDA_GOD_MODE = int(_status_cfg.get("vida_god_mode", 9999))
TURNOS_BATERIA = int(
    os.environ.get(
        "TURNOS_BATERIA",
        _status_cfg.get(
            "turnos_bateria", _constantes_legadas.get("turnos_bateria", 12)
        ),
    )
)
TURNOS_LUZ_INICIAL = int(_status_cfg.get("turnos_luz_inicial", 3))
ISQUEIRO_TURNOS_BONUS = int(_status_cfg.get("isqueiro_turnos_bonus", 2))
ISQUEIRO_LIMITE_LUZ = int(_status_cfg.get("isqueiro_limite_luz_uso", 2))
ISQUEIRO_BARULHO = int(_status_cfg.get("isqueiro_barulho_extra", 10))

# Constantes de Barulho e IA
REDUCAO_BARULHO_TURNO = int(_ia_cfg.get("reducao_passiva_turno", 5))
LIMIAR_CACA = int(_ia_cfg.get("limiar_caca", 80))
LIMIAR_ALERTA = int(_ia_cfg.get("limiar_alerta", 50))
CHANCE_MOVER_PATRULHA = float(_ia_cfg.get("chance_mover_patrulha", 0.5))
BARULHO_CORRER = int(_ia_cfg.get("barulho_correr", 100))
TURNOS_ENJOADO_CORRER = int(_ia_cfg.get("turnos_enjoado_correr", 2))
CHANCE_PERDER_ITEM_CORRER = float(_ia_cfg.get("chance_perder_item_correr", 0.25))
BARULHO_IR_ABRIR = int(_ia_cfg.get("barulho_ir_abrir", 15))
REDUCAO_OLHAR_ESPERAR = int(_ia_cfg.get("reducao_olhar_esperar", 10))
BARULHO_ACAO_COMUM = int(_ia_cfg.get("barulho_acao_comum", 5))
BARULHO_ERROS_CONSECUTIVOS = int(_ia_cfg.get("barulho_erros_consecutivos", 30))
LIMITE_ERROS_DICA = int(_ia_cfg.get("limite_erros_dica", 3))
LIMITE_BARULHO_FANTASMA = int(_ia_cfg.get("limite_barulho_desafio_fantasma", 50))

# Constantes de Exploração
CHANCE_TROPECO_ESCURO_PCT = int(_exp_cfg.get("chance_tropeco_escuro_pct", 10))
CHANCE_EVENTO_ALEATORIO = float(_exp_cfg.get("chance_evento_aleatorio", 0.25))
COFRE_SENHA = str(
    os.environ.get(
        "COFRE_SENHA",
        _exp_cfg.get("cofre_senha", _constantes_legadas.get("cofre_senha", "1994")),
    )
)
MAX_TENTATIVAS_COFRE = int(_exp_cfg.get("max_tentativas_cofre", 3))
TEMPO_SPEEDRUN_VERDADEIRO = float(_exp_cfg.get("tempo_speedrun_verdadeiro_seg", 360.0))
TEMPO_SPEEDRUN_NEUTRO = float(_exp_cfg.get("tempo_speedrun_neutro_seg", 300.0))
