_default:
    @just --list

# ── dev infra (process-compose + postgres) ─────────────────────────────────

# Start postgres in the foreground; Ctrl-C cleans up.
dev-up:
    process-compose up

# Start postgres detached. Use `dev-attach` to view the TUI.
dev-up-detached:
    process-compose up -t=false --detached

# Attach to a detached process-compose session.
dev-attach:
    process-compose attach

# Stop a detached process-compose session.
dev-down:
    process-compose down

# Stop process-compose (if running) and wipe the local postgres data dir.
dev-clean:
    -process-compose down
    rm -rf .dev/pg

# ── js workflow ────────────────────────────────────────────────────────────

install:
    yarn install

# Release build of the native addon + index.{js,d.ts}.
build:
    yarn build

build-debug:
    yarn build:debug

# Load-only smoke tests (no postgres required).
test:
    yarn test

# Same as `test`, but the integration suite picks up PG_CON from the env
# (set automatically by `nix develop`). Make sure `just dev-up` is running.
test-it:
    yarn test

# ── rust hygiene ───────────────────────────────────────────────────────────

check:
    cargo check

clippy:
    cargo clippy --all-targets -- -D warnings

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

# ── cleanup ────────────────────────────────────────────────────────────────

clean:
    cargo clean
    rm -rf node_modules *.node index.js index.d.ts
