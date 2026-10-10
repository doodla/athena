#!/bin/sh
# Athena's one setup command. It exists because nothing written in Rust can run before Rust is installed:
# it installs rustup from a pinned, checksum-verified rustup-init when cargo is missing, then hands off to
# `cargo run -p xtask -- setup`, which does everything else (docs/design/athena-v0.md §11). Idempotent.
set -eu
cd "$(dirname "$0")"

RUSTUP_VERSION=1.28.2
if ! command -v cargo >/dev/null 2>&1 && [ -x "${CARGO_HOME:-$HOME/.cargo}/bin/cargo" ]; then
    PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
fi
if ! command -v cargo >/dev/null 2>&1; then
    case "$(uname -s)-$(uname -m)" in
        Linux-x86_64) target=x86_64-unknown-linux-gnu; sum=20a06e644b0d9bd2fbdbfd52d42540bdde820ea7df86e92e533c073da0cdd43c ;;
        Linux-aarch64) target=aarch64-unknown-linux-gnu; sum=e3853c5a252fca15252d07cb23a1bdd9377a8c6f3efa01531109281ae47f841c ;;
        Darwin-x86_64) target=x86_64-apple-darwin; sum=9c331076f62b4d0edeae63d9d1c9442d5fe39b37b05025ec8d41c5ed35486496 ;;
        Darwin-arm64) target=aarch64-apple-darwin; sum=20ef5516c31b1ac2290084199ba77dbbcaa1406c45c1d978ca68558ef5964ef5 ;;
        *) echo "setup.sh: no pinned rustup-init for $(uname -s) $(uname -m); install Rust from https://rustup.rs" >&2; exit 1 ;;
    esac
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    curl -fsSL --retry 3 -o "$tmp/rustup-init" "https://static.rust-lang.org/rustup/archive/$RUSTUP_VERSION/$target/rustup-init"
    actual="$( (sha256sum "$tmp/rustup-init" 2>/dev/null || shasum -a 256 "$tmp/rustup-init") | cut -d' ' -f1)"
    [ "$actual" = "$sum" ] || { echo "setup.sh: rustup-init checksum mismatch (expected $sum, got $actual); refusing to run it" >&2; exit 1; }
    chmod +x "$tmp/rustup-init"
    "$tmp/rustup-init" -y --no-modify-path --profile minimal --default-toolchain none
    PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
fi
exec cargo run --quiet --locked --release -p xtask -- setup "$@"
