#!/usr/bin/env bash

set -euo pipefail

if ! [ -z "$(git status --porcelain=v1 2>/dev/null)" ]; then
    echo "You have unstaged changes."
    exit 1
fi

cd "$(git rev-parse --show-toplevel)"

function tombl() {
    cargo +nightly run --release -- "$@"
}

# We don't *need* nightly, but want to use -Z build-std to make the executable
# smaller
cargo +nightly build \
    --release \
    -Z build-std=std,panic_abort \
    -Z build-std-features=panic_immediate_abort \
    --target x86_64-unknown-linux-musl
eval "$(tombl -e meta=package Cargo.toml)"
name="${meta[name]}-v${meta[version]}"
out="releases/$name"
mkdir -p "$out"
cp "target/x86_64-unknown-linux-musl/release/${meta[name]}" "$out"
strip "$out/${meta[name]}"
tar cfz "$out.tar.gz" "$out"
rm -rf "$out"
