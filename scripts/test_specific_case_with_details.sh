#!/usr/bin/env bash

TEST_NAME="confirmed_subscriber_should_get_a_newsletter"

export RUST_LOG="sqlx=error,info"
export TEST_LOG=enabled
cargo t $TEST_NAME | bunyan