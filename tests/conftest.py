import pytest
import sys
import os


sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'server')))

from app import MEMORIA_SESSOES, app


@pytest.fixture
def cliente():
    
    
    app.config['TESTING'] = True
    
    with app.test_client() as client:
       
        yield client
        
        MEMORIA_SESSOES.clear()