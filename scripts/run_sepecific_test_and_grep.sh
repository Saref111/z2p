#!/usr/bin/env bash

export TEST_NAME="non_existing_user_is_rejected"

TEST_LOG=true cargo test --quiet --release \
$TEST_NAME | grep "HTTP REQUEST" | bunyan