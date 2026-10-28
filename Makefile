.PHONY: setup test lint format run cli push

setup:
	python3 -m venv venv
	./venv/bin/pip install -U pip
	./venv/bin/pip install -e ".[dev]"

test:
	PYTHONPATH=server ./venv/bin/pytest tests/ --cov=server --cov-report=term-missing -v

lint:
	./venv/bin/ruff check .
	./venv/bin/ruff format --check .
	PYTHONPATH=server ./venv/bin/mypy server/

format:
	./venv/bin/ruff check --fix .
	./venv/bin/ruff format .

run:
	PYTHONPATH=server ./venv/bin/python server/app.py

cli:
	PYTHONPATH=server ./venv/bin/python server/cli.py

push: format lint test
	git add .
	git commit -m "chore: auto-format, lint and pass tests"
	git push
