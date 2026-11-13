# Folder structure

This is the only allowed layout. Do not create other folders.

- `specs/` holds `spec.md`, `design.md` and `tickets.md` for the whole app.
- Each module gets its own folder `specs/<module-name>/` with the same three files.
- `docs/` holds notes for people.
- `src/<package_name>/` holds all application Python code.
- `tests/` holds all tests.
- `scripts/` holds helper scripts.
- `sql/` holds SQL scripts.
- `data/` holds data and database files. These are not version controlled.
- `.agents/prompts/` holds the rules for agents. Do not edit them unless asked.
- Root only holds config files and `README.md`, `AGENTS.md`, `CLAUDE.md`.

Run `ls-lint` before you finish. If it fails, fix the place of the file.
