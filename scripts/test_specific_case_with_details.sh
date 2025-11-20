#!/usr/bin/env bash

TEST_NAME="newsletter_creation_is_idempotent"

export RUST_LOG="sqlx=error,info"
export TEST_LOG=enabled
cargo t $TEST_NAME | bunyan