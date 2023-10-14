#!/bin/sh

set -xe

cargo fmt --all -- --check
cargo clippy --all --locked -- -D warnings
cargo test --all --locked
