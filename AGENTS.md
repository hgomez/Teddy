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

All five must pass. A commit that fails any of them will be rejected by CI (`.github/workflows/ci.yml` and `.github/workflows/plumber.yml`):
- `cargo fmt` / `cargo clippy` are blocking jobs — code style and warnings are non-negotiable
- `cargo audit` is a blocking job — a new RUSTSEC advisory must be either fixed (dependency update) or explicitly ignored in `.cargo/audit.toml` with a new entry in `docs/DECISIONS.md` explaining why
- `plumber analyze` is a blocking job — the CI/CD security gate (`.plumber.yaml` overlay) must score 100/100 (`min-points: 100` in `.github/workflows/plumber.yml`); any new finding must be fixed, not waived

## Plumber version policy

`plumber` (CI/CD security gate) is **pinned to an exact version** (never `latest`) in `mise.toml`, and in `.github/workflows/plumber.yml` through the **official** `getplumber/plumber` GitHub Action (SHA-pinned to the release matching the mise.toml pin; the action verifies the binary against checksums.txt + SLSA attestation).

The pin is **updated manually, never automatically** (no scheduled version-check job — a bump must be a reviewed, deliberate change):

```sh
mise run update-plumber   # scripts/update-plumber.sh: bumps the mise.toml pin to the latest release, never downgrades
```

Then move the action SHA + version comment in `.github/workflows/plumber.yml` to the same release, validate with the new version (`plumber config validate` + `plumber analyze`), and fix any new findings. Release notes: https://github.com/getplumber/plumber/releases

## Structure

- `src/main.rs` — bootstrap, route table (`/admin/*` guarded)
- `src/conf.rs` — configuration from `config.json` (optional, defaults in code)
- `src/guards.rs` — auth guard: `Authorization` header must equal Base64(`user:password`)
- `src/handlers/` — one module per endpoint (`welcome`, `ping`, `download`, `upload`, `execute`)
- `deploy/` — systemd unit (`teddy.service`) and example `config.json`
- `packaging/` — native packages: `deb/` (debian/ tree), `rpm/` (spec file)
- `.github/workflows/` — `ci.yml` (validation gate), `plumber.yml` (CI/CD security scan + score badge) and `release.yml` (tag-triggered packaging + release)
- `scripts/update-plumber.sh` — bumps the plumber pin in `mise.toml` (run via `mise run update-plumber`)
- `build.sh` — release binaries into `dist/` (host or cross-compiled)

## Conventions

- Rust edition 2024, formatting enforced by `cargo fmt`, warnings denied via clippy
- Error handling with `thiserror` for config errors
- Tests live in the same file as the handler (`#[cfg(test)]` + `actix_web::test`)
- Commit messages: conventional style (`fix:`, `feat:`, ...)

## Gotchas

- **cargo-audit ignores** live in `.cargo/audit.toml` (NOT root `audit.toml`, NOT `Cargo.toml` — cargo-audit 0.22+ only reads `.cargo/audit.toml`). See `docs/DECISIONS.md`.
- **plumber version** is pinned in `mise.toml` and in `.github/workflows/plumber.yml` (official `getplumber/plumber` action, SHA + version comment) — both move together on a bump, done manually with `mise run update-plumber` (NOT `.mise.toml`; there is no scheduled version-check job anymore). See `docs/DECISIONS.md` (D-006).
- **plumber allowlist** (`.plumber.yaml`) is an explicit list: `Swatinem/rust-cache` + `getplumber/plumber` only. Any new third-party action must be added there AND SHA-pinned in the workflows, or the gate fails.
- **runner label** is pinned to `ubuntu-24.04` (NOT `ubuntu-latest` — the label migrated to Ubuntu 26 on 2026-10-19 and must not be used). Keep GitHub actions on node24 versions (checkout v7+, upload-artifact v7+, download-artifact v8+), pinned by SHA.
- `README.md` is partially outdated: config keys are lowercase (`host`, `port`, `user`, `password`) with defaults `teddy`/`rocks`; default port is 3000.
- `/admin/exec` passes `parameters` as a single argument (no shell, no multi-args) — do not change semantics without updating `docs/DECISIONS.md`.
- RUSTSEC-2026-0258 (h2, via actix-http v3) is ignored on purpose. Re-evaluate when actix-web upgrades to h2 0.4+.

## Security sensitivity

This is a remote-execution agent by design: `/admin/exec` and `/admin/download` allow arbitrary commands and file reads. Never weaken the auth guard, never log tokens, and do not broaden these endpoints without an explicit decision record.
