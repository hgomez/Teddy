# RPM spec for teddy.
#
# Build (from a machine with the Rust toolchain >= 1.85, edition 2024):
#
#   VERSION=2.1.4
#   git archive --format=tar.gz --prefix=teddy-${VERSION}/ v${VERSION} \
#       -o ~/rpmbuild/SOURCES/teddy-${VERSION}.tar.gz
#   rpmbuild -bb packaging/rpm/teddy.spec
#
# The build runs `cargo build --release --locked`, which downloads the
# crates pinned in Cargo.lock from crates.io (no vendoring in this repo);
# the build host needs network access to crates.io.
# Without git, create the tarball manually with a teddy-$VERSION/ top
# directory. The result lands in ~/rpmbuild/RPMS/<arch>/.

Name:           teddy
Version:        2.1.4
Release:        1%{?dist}
Summary:        Agent daemon exposing a REST API to exec commands and transfer files

License:        GPL-3.0-or-later
URL:            https://github.com/hgomez/Teddy
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.85
BuildRequires:  cargo
BuildRequires:  systemd-rpm-macros
Requires:       systemd

%global debug_package %{nil}

%description
Teddy is a small agent daemon written in Rust (Actix-web 4), deployable on
Linux, macOS and Windows. It exposes a REST API to execute a command (single
command, no shell), upload files via multipart form (up to 100 MB) and
download files from the machine. Prometheus metrics are exposed on a
dedicated port (:9000/metrics).

The /admin/* endpoints are protected by an Authorization header equal to
Base64(user:password). Configuration lives in /etc/teddy/config.json
(optional: sane defaults are built in). The systemd unit is installed but
not enabled: set strong credentials first, then run
"systemctl enable --now teddy". Never expose Teddy on an untrusted network:
it is a remote-execution agent by design.

%prep
%autosetup

%build
cargo build --release --locked

%install
install -D -m 0755 target/release/teddy %{buildroot}%{_bindir}/teddy
install -D -m 0640 deploy/config.json    %{buildroot}%{_sysconfdir}/teddy/config.json
install -D -m 0644 deploy/teddy.service  %{buildroot}%{_unitdir}/teddy.service
install -D -m 0644 README.md             %{buildroot}%{_pkgdocdir}/README.md

%pre
getent passwd teddy >/dev/null || useradd -r -d /nonexistent -s /sbin/nologin \
    -c "Teddy agent daemon" teddy

%files
%{_bindir}/teddy
%config(noreplace) %attr(0640,root,teddy) %{_sysconfdir}/teddy/config.json
%{_unitdir}/teddy.service
%doc %{_pkgdocdir}/README.md

%post
%systemd_post %{name}.service

%preun
%systemd_preun %{name}.service

%postun
%systemd_postun %{name}.service

%changelog
* Thu Oct 15 2026 Teddy maintainers <teddy@localhost> - 2.1.4-1
- Initial package.
