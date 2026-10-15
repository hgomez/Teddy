#!/bin/sh
# Builds release binaries into dist/.
# Usage: ./build.sh [target-triple ...]
#   no argument: build for the host only
#   aarch64 cross-build requires: rustup target add aarch64-unknown-linux-gnu,
#     a GNU cross linker (e.g. gcc-aarch64-linux-gnu) exported as
#     CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc,
#     and the target libc headers (Debian/Ubuntu: libc6-dev-arm64-cross)
set -eu

mkdir -p dist
rm -f dist/teddy-*

# target triple -> artifact name (teddy-linux-<arch>)
name() {
    case $1 in
        x86_64-unknown-linux-gnu)  echo linux-amd64 ;;
        aarch64-unknown-linux-gnu) echo linux-arm64 ;;
        *) echo "$1" ;;
    esac
}

build() {
    target=$1
    out=dist/teddy-$(name "$target")
    cargo build --release --locked --target "$target"
    cp "target/$target/release/teddy" "$out"
    echo "$out"
}

if [ $# -eq 0 ]; then
    cargo build --release --locked
    cp target/release/teddy dist/teddy
    echo dist/teddy
else
    for target in "$@"; do
        build "$target"
    done
fi
