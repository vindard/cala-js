# @vindard/cala-ledger

Node.js bindings for [cala-ledger](https://crates.io/crates/cala-ledger), a Rust double-entry accounting ledger built on Postgres / SQLx. Provides an in-process, async TypeScript API — no separate daemon, no IPC.

This is a [napi-rs](https://napi.rs/) wrapper around the published `cala-ledger` crate. The outbox server is intentionally not exposed. Not published to npm; consumers link this repo directly.

## Consuming from another project

You build the native addon here, then link it from the consumer repo. Two options:

### 1. file: dependency (recommended for sibling checkouts)

In the consumer's `package.json`:

```json
"dependencies": {
  "@vindard/cala-ledger": "file:../cala-js"
}
```

Then, from this repo:

```bash
just build           # produces the .node binary + index.js + index.d.ts
```

From the consumer repo:

```bash
yarn install
```

Yarn copies the built artifact into `node_modules/@vindard/cala-ledger`. Re-run `just build` followed by `yarn install --force` in the consumer when you change the wrapper.

### 2. yarn link / npm link (symlink, survives rebuilds)

```bash
# in this repo
just build
yarn link

# in the consumer repo
yarn link @vindard/cala-ledger
```

Subsequent `just build` runs in this repo are picked up by the consumer without re-linking.

Either way, the consumer needs to be on the same triple (OS + arch + libc) as where you built the `.node`. For typical dev on one laptop that's a non-issue.

## Usage

```ts
import { CalaLedger } from "@vindard/cala-ledger";

const cala = await CalaLedger.connect({
  pgCon: process.env.PG_CON!,
  maxConnections: 10,
});

const journal = await cala.journals().create({ name: "Main" });

const cash = await cala.accounts().create({
  code: "ASSETS:CASH",
  name: "Cash",
});

const list = await cala.accounts().list({ first: 50 });
```

The auto-generated `index.d.ts` is the full API contract. Surface summary:

- `CalaLedger.connect(config)` — establishes the PG connection pool and runs cala's migrations.
- `cala.accounts()` — create, list (paginated by name cursor).
- `cala.journals()` — create, find by id.
- `cala.txTemplates()` — create, find by code.
- `cala.transactions()` — find by id / external id, void, post.

## Development

This repo uses a Nix flake plus direnv for the toolchain, `process-compose` for a local Postgres, and `just` as the task runner.

### One-time setup

```bash
direnv allow         # activates the flake; provisions just, process-compose, postgres, node, yarn, rust
yarn install
```

After `direnv allow`, every subsequent `cd` into this directory auto-loads the toolchain. `PG_CON` is exported automatically by the devshell, pointed at `127.0.0.1:5433`.

### Common tasks

```bash
just                 # list every recipe
just dev-up          # start postgres (foreground; Ctrl-C cleans up)
just build           # compile the native addon (release)
just test            # load-only smoke tests
just test-it         # load + integration; needs `just dev-up` running
just dev-down        # stop a detached process-compose
just dev-reset       # wipe .dev/pg and reinit
just clippy          # cargo clippy --all-targets -- -D warnings
just fmt             # cargo fmt
```

### Build flow

`yarn build` invokes `napi build --platform --release`, which:

1. Cargo-compiles `cala-ledger` 0.15.11 + this wrapper into a `cala-ledger.<triple>.node` cdylib in the project root.
2. Regenerates `index.js` and `index.d.ts` from the `#[napi]` annotations in `src/`.

`index.js`, `index.d.ts`, and the `.node` file are gitignored — they're recreated on every build. After a successful `just build`, the directory contains everything a `file:` linker needs.

### Tests

- `__test__/load.spec.mjs` — verifies the addon loads and exports the expected classes. No Postgres required.
- `__test__/integration.spec.mjs` — connects to Postgres and exercises journal + account flows. Skipped if `PG_CON` is unset.

## Origin

The original Node bindings lived inside the cala monorepo at `cala-nodejs/` and were removed in [GaloyMoney/cala#615](https://github.com/GaloyMoney/cala/pull/615) on 2025-11-22. This repo recreates that wrapper as a standalone crate pinned to the published `cala-ledger` 0.15.11. Two surface changes from the removed source:

- The outbox server (`OutboxServerConfig`, `awaitOutboxServer()`) is not exposed. cala-ledger no longer ships an in-process gRPC outbox; if you need outbox events, rebuild on the new `register_outbox_listener()` API.
- Pagination cursors are `AccountByNameCursor` (singular), matching cala-ledger's current type names.
