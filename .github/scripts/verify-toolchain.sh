#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/../.."
expected=$(sed -n 's/^channel = "\([^"]*\)"/\1/p' rust-toolchain.toml)

rustup show active-toolchain
for tool in rustc cargo; do
    version=$("$tool" --version)
    echo "$version"
    if [[ "$version" != "$tool $expected "* ]]; then
        echo "Expected $tool $expected from rust-toolchain.toml; check PATH and RUSTUP_TOOLCHAIN." >&2
        exit 1
    fi
done
