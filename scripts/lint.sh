#!/bin/bash

set -e

./scripts/format_nightly.sh
./scripts/clippy.sh
./scripts/check.sh
