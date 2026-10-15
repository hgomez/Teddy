# Releasing Teddy

How releases are produced: everything is automated by GitHub Actions, the
only human gesture is pushing a tag. This file documents the full flow, the
artifacts, and the recovery procedures.

## TL;DR

```sh
# 1. Bump the version in Cargo.toml (e.g. 2.1.4 -> 2.2.0), commit and push:
git commit -am "chore: release 2.2.0" && git push
#    -> check CI is green on main: gh run list --branch main --limit 3

# 2. Tag and push the tag:
git tag -a v2.2.0 -m "2.2.0" && git push origin v2.2.0
#    or in one step (creates tag + release on GitHub):
#    gh release create v2.2.0 --title v2.2.0 --generate-notes
```

That's it: the `verify`, `binaries`, `deb`, `rpm` and `release` jobs produce
and publish the binaries, the `.deb` and the `.rpm` automatically.
The only manual step is the `Cargo.toml` bump — `verify` rejects any tag
that does not match it.

## The pipeline at a glance

`.github/workflows/release.yml` triggers on pushing a tag matching `v*`.
The version must match `Cargo.toml` (`env!("CARGO_PKG_VERSION")` is compiled
into the binary, so the tag, `Cargo.toml`, the deb changelog and the rpm spec
all carry the same version):

```
git push origin v2.1.4
        │
        ├─ job verify      → tag == Cargo.toml version (fails fast otherwise)
        ├─ job binaries    → ./build.sh → teddy-linux-amd64, teddy-linux-arm64
        ├─ job deb         → debian:trixie container → teddy_2.1.4_amd64.deb
        ├─ job rpm         → fedora:44 container     → teddy-2.1.4-1.fc44.x86_64.rpm
        └─ job release     → gh release create --generate-notes + upload of everything
```

All build jobs run in parallel after `verify`; `release` waits for
`binaries`, `deb` and `rpm`. A failure in one job does not block the others.

## Step by step

1. **Check CI is green on `main`** — branch protection requires it for PRs,
   and a release built from a red `main` is a bad idea anyway:
   ```sh
   gh run list --branch main --limit 3
   ```
2. **Bump `Cargo.toml` `version`** to the release version (e.g. `2.1.4`) —
   the binary version comes from `env!("CARGO_PKG_VERSION")`, and the
   `verify` job rejects a tag that does not match it. Commit and let CI pass.
3. **Create the tag and the release**:
   ```sh
   git tag -a v2.1.4 -m "2.1.4" && git push origin v2.1.4
   # or in one step, creating both on GitHub:
   gh release create v2.1.4 --title v2.1.4 --generate-notes
   ```
4. **Watch the run**: https://github.com/hgomez/Teddy/actions — expect a few
   minutes end to end (the packaging containers are the slow part).
5. **Verify**:
   - https://github.com/hgomez/Teddy/releases shows the release with the
     two binaries, the `.deb` and the `.rpm`;
   - sanity-check an artifact locally, e.g. `dpkg -c teddy_*.deb` or
     `rpm -qlp teddy-*.rpm`.

## Version sources: what you maintain and what you don't

| Where | Who sets it | Notes |
|---|---|---|
| git tag | **you**, the trigger | must match `Cargo.toml`, enforced by `verify` |
| `Cargo.toml` `version` | you, before tagging | compiled into the binary (`env!("CARGO_PKG_VERSION")`) |
| `debian/changelog` | CI (`sed` before `dpkg-buildpackage`) | committed value is a placeholder |
| rpm spec `Version:` | CI (`sed` before `rpmbuild`) | same |

## Recovery procedures

A release went wrong? Everything is reversible:

```sh
gh release delete v2.1.4 --yes          # remove the GitHub release
git push origin :refs/tags/v2.1.4       # remove the remote tag
git tag -d v2.1.4                       # remove the local tag
# fix what needs fixing, then push the tag again — the pipeline re-runs
```

The `release` job itself is idempotent (`gh release upload --clobber` and a
conditional `gh release create`), so re-running over an existing release
merely refreshes the artifacts.

## Security posture of the pipeline

Enforced by the `.plumber.yaml` policy and `AGENTS.md`:

- only GitHub-owned actions (`actions/*`), pinned to commit SHAs;
- build container images pinned by digest (`debian:trixie`, `fedora:44`);
- `persist-credentials: false` on every checkout;
- least-privilege permissions: `contents: read` everywhere, `contents: write`
  only on the `release` job;
- no third-party release action — publishing uses the preinstalled `gh` CLI;
- no caching in the release path;
- `main` is branch-protected; the plumber gate (`ci.yml`) must be green
  before merge.

## Local package builds

See `packaging/deb/README.md` and `packaging/rpm/README.md`. Both builds run
`cargo build --release --locked`, which downloads the crates pinned in
`Cargo.lock` from crates.io (no vendoring in this repo), so the build host
needs network access.
