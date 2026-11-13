# Show all commands
default:
    @just --list

# Install dependencies
setup:
    uv sync

# Run the tests
test:
    uv run pytest

# Check the code style
lint:
    uv run ruff check .

# Lint and test together
check: lint test

# 
# Check the folder layout and names
layout:
    ls-lint

# Show what organize would move or rename (changes nothing)
tidy-sim:
    organize sim .organize.yaml

# Move and rename files according to organize.yaml
tidy:
    organize run .organize.yaml
