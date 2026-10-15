# Packaging Debian

Sources for the `teddy` package (standard debhelper `debian/` tree).

## Structure

- `debian/control` — metadata and `Build-Depends` (`debhelper-compat 13`, `cargo`, `rustc >= 1.85`)
- `debian/rules` — the Rust build goes through `override_dh_auto_build` (`cargo build --release --locked`, same commands as the README); the systemd unit is installed but **neither enabled nor started** (`--no-enable --no-start`): credentials must be set first. Cargo downloads the crates pinned in `Cargo.lock` from crates.io at build time, so the build host needs network access
- `debian/install` — systemd unit, `deploy/config.json` → `/etc/teddy/config.json` (conffile), README
- `debian/postinst` — creates the unprivileged `teddy` system user the unit runs as
- `debian/changelog` — the package version is read from here; CI seds it from the git tag
- `debian/copyright` — machine-readable DEP-5 format

## Build

On a Debian/Ubuntu machine (or the `debian:trixie` container used by CI, see `.github/workflows/release.yml`):

```sh
# debian/ must sit at the top of the source tree
cp -r packaging/deb/debian .
dpkg-buildpackage -us -uc -b
rm -rf debian ../teddy_*            # cleanup
```

The result (`../teddy_<version>_<arch>.deb`) contains:

- `/usr/bin/teddy`
- `/etc/teddy/config.json` (conffile: local changes survive upgrades)
- `/usr/lib/systemd/system/teddy.service`
- `/usr/share/doc/teddy/`

## systemd

`debian/rules` passes `--no-enable --no-start`: the unit is installed but not
enabled, because the shipped `config.json` carries the built-in default
credentials (`teddy`/`rocks`) and Teddy is a remote-execution agent. After
setting strong credentials:

```sh
systemctl enable --now teddy
```

## Version

`debian/changelog` is the source of truth for the package version. The CI
rewrites the header from the git tag before `dpkg-buildpackage`:

```sh
sed -i "1s/teddy (.*)/teddy (${VERSION})/" debian/changelog
```

Format `3.0 (native)`: no `orig` tarball needed, the CI builds straight from
the source tree.
