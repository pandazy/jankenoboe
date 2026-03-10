# Task: Clean Removability & Docker E2E Testing

**Completed:** 2026-02-19

## Goals
1. Make the CLI cleanly removable (uninstall script + docs)
2. Set up Docker-based e2e test environment for the compiled/installed binary
3. Add GitHub Actions CI for Linux + macOS

## Checklist
- [x] Create `uninstall.sh` for binary installs
- [x] Update README.md with uninstall instructions
- [x] Create `e2e/Dockerfile` for e2e testing (multi-stage: build + test)
- [x] Create `e2e/run_tests.sh` — shell-based e2e test suite (41 tests)
- [x] Create `Makefile` for easy Docker e2e test execution
- [x] Add `.dockerignore` to exclude target/, .git/ from build context
- [x] Add `_progress_*.md` to `.gitignore`
- [x] Make e2e script portable (configurable paths via env vars)
- [x] Create `.github/workflows/e2e.yml` for Linux + macOS CI
- [x] Verify Docker build and all 41 e2e tests pass
- [x] Update documentation (README.md, AGENTS.md, docs/development.md, docs/structure.md)

## Files Created
- `uninstall.sh` — cross-platform binary uninstaller
- `e2e/Dockerfile` — multi-stage Docker build for e2e testing
- `e2e/run_tests.sh` — shell-based e2e test suite (41 assertions across 10 sections)
- `Makefile` — Docker e2e test targets (`make e2e`, `make clean-e2e`)
- `.dockerignore` — excludes target/, .git/, *.db from Docker build context
- `.github/workflows/e2e.yml` — GitHub Actions CI (Docker e2e, native Linux + macOS e2e, unit tests + clippy)

## Files Modified
- `README.md` — added Uninstallation section
- `AGENTS.md` — added e2e/, uninstall, CI docs
- `.gitignore` — added `_progress_*.md`
- `docs/development.md` — added E2E Testing, CI, and Uninstalling sections; updated Rust prerequisite to 1.85+
- `docs/structure.md` — added e2e/, uninstall.sh, Makefile, .dockerignore to directory layout

## E2E Test Sections (41 tests)
1. Binary Basics (4): --version, no args, missing JANKENOBOE_DB
2. Invalid Table Errors (4): get, search, duplicates, create
3. CRUD Lifecycle (7): create → get → update → verify → delete
4. Search (3): search by name, count, value
5. Duplicates (2): duplicate group detection
6. Song-Artist Relationship (2): create song, verify FK
7. Bulk Reassign (3): reassign count, verify moved
8. Learning (4): learning-due empty, learning-batch creates record
9. Error Handling (6): update/delete nonexistent, empty batch, missing args
10. Uninstall Verification (3): binary exists, which resolves, rm removes

## Key Decisions
- Used `rust:slim-bookworm` (latest) for Docker base image since `jankensqlhub` v1.3.0 uses `let` chains requiring Rust 1.88+
- E2e script uses env vars (`JANKENOBOE_INIT_SQL`, `JANKENOBOE_BINARY_PATH`) with defaults for Docker, overridable for native CI
- Uninstall script checks both `~/.local/bin/` and `~/.cargo/bin/` to cover all install methods