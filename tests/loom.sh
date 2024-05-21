#!/usr/bin/env bash

# Runs loom tests with defaults for loom's configuration values.
#
# The tests are compiled in release mode to improve performance, but debug
# assertions are enabled.
#
# Any arguments to this script are passed to the `cargo test` invocation.

# Useful:
# LOOM_LOG=debug
# LOOM_CHECKPOINT_FILE=target/loom-checkpoint.json

time RUSTFLAGS="${RUSTFLAGS} --cfg loom -C debug-assertions" \
    RUST_BACKTRACE=full \
    LOOM_LOCATION=1 \
    cargo test --release --test loom --features loom "$@" -- --nocapture