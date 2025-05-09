#!/usr/bin/env bash
command=$1

# If command is empty
if [ -z "$command" ]
then
    cargo watch -x check -x test -x run
elif [ "$command" == "coverage" ]
then
    cargo tarpaulin --ignore-tests
elif [ "$command" == "test" ]
then
    if ! [ -x "$(command -v bunyan)" ]; then
        echo >&2 "Error: bunyan is not installed. Use: cargo install bunyan"
        exit 1
    fi
    TEST_LOG=true cargo test $2 | bunyan
elif [ "$command" == "fmt" ]
then
    cargo fmt
fi