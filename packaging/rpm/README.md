# Packaging RPM

Sources for the `teddy` package (standard RPM spec file).

## Build

On Fedora/RHEL/openSUSE with the Rust toolchain (>= 1.85, edition 2024):

```sh
# 1. Source tarball, with a teddy-<version>/ top directory
VERSION=2.1.4
git archive --format=tar.gz --prefix=teddy-${VERSION}/ v${VERSION} \
    -o ~/rpmbuild/SOURCES/teddy-${VERSION}.tar.gz
# (without git: tar czf ... --transform "s,^,teddy-${VERSION}/," \
#    --exclude target --exclude dist . )

# 2. The spec
mkdir -p ~/rpmbuild/SPECS && cp packaging/rpm/teddy.spec ~/rpmbuild/SPECS/

# 3. Build
rpmbuild -bb ~/rpmbuild/SPECS/teddy.spec
```

The `.rpm` lands in `~/rpmbuild/RPMS/<arch>/` and contains:

- `/usr/bin/teddy`
- `/etc/teddy/config.json` — `%config(noreplace)`, mode `0640 root:teddy`: local changes survive upgrades
- `/usr/lib/systemd/system/teddy.service`
- `/usr/share/doc/teddy/`

## Scriptlets

- `%pre` creates the unprivileged `teddy` system user the unit runs as
- `%systemd_post`/`%systemd_preun`/`%systemd_postun` handle daemon-reload and
  clean shutdown on removal

The unit is not enabled automatically: the shipped `config.json` carries the
built-in default credentials (`teddy`/`rocks`) and Teddy is a
remote-execution agent. Set strong credentials in `/etc/teddy/config.json`,
then `systemctl enable --now teddy`.

## Version

`Version:` and the `Source0` tarball must be aligned; the CI regenerates both
(a `sed` on the `Version:` field is enough).
