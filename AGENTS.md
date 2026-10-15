# AGENTS.md

Agent daemon in Rust (Actix-web 4) exposing a REST API to exec commands, upload and download files on a machine.

## Commands

- Build: `cargo build`
- Test: `cargo test`
- Format check: `cargo fmt --all -- --check`
- Lint: `cargo clippy -- -D warnings`
- Security audit: `cargo audit`
- Run: `cargo run` (API on `:3000`, Prometheus metrics on `:9000/metrics`)

## Validation gate (required before every commit)

The CI enforces these; run them locally before committing, in this order:

```sh
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo test
cargo audit
plumber analyze
```

All five must pass. A commit that fails any of them will be rejected by CI (`.github/workflows/ci.yml`):
- `cargo fmt` / `cargo clippy` are blocking jobs — code style and warnings are non-negotiable
- `cargo audit` is a blocking job — a new RUSTSEC advisory must be either fixed (dependency update) or explicitly ignored in `.cargo/audit.toml` with a new entry in `docs/DECISIONS.md` explaining why
- `plumber analyze` is a blocking job — the CI/CD security gate (`.plumber.yaml` overlay) must score 100/100; any new finding must be fixed, not waived

## Plumber version policy

`plumber` (CI/CD security gate) must **always be pinned to the latest release**, in two places that move together:

- `.mise.toml` — pinned version for local runs (`mise install`)
- `.github/workflows/ci.yml` (`plumber` job) — pinned version **and** the official SHA256 checksum of the binary, verified before execution

This is **enforced automatically**: a daily scheduled job (`plumber_version_check`) fails when the pins are outdated or inconsistent with each other. On such a failure, bump both places together (release notes: https://github.com/getplumber/plumber/releases), then re-run `plumber analyze` — a new version may introduce new controls and new findings.

## Structure

- `src/main.rs` — bootstrap, route table (`/admin/*` guarded)
- `src/conf.rs` — configuration from `config.json` (optional, defaults in code)
- `src/guards.rs` — auth guard: `Authorization` header must equal Base64(`user:password`)
- `src/handlers/` — one module per endpoint (`welcome`, `ping`, `download`, `upload`, `execute`)

## Conventions

- Rust edition 2024, formatting enforced by `cargo fmt`, warnings denied via clippy
- Error handling with `thiserror` for config errors
- Tests live in the same file as the handler (`#[cfg(test)]` + `actix_web::test`)
- Commit messages: conventional style (`fix:`, `feat:`, ...)

## Gotchas

- **cargo-audit ignores** live in `.cargo/audit.toml` (NOT root `audit.toml`, NOT `Cargo.toml` — cargo-audit 0.22+ only reads `.cargo/audit.toml`). See `docs/DECISIONS.md`.
- **plumber version** is pinned in TWO places that must be bumped together: `.mise.toml` and the `plumber` job in `.github/workflows/ci.yml` (version + SHA256 checksum). A daily scheduled CI job fails when the pin is no longer the latest release. See `docs/DECISIONS.md` (D-006).
- `README.md` is partially outdated: config keys are lowercase (`host`, `port`, `user`, `password`) with defaults `teddy`/`rocks`; default port is 3000.
- `/admin/exec` passes `parameters` as a single argument (no shell, no multi-args) — do not change semantics without updating `docs/DECISIONS.md`.
- RUSTSEC-2026-0258 (h2, via actix-http v3) is ignored on purpose. Re-evaluate when actix-web upgrades to h2 0.4+.

## Security sensitivity

This is a remote-execution agent by design: `/admin/exec` and `/admin/download` allow arbitrary commands and file reads. Never weaken the auth guard, never log tokens, and do not broaden these endpoints without an explicit decision record.
