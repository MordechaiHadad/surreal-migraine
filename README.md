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
# numeric paired (default) — creates a folder with up/down
# either binary name can be used: `smg` or `surrealdb-migraine`
smg add "Create users"

# temporal paired (default)
surrealdb-migraine add --temporal "Add audit table"

# specify directory and increase verbosity
smg add --dir ./migrations -v "Fix schema"

# Single-file mode (legacy behavior)
# Use `--single` to create a single `.surql` file instead of a paired folder
smg add --single "Create users"
surrealdb-migraine add --single --temporal "Add index"
```

CLI quick reference

- `add <NAME>` — create a migration file using NAME (sanitized).
- `--temporal` / `-t` — use timestamp prefix instead of numeric.
- `--dir <DIR>` — override migrations directory (defaults to ./migrations).
- `-v, -vv` — increase logging verbosity (debug/trace).

Notes on binary names

- The project provides two executable names that point to the same CLI: `smg` and `surrealdb-migraine`.
- If you install from crates.io with `cargo install surreal-migraine`, Cargo will install the crate's binaries (both `smg` and `surrealdb-migraine` when available).
- To install a specific binary from the local checkout:

```powershell
# install only `smg` from the current path
cargo install --path . --bin smg

# install only `surrealdb-migraine`
cargo install --path . --bin surrealdb-migraine
```

Notes

- Names are sanitized (whitespace -> underscores, invalid chars removed).
- Numeric mode picks the next numeric prefix (e.g. `000_...`, `001_...`).
- Temporal mode uses a timestamp `YYYYMMDDHHMMSS` and will append a suffix if a collision occurs.