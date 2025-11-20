#!/usr/bin/env bash

TEST_NAME="concurrent_form_submission_is_handled_gracefully"

export RUST_LOG="sqlx=error,info"
export TEST_LOG=enabled
cargo t $TEST_NAME | bunyan