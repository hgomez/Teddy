# Decision Log

Architectural and security decisions for Teddy, newest first. Each entry: date, context, decision, consequences.

---

## D-009 — Adopt the Apache License 2.0

**Date:** 2026-10-15

**Context:** The repository had no LICENSE file; the packaging metadata provisionally declared GPL-3.0-or-later (mirroring ycast-go), pending a maintainer decision (flagged in D-008).

**Decision:** Teddy is released under the **Apache License 2.0** (`LICENSE` at the repo root, `license = "Apache-2.0"` in `Cargo.toml`, rpm `License: Apache-2.0` with the file shipped as `%license`, deb copyright in DEP-5 format referencing the shipped text).

**Consequences:** Permissive licensing compatible with all dependencies; the GPL-3.0 mention in earlier packaging drafts is obsolete. Any future dependency must remain Apache-2.0-compatible (it is a superset-compatible choice for typical MIT/BSD/Apache-2.0 deps; GPL-only dependencies would now be the ones to reject).

---

## D-008 — Native Debian and RPM packaging, tag-triggered release pipeline

**Date:** 2026-10-15

**Context:** Teddy is a deployable agent; admins expect native packages (`.deb`/`.rpm`) and release binaries rather than a source-only repository. Pattern borrowed from the same maintainers' `hgomez/ycast-go` release pipeline, adapted to Rust.

**Decision:**
- `packaging/deb/` (debhelper 13 tree) and `packaging/rpm/` (spec) build the package from source with `cargo build --release --locked`; crates are fetched from crates.io (no vendoring — documented in both packaging READMEs). A `teddy` system user is created by the package scripts; the systemd unit (`deploy/teddy.service`, hardened) is installed in `/etc/teddy` working-directory mode but **never enabled or started**: the shipped `config.json` carries the public default credentials and Teddy is a remote-execution agent (D-001). `/etc/teddy/config.json` is a conffile / `%config(noreplace)`.
- `.github/workflows/release.yml` triggers on `v*` tags: a `verify` job rejects any tag that does not match the `Cargo.toml` version (the binary version comes from `env!("CARGO_PKG_VERSION")`), then three parallel jobs produce `teddy-linux-{amd64,arm64}` binaries (`build.sh`, arm64 via the GNU cross linker), the `.deb` (debian:trixie container) and the `.rpm` (fedora:44 container); a final job publishes with the `gh` CLI.
- Security posture identical to `ci.yml` (plumber gate): GitHub-owned actions only, pinned to SHAs; container images pinned by digest; `persist-credentials: false` everywhere; `contents: read` except `contents: write` on the release job; no third-party release action, no cache in the release path.
- Licensing: packaging metadata declares **Apache-2.0** (see D-009); the repository carries a `LICENSE` file shipped as `%license` (rpm) and in `/usr/share/doc/teddy/` (deb).

**Consequences:** A release is `bump Cargo.toml → tag v<X> → push`, fully automated after that. The deb/rpm builds require network access to crates.io at build time (accepted trade-off vs. vendoring ~277 crates in git). Package version is stamped by CI `sed` from the tag, like ycast-go.

---

## D-007 — Move to Rust edition 2024

**Date:** 2026-10-15

**Context:** The toolchain moved to Rust 1.99 with zero friction (full validation gate passed, see D-005/D-006 for the gate). Edition 2021 is still supported, but edition 2024 (stabilized in Rust 1.85) is the current default for new code and brings safer defaults (e.g. stricter closures/unsafe rules, updated `impl Trait` capture rules, style_edition 2024 import ordering in rustfmt).

**Decision:**
- Bump `Cargo.toml` to `edition = "2024"`.
- `cargo fix --edition` reported no code changes required; the only mechanical impact was import reordering by rustfmt's 2024 style edition (applied via `cargo fmt --all`).
- The validation gate (fmt, clippy `-D warnings`, test, audit, plumber) passes unchanged.

**Consequences:** No behavior change; dependency and CI setup untouched (CI installs `stable`, which supports edition 2024). Contributors need a Rust toolchain >= 1.85 to build; contributors on older toolchains will get a clear edition error rather than a silent mismatch.

---

## D-006 — Plumber: exact pin, enforced in CI by the official action

**Date:** 2026-10-15

