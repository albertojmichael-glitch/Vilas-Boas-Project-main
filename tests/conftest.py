import pytest
from app import app, MEMORIA_SESSOES

@pytest.fixture
def cliente():
    """
    Cria um cliente de teste do Flask. 
    Isso nos permite fazer requisições GET/POST simulando um navegador real,
    mantendo os cookies de sessão entre as chamadas!
    """
    
    app.config['TESTING'] = True
    
    with app.test_client() as client:
       
        yield client
        
        MEMORIA_SESSOES.clear()