#!/usr/bin/env bash

TEST_NAME="request_missing_auth_rejected"

export RUST_LOG="sqlx=error,info"
export TEST_LOG=enabled
cargo t $TEST_NAME | bunyan