surreal-migraine
=================

A tiny CLI helper to create SurrealDB migration files (numeric or temporal prefixed).

Goals
- Create migration files quickly using a simple CLI.
- Support numeric (incrementing) prefixes and temporal (timestamp) prefixes.
- Keep names filesystem-safe by sanitizing input.
- Use structured logging and friendly errors for developer UX.

Quick summary
- Binary: `surreal-migraine` (provided by this Cargo package).
- Commands: `add` — create a new migration file.

Why this tool
- Small convenience for generating migration files that follow a predictable scheme:
  - Numeric: `000_name.surql`, `001_other.surql`, ...
  - Temporal: `20250818123045_name.surql` (falls back to suffixing if collision)

Install

Build from source (recommended for development):

```powershell
# from repository root
cargo build --release
# optionally install to cargo bin
cargo install --path .
```

Usage

Run directly with cargo (development)

```powershell
# create next numeric migration in ./migrations
cargo run -- add "Create users"

# create a temporal (timestamped) migration
cargo run -- add --temporal "Add audit table"

# specify an explicit migrations directory and enable verbose logging
cargo run -- add --dir ./db/migrations -v "Fix schema"
```

If you installed with `cargo install`, invoke the installed binary:

```powershell
surreal-migraine add "Create users"
surreal-migraine add --temporal "Add audit table" --dir ./migrations -vv
```

CLI summary
- `add <NAME>` — create a migration file with the provided name.
- `--temporal` — use a timestamp prefix instead of numeric.
- `--dir <DIR>` — override the migrations directory (defaults to `./migrations` or the current `migrations` dir when running inside one).
- `-v` / `-vv` — increase logging verbosity (maps to tracing env filter).

Behavior notes
- Names are sanitized: whitespace becomes underscores, invalid characters removed, and duplicate underscores collapsed.
- Numeric mode finds the highest existing numeric prefix and creates the next one atomically (retries on collisions).
- Temporal mode uses timestamp `YYYYMMDDHHMMSS` and appends a numeric suffix if a name already exists for that second.
- Files are created with a small header including the original name and creation timestamp.

Examples

Numeric example (first run creates `000_Create_users.surql`):

```powershell
cargo run -- add "Create users"
# -> migrations/000_Create_users.surql
```

Temporal example (timestamped name):

```powershell
cargo run -- add --temporal "Add audit table"
# -> migrations/20250818123045_Add_audit_table.surql
```

Developer guide

Run tests

```powershell
cargo test
```

Lint & format

```powershell
cargo fmt --all
cargo clippy -- -D warnings
```

Project layout
- `src/` — main program modules (CLI, fs helpers, name sanitization, regex consts).
- `tests/` — integration tests that exercise the binary behavior.

Design notes
- CLI parsing: `clap` (derive-based). This project follows common clap patterns: subcommands, flags, and `-v` count for verbosity.
- Time formatting: `chrono` for the temporal prefix.
- Logging: `tracing` + `tracing-subscriber` (env filter controlled by `RUST_LOG` or `-v`).
- Error handling: `eyre` + `color-eyre` for human-friendly errors.
- Regex: centralized regex statics for sanitization.

Contributing
- Open issues or PRs for bugs or enhancements.
- Keep changes small and add tests for behavior changes (especially filename logic).