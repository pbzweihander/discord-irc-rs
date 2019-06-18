#!/bin/bash -e
cargo fmt -- --check --verbose
cargo clippy
