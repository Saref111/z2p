#!/usr/bin/env bash

TEST_NAME="subscribe_fails_if_there_is_fatal_db_error"

export RUST_LOG="sqlx=error,info"
export TEST_LOG=enabled
cargo t $TEST_NAME | bunyan