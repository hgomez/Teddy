# Decision Log

Architectural and security decisions for Teddy, newest first. Each entry: date, context, decision, consequences.

---

## D-005 — CI hardening: pinned actions, least-privilege token, plumber allowlist

**Date:** 2026-10-15

**Context:** `plumber analyze` scored the CI workflow 63/100 (FAILED): checkout persisted the `GITHUB_TOKEN` in `.git/config` (ISSUE-307), no explicit `permissions:` block so the token inherited the repo default scopes (ISSUE-801), and the third-party actions `dtolnay/rust-toolchain@stable` and `Swatinem/rust-cache@v2` were referenced by mutable refs (two HIGH attack paths).

**Decision:**
- All actions in `.github/workflows/ci.yml` are pinned to full commit SHAs (with a version comment); Dependabot keeps the pins fresh.
- Workflow-level `permissions: contents: read` (least privilege; no job needs more) and `persist-credentials: false` on every checkout.
- `dtolnay/rust-toolchain` is replaced by a plain `rustup toolchain install` run step: the toolchain no longer arrives through a third-party action, which removed the HIGH attack path entirely.
- `Swatinem/rust-cache` is kept (SHA-pinned) and explicitly trusted in a **minimal** `.plumber.yaml` overlay (`extends: plumber:default`) — it is below the default `minimumStars: 20000` floor. It must always stay SHA-pinned; removal of the allowlist entry or the pin fails the plumber gate.

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
