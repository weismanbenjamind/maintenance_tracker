#!/bin/zsh

set -e

echo Building debug artifact
cargo build
echo Built debug artifact

echo Building release artifact
cargo build -r
echo Built release artifact
