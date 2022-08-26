#!/usr/bin/env bash

set -euo pipefail

if [[ -n "$(git status --porcelain=v1 2>/dev/null)" ]]; then
    echo "You have unstaged changes."
    exit 1
fi

cd "$(git rev-parse --show-toplevel)"

# We don't *need* nightly, but want to use -Z build-std to make the executable
# smaller
bin=tombl
cargo +nightly build \
    --release \
    -Z build-std=std,panic_abort \
    -Z build-std-features=panic_immediate_abort \
    --target x86_64-unknown-linux-musl \
    --bin "$bin"
tombl="target/x86_64-unknown-linux-musl/release/$bin"
eval "$($tombl -e meta=package Cargo.toml)"
name="${meta[name]}-v${meta[version]}"
out="releases/$name"
mkdir -p "$out"
cp "$tombl" "$out/$bin"
strip "$out/$bin"
cd releases
tar cfz "$name.tar.gz" "$name"
rm -rf "$name"
