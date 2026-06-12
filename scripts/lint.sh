#!/bin/bash

set -e

./scripts/format.sh
./scripts/clippy.sh
./scripts/check.sh
