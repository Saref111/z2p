#!/usr/bin/env bash
command=$1

# If command is empty
if [ -z "$command" ]
then
    cargo watch -x check -x test -x run
elif [ "$command" == "coverage" ]
then
    cargo tarpaulin --ignore-tests
elif [ "$command" == "fmt" ]
then
    cargo fmt
fi