**Context:** Plumber is the CI/CD security gate (D-005). Its verdicts depend on the version: each release can add controls, so a stale plumber silently weakens the gate while an unpinned one makes runs non-reproducible. Both failure modes had to be excluded. The first iteration (manual curl + SHA256 checksum in `ci.yml`, daily `plumber_version_check` job forcing the bump) is superseded by this decision, which adopts the same strategy as ycast-go.

**Decision:**
- Plumber is **pinned to an exact version** (never `latest`) in `mise.toml` for local runs, and updated **manually** with `mise run update-plumber` (`scripts/update-plumber.sh`: reads the latest stable GitHub release, never downgrades, rewrites the pin).
- The analysis runs in CI (`.github/workflows/plumber.yml`, its own workflow) through the **official** `getplumber/plumber` GitHub Action, SHA-pinned to the release matching the mise.toml pin. The action verifies the installed binary against the release `checksums.txt` and its SLSA build-provenance attestation.
- CI is gated at `min-points: 90` and `score-push: true` publishes the live Plumber Score badge shown in the README. The job is a required status check on `main`, so a pull request that drops the score below 90 cannot be merged. The repo must stay at 100/100; the 90-point floor exists to absorb a minor finding introduced by a new plumber control without hard-blocking unrelated work — such a finding must still be fixed promptly, and the floor must never be lowered to accommodate a real regression.
- The plumber action is **the deliberate third-party action** (alongside `Swatinem/rust-cache`, D-005) explicitly allowed in the `githubActionMustComeFromAuthorizedSources` overlay in `.plumber.yaml` — the policy tool polices its own dependency. The daily scheduled `plumber_version_check` job was removed on purpose: a pin bump must be a reviewed, deliberate change.
- Update process: run `mise run update-plumber`, move the action SHA + version comment in `.github/workflows/plumber.yml` to the same release, validate the overlay with the new version (`plumber config validate` + `plumber analyze`), and fix any new findings (never waive them).

**Consequences:** Deterministic, checksum- and attestation-verified analysis in CI and locally, with a score floor of 90 (a merge cannot drop it below that; a fix is needed to recover). The pin can drift from upstream between manual updates — that is acceptable for a security gate that should only move on purpose. The cost of the gate — a new plumber release surfacing a new finding breaks the pin bump — is deliberate: it fails loudly, and the bump is validated manually before landing anyway.

---

## D-005 — CI hardening: pinned actions, least-privilege token, plumber allowlist

**Date:** 2026-10-15

**Context:** `plumber analyze` scored the CI workflow 63/100 (FAILED): checkout persisted the `GITHUB_TOKEN` in `.git/config` (ISSUE-307), no explicit `permissions:` block so the token inherited the repo default scopes (ISSUE-801), and the third-party actions `dtolnay/rust-toolchain@stable` and `Swatinem/rust-cache@v2` were referenced by mutable refs (two HIGH attack paths).

**Decision:**
- All actions in `.github/workflows/ci.yml` are pinned to full commit SHAs (with a version comment); Dependabot keeps the pins fresh.
- Workflow-level `permissions: contents: read` (least privilege; no job needs more) and `persist-credentials: false` on every checkout.
- `dtolnay/rust-toolchain` is replaced by a plain `rustup toolchain install` run step: the toolchain no longer arrives through a third-party action, which removed the HIGH attack path entirely.
- `Swatinem/rust-cache` is kept (SHA-pinned) and explicitly trusted in a **minimal** `.plumber.yaml` overlay (`extends: plumber:default`) — it is below the default `minimumStars: 20000` floor. It must always stay SHA-pinned; removal of the allowlist entry or the pin fails the plumber gate. (Later tightened to an explicit allowlist of `Swatinem/rust-cache` + `getplumber/plumber` only, see D-006.)

**Consequences:** `plumber analyze` now scores 100/100 (PASSED). The allowlist entry in `.plumber.yaml` and the pin in `ci.yml` must move together — if the action is ever swapped or unpinned, the decision must be revisited here.

---

## D-004 — Keep Actix-web 4 (framework evaluation)

**Date:** 2026-10-15

**Context:** Question raised whether Actix-web 4 is still the right framework. Measured facts: 20 MB RSS at rest ("low memory" constraint met); 277 total deps; actix-web hello-world pulls ~155 deps vs ~61 for axum; the binary compiles two HTTP/2 stacks (h2 0.3.27 pinned by actix-http 3.18 — the source of the RUSTSEC-2026-0258 ignore in D-003 — and h2 0.4.20 pulled via hyper, itself pulled by `metrics-exporter-prometheus`). Actix-web provides everything Teddy needs first-party: multipart upload with size limit (`actix-multipart`), file serving (`actix-files`), Prometheus middleware (`actix-web-metrics`), guards, and integration test utilities — all five handlers and tests rely on them. Main downsides: the h2 0.3 pin (no patched 0.3.x release; fix requires the not-yet-released actix-http 4 / actix-web 5), the dual HTTP/2 stack, and the heavier legacy dependency tree compared to the axum/hyper/tokio baseline, which is now the recommended default for new Rust web projects.

**Decision:** Stay on Actix-web 4. A migration to axum would mean rewriting all handlers, converting the guard to middleware, replacing multipart/static-file/metrics/test layers — for no visible product change and no meaningful performance gain for a local admin agent. Re-evaluate at two trigger points:
1. Release of actix-web 5 (h2 0.4+): straightforward upgrade, removes the RUSTSEC-2026-0258 ignore from D-003. Expected scenario.
2. Significant functional growth of the project: axum becomes the natural successor (`axum::extract::Multipart`, `tower-http::ServeDir`, and a hyper-free Prometheus exporter to eliminate the h2 duplication).

**Consequences:** No code change now. The h2 0.3 pin and the dependency weight remain accepted risks, mitigated operationally (admin service typically behind a reverse proxy). Track actix-web issue #4199 for the h2 upgrade path.

---

## D-003 — Ignore RUSTSEC-2026-0258 (h2) and move cargo-audit config to `.cargo/audit.toml`

**Date:** 2026-10-15

**Context:** `cargo audit` fails on RUSTSEC-2026-0258 (DoS via unbounded empty DATA frames, h2 0.3.27) which is pinned by `actix-http` v3 / actix-web 4. Fixing requires h2 >= 0.4.16, not yet supported by actix-web 4. Additionally, cargo-audit 0.22 no longer reads a root-level `audit.toml` nor the `[advisories]` table in `Cargo.toml`, so the existing ignore file was dead config.

**Decision:**
- Track ignores in `.cargo/audit.toml` (the only location read by cargo-audit 0.22+).
- Ignore RUSTSEC-2026-0258 until actix-web supports h2 0.4+.
- CI runs plain `cargo audit` (no `--ignore` flag) so the committed file is the single source of truth.

**Consequences:** Audit passes locally and in CI. Risk of the DoS is mitigated operationally (service expected to sit behind a reverse proxy / not exposed raw). Must remove the ignore once actix-web upgrades (tracked via actix-web issue #4199).

---

## D-002 — Weekly grouped Dependabot updates

**Date:** 2026-03-18

**Context:** Frequent individual dependency PRs create review noise and CI churn.

**Decision:** Dependabot runs weekly on cargo, grouping all updates in one PR (`rust-dependencies` group), max 5 open PRs, labels `dependencies` + `rust`.

**Consequences:** Single reviewable dependency PR per week; minor delay in receiving security patches (max 7 days).

---

## D-001 — Actix-web 4, single-file config, Base64 token auth

**Date:** initial design

**Context:** Small, self-contained cross-platform daemon (Linux/Mac/Windows) with low memory footprint, controlled over REST.

**Decision:**
- Actix-web 4 as HTTP framework; one handler module per endpoint under `src/handlers/`.
- Optional `config.json` (keys: `host`, `port`, `user`, `password`); hardcoded defaults (`0.0.0.0:3000`, `teddy`/`rocks`) with warn-and-fallback on missing/broken file.
- Auth: `Authorization` header compared against Base64(`user:password`) computed at startup, enforced by an Actix guard on the `/admin/*` scope. No TLS in-process — deployments put Teddy behind a reverse proxy.
- `/admin/exec` runs the command with `parameters` as a single argument (no shell interpolation); quotes are stripped.

**Consequences:** Minimal footprint and simple deployment. Limitations accepted: credentials in plain file, non-standard auth header (no `Basic ` prefix), no multi-argument commands, no path restriction on `/admin/download`. Hardening (constant-time token comparison, path allowlist, multi-args) is deferred and must be recorded here when decided.
