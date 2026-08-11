import json
import os
# Importa o seu data.py atual
import data 

# Garante que a pasta data/ existe
os.makedirs("data", exist_ok=True)

# Monta a estrutura do novo JSON juntando suas variáveis antigas com as novas
dados_json = {
    "constantes": {
        "max_inventario": data.MAX_INVENTARIO,
        "cofre_senha": data.COFRE_SENHA,
        "vida_normal": 3,
        "vida_pesadelo": 2,
        "turnos_bateria": 12,
        "energia_min_noite": 100,
        "energia_max_noite": 100
    },
    "mapa_original": data.MAPA_ORIGINAL,
    "descricoes_itens": data.descricoes_itens
}

# Salva o arquivo JSON perfeitamente formatado
with open("data/game_data.json", "w", encoding="utf-8") as f:
    json.dump(dados_json, f, ensure_ascii=False, indent=4)

print("✅ SUCESSO! O arquivo data/game_data.json foi criado com todo o seu mapa!")