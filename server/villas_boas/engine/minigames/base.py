class BaseMinigame:
    """interface universal para todos os minigames."""
    
    def __init__(self, jogo):
        self.jogo = jogo
        self.ui = jogo.ui_handler
        
        self.turno_atual = 0 
        self.estado_interno = {}

    def processar_turno(self, comando: str, jogo) -> str:
        
        raise NotImplementedError("Este minigame esqueceu de implementar processar_turno()")

    def imprimir_status(self):
        """imprime a tela do minigame na interface."""
        raise NotImplementedError("Este minigame esqueceu de implementar imprimir_status()")