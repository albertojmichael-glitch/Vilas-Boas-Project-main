class BaseMinigame:
    """Interface universal para todos os minigames do Vilas Boas."""
    
    def __init__(self, jogo):
        self.jogo = jogo
        self.ui = jogo.ui_handler
        
        self.turno_atual = 0 
        self.estado_interno = {}

    def processar_turno(self, comando: str, jogo) -> str:
        """
        Processa o input do jogador. 
        DEVE retornar obrigatoriamente uma destas strings:
        - "continuar"
        - "morte"
        - "vitoria_NOME_DO_MINIGAME"
        """
        raise NotImplementedError("Este minigame esqueceu de implementar processar_turno()")

    def imprimir_status(self):
        """Imprime a tela do minigame na interface."""
        raise NotImplementedError("Este minigame esqueceu de implementar imprimir_status()")