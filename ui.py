import os
import re
import sys
import time

import colorama

colorama.init()
if sys.platform == "win32":
    os.system("color")

DEBUG_MODE = False

DOS_VERDE = "\033[92m"
DOS_BRANCO = "\033[97m"
DOS_AMARELO = "\033[93m"
DOS_VERMELHO = "\033[91m"
RESET = "\033[0m"


def limpar_tela():
    print("\n" * 50)


def pausar(segundos):
    if not DEBUG_MODE:
        time.sleep(segundos)


def digitar(texto, tempo_base=0.03, cor="", jogo=None):
    texto_final = f"{cor}{texto}{RESET}" if cor else texto
    if DEBUG_MODE:
        print(texto_final)
        return
    tempo_real = tempo_base
    if jogo is not None:
        if getattr(jogo, "hp", 3) <= 1:
            tempo_real = 0.005
        elif getattr(jogo, "turnos_no_escuro", 0) >= 3:
            tempo_real = 0.01
    partes = re.split(r"(\033\[[0-9;]*m)", texto_final)
    for parte in partes:
        if parte.startswith("\033["):
            sys.stdout.write(parte)
            sys.stdout.flush()
        else:
            for char in parte:
                sys.stdout.write(char)
                sys.stdout.flush()
                time.sleep(tempo_real)
    print()


class UIHandler:
    """Classe base para abstrair as saídas de texto."""

    def limpar(self):
        limpar_tela()

    def pausar(self, segs):
        pausar(segs)

    def exibir(self, texto):
        print(texto)

    def animar(self, texto, tempo=0.03, cor="", jogo=None):
        digitar(texto, tempo, cor, jogo)

    def obter_input(self, prompt_text):
        return input(prompt_text)


default_ui = UIHandler()
