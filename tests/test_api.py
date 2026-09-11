def test_ping(cliente):
    """Verifica se o servidor Flask está online e respondendo."""
    resposta = cliente.get('/ping')
    assert resposta.status_code == 200
    assert b"Estou vivo!" in resposta.data

def test_iniciar_jogo_cria_sessao(cliente):
    """Testa se a rota /iniciar gera um UUID seguro e retorna o estado inicial."""
    resposta = cliente.get('/iniciar')
    
    
    assert resposta.status_code == 200
    dados = resposta.get_json()
    
    
    assert "estado" in dados
    assert "linhas" in dados
    assert dados["estado"]["sala"] == "SISTEMA"

def test_fluxo_comando_basico(cliente):
    """
    Simula um jogador acessando o site e digitando o comando 'dir'
    para sair da tela de boot e ir para o Menu.
    """
    
    cliente.get('/iniciar')
    
    
    payload = {
        "comando": "dir",
        "telemetria": False
    }
    resposta = cliente.post('/comando', json=payload)
    
    assert resposta.status_code == 200
    dados = resposta.get_json()
    
    
    texto_terminal = "".join(dados["linhas"])
    assert "Directory of A:\\" in texto_terminal
    assert "COMMAND  COM" in texto_terminal

def test_comando_invalido_seguranca(cliente):
    """Garante que a API bloqueie payloads maliciosos ou muito longos."""
    cliente.get('/iniciar')
    
    payload_malicioso = {
        "comando": "a" * 300, 
        "telemetria": False
    }
    resposta = cliente.post('/comando', json=payload_malicioso)
    
    
    assert resposta.status_code == 400
    dados = resposta.get_json()
    
   
    assert "[ ERRO DE SEGURANÇA ]" in dados["linhas"][0]