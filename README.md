surreal-migraine
=================

A tiny CLI to generate SurrealDB migration files (numeric or temporal prefixed).

Install

```powershell
# install from crates.io
cargo install surreal-migraine

# or build locally
cargo build --release
```

Usage

```powershell
# numeric (auto-increment)
surreal-migraine add "Create users"

# temporal (timestamped)
surreal-migraine add --temporal "Add audit table"

# specify directory and increase verbosity
surreal-migraine add --dir ./migrations -v "Fix schema"
```

CLI quick reference

- `add <NAME>` — create a migration file using NAME (sanitized).
- `--temporal` / `-t` — use timestamp prefix instead of numeric.
- `--dir <DIR>` — override migrations directory (defaults to ./migrations).
- `-v, -vv` — increase logging verbosity (debug/trace).

Notes

- Names are sanitized (whitespace -> underscores, invalid chars removed).
- Numeric mode picks the next numeric prefix (e.g. `000_...`, `001_...`).
- Temporal mode uses a timestamp `YYYYMMDDHHMMSS` and will append a suffix if a collision occurs.