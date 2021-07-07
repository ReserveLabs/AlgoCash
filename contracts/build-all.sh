#!/usr/bin/env bash

set -eu

cargo +nightly contract build --manifest-path util/Cargo.toml
cargo +nightly contract build --manifest-path asset/Cargo.toml
cargo +nightly contract build --manifest-path oracle/Cargo.toml
cargo +nightly contract build --manifest-path boardroom/Cargo.toml
cargo +nightly contract build --manifest-path treasury/Cargo.toml