# Development Guidelines

## Prerequisites

- Rust 1.85 or later
- [JankenSQLHub](https://github.com/pandazy/jankensqlhub) (installed via crates.io)
- Docker (for e2e tests)

## Building

```bash
cargo build --release    # Build optimized binary
cargo build              # Build debug binary
```

The binary is output to `target/release/jankenoboe` (or `target/debug/jankenoboe` for debug builds).

## Installing Locally

```bash
cargo install --path .   # Installs to ~/.cargo/bin/jankenoboe
```

## Testing

```bash
cargo test
```

## Test Coverage

Generate coverage reports using [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov):

### Setup (one-time)

```bash
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview
```

### Usage

```bash
# Summary table
JANKENOBOE_DB=":memory:" cargo llvm-cov --no-cfg-coverage --summary-only

# Detailed line-by-line text report
JANKENOBOE_DB=":memory:" cargo llvm-cov --no-cfg-coverage --text

# HTML report (opens in browser)
JANKENOBOE_DB=":memory:" cargo llvm-cov --no-cfg-coverage --html
open target/llvm-cov/html/index.html
```

> **Note:** `--no-cfg-coverage` is used because the project does not use `#[cfg(coverage)]` conditional compilation. The `JANKENOBOE_DB=":memory:"` env var uses an in-memory SQLite database for tests.

## Code Quality

After making changes, always run:

```bash
cargo clippy --fix --allow-dirty  # Fix compiler warnings
cargo fmt                          # Format code
```

## Testing Principles

- Do not use conditional assertions; if wrapped in a callback function, ensure the wrapper executes 100% of the time
- Ensure tests are simple and straightforward to read
- Aim for broader coverage with fewer, well-designed tests
- Prioritize testing for security issues, particularly SQL injection vulnerabilities
- Avoid loops in tests, except when iterating over arrays with obvious elements
- Do not over-engineer tests
- Always assert exact values; avoid vague assertions like `option.is_some` or `result.is_ok`
- Do not use special numbers like PI or E (triggers unnecessary linting errors)
- Test CLI commands with various parameter combinations
- Test error handling for invalid table names, missing fields, and malformed inputs

## Code Quality Standards

- Apply Occam's Razor; be minimalist
- Follow Rust API design principles and idioms
- Prefer explicit error handling over panics
- Use meaningful names for functions, variables, and types
- Add documentation comments for public APIs
- Keep functions small and focused on single responsibilities
- Use pattern matching for error handling and control flow
- If a variable has no possibility of being None within its flow, do not define it as an Option
- Use proper exit codes for different error scenarios (0 for success, 1 for errors)

## E2E Testing (Docker)

End-to-end tests run against the compiled, installed binary inside a Docker container — simulating real user installation and usage.

```bash
make e2e          # Build Docker image and run e2e tests
make e2e-build    # Build the e2e Docker image only
make e2e-run      # Run the e2e tests (image must exist)
make clean-e2e    # Remove the e2e Docker image
```

The e2e tests cover:
- Binary basics (--version, no args, missing env var)
- Invalid table error handling
- Full CRUD lifecycle (create, get, update, delete)
- Search and duplicate detection
- Song-Artist relationships
- Bulk reassignment
- Learning (due, batch)
- Error handling for nonexistent records
- Clean uninstall verification (binary removal)

See [`e2e/Dockerfile`](../e2e/Dockerfile) and [`e2e/run_tests.sh`](../e2e/run_tests.sh).

### CI (GitHub Actions)

On every pull request, GitHub Actions runs:
- **E2E (Docker/Linux):** `make e2e` — Docker-based e2e tests
- **E2E (Linux):** Native build + e2e on `ubuntu-latest`
- **E2E (macOS):** Native build + e2e on `macos-latest`
- **Unit Tests (Linux + macOS):** `cargo test` + `cargo clippy`

See [`.github/workflows/e2e.yml`](../.github/workflows/e2e.yml).

## Upgrading

Re-run the install script to upgrade — it always fetches and installs the latest release, overwriting the existing binary:

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

If you installed via Cargo:

```bash
cargo install --git https://github.com/pandazy/jankenoboe.git --force
```

See [README.md Upgrading](../README.md#upgrading) for full details.

## Uninstalling

```bash
# If installed via install.sh or manual copy:
sh uninstall.sh

# If installed via cargo:
cargo uninstall jankenoboe
```

See [README.md Uninstallation](../README.md#uninstallation) for full details.

## Releasing

### Creating a Release

To publish a new version:

1. Update [`release.md`](../release.md) in the project root with the release notes for this version
2. Commit the changes
3. Push a Git tag starting with `v`:

```bash
git tag v1.0.0
git push origin v1.0.0
```

This triggers the [release workflow](../.github/workflows/release.yml), which:
1. Builds binaries for all 3 platform targets
2. Packages each binary as a `.tar.gz` archive
3. Creates a GitHub Release using the contents of `release.md` as the release body, with all assets attached

### Cross-platform Builds

The release workflow builds for:

| Target | OS | Architecture | Runner |
|--------|----|-------------|--------|
| `x86_64-unknown-linux-gnu` | Linux | x86_64 | `ubuntu-latest` |
| `aarch64-unknown-linux-gnu` | Linux | ARM64 | `ubuntu-latest` (via cross) |
| `aarch64-apple-darwin` | macOS | Apple Silicon | `macos-latest` |

The release assets are named following the pattern expected by `install.sh`:
- `jankenoboe-linux-x86_64.tar.gz`
- `jankenoboe-linux-aarch64.tar.gz`
- `jankenoboe-macos-aarch64.tar.gz`

Each `.tar.gz` contains just the `jankenoboe` binary.

See [`.github/workflows/release.yml`](../.github/workflows/release.yml).

## Task Management

Progress files use the naming convention: `_progress_<task_name>.md`

When starting a new task:
1. Read the progress file to review completed work; if it doesn't exist, create one
2. If exceeding context window capacity, save progress to the progress file, ensuring content is clear and actionable for another LLM agent to resume

## JankenSQLHub Integration

- Use JankenSQLHub's parameter validation to prevent SQL injection
- Leverage `#[table]` with `enum` constraints for safe dynamic table names
- Use `~[fields]` with `enumif` for per-table field validation
- Use `enum` or `enumif` constraints to whitelist valid values per context
- `@param` defaults to string type — no need for `{"type": "string"}`
- Build queries using JankenSQLHub's `QueryDef` (JSON-configured)
- Execute queries using the appropriate runner (SQLite in this case)

## Documentation Updates

When asked to update documents, check if any of the following have out-of-sync facts:
- README.md
- AGENTS.md
- .clinerules
- docs/*.md
