#!/usr/bin/env bash

set -eu

cargo +nightly contract build --debug --manifest-path util/Cargo.toml
cargo +nightly contract build --debug --manifest-path asset/Cargo.toml
cargo +nightly contract build --debug --manifest-path oracle/Cargo.toml
cargo +nightly contract build --debug --manifest-path distributor/Cargo.toml
cargo +nightly contract build --debug --manifest-path boardroom/Cargo.toml
cargo +nightly contract build --debug --manifest-path treasury/Cargo.toml