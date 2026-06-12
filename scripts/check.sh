#!/bin/zsh

set -e

echo Checking Debug
cargo check
echo Debug check complete

echo Checking Release
cargo check -r
echo Release check complete
