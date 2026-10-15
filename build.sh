#!/bin/sh
# Builds release binaries into dist/.
# Usage: ./build.sh [target-triple ...]
#   no argument: build for the host only
#   aarch64 cross-build requires: rustup target add aarch64-unknown-linux-gnu
#     and a GNU cross linker (e.g. gcc-aarch64-linux-gnu) exported as
#     CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
set -eu

mkdir -p dist
rm -f dist/teddy-*

build() {
    target=$1
    out=dist/teddy-${target%%-*}-${target##*-}
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
