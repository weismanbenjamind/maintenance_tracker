#!/bin/zsh

FAILED=0

echo Running tests
cargo test || FAILED=1
echo Done running tests

echo Formatting
cargo +nightly fmt --check || FAILED=1
echo Formatting complete

echo Checking debug artifact
cargo check || FAILED=1
echo Debug check complete

echo Checking release artifact
cargo check -r || FAILED=1
echo Release check complete

echo Running Clippy on debug artifact
cargo clippy || FAILED=1
echo Clippy run on debug artifact complete

echo Running Clippy on release artifact
cargo clippy -r || FAILED=1
echo Clippy run on release artifact complete
