# Usa uma imagem oficial do Python leve
FROM python:3.11-slim

# Impede o Python de gerar arquivos .pyc na produção
ENV PYTHONDONTWRITEBYTECODE=1
# Garante que os logs (como os do logger no app.py) apareçam instantaneamente no console
ENV PYTHONUNBUFFERED=1

# Define o diretório de trabalho dentro do contêiner
WORKDIR /app

# Copia o arquivo de dependências e instala
COPY requirements.txt .
RUN pip install --upgrade pip && \
    pip install --no-cache-dir -r requirements.txt

# Copia o restante dos arquivos do projeto
COPY . .

# Expõe a porta 5000 que o Gunicorn vai utilizar
EXPOSE 5000

# Inicia o servidor Gunicorn com 4 workers para lidar com múltiplas conexões
# O formato 'app:app' diz para buscar a variável 'app' dentro do arquivo 'app.py'
CMD ["gunicorn", "--workers", "4", "--bind", "0.0.0.0:5000", "app:app"]