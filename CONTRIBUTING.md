#### `CONTRIBUTING.md`
```markdown
# Contribuindo com o Villas-Boas 1982

Ficamos felizes com o seu interesse em contribuir! Siga as regras abaixo para enviar *Pull Requests*.

## Fluxo de Desenvolvimento
1. Crie um *fork* do repositório.
2. Crie sua branch de *feature* (`git checkout -b feature/minha-nova-sala`).
3. Instale as dependências de desenvolvimento: `pip install -r requirements-dev.txt`.
4. Faça suas mudanças no código e no `data.py`.
5. Valide seu código rodando o Makefile:
   - `make format` (Formata com o Black)
   - `make lint` (Checa tipagem com o Mypy)
   - `make test` (Roda os testes Unitários)
6. Abra o Pull Request descrevendo suas mudanças detalhadamente.