#!/bin/zsh

set -e

echo Running Clippy on debug artifact
cargo clippy
echo Clippy run on debug artifact complete

echo Running Clippy on release artifact
cargo clippy -r
echo Clippy run on release artifact complete
