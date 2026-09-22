import pytest
from server.app import app, MEMORIA_SESSOES

@pytest.fixture
def cliente():
    
    
    app.config['TESTING'] = True
    
    with app.test_client() as client:
       
        yield client
        
        MEMORIA_SESSOES.clear()