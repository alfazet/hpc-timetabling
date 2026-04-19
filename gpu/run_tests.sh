#!/usr/bin/env bash

if [[ -z "$1" ]]; then
  echo "target must be non-empty"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BUILD_DIR="$SCRIPT_DIR/build"
TEST_TARGET="$1_tests"

cmake -S "$SCRIPT_DIR" -B "$BUILD_DIR" -DCMAKE_BUILD_TYPE=Debug || exit 1
cmake --build "$BUILD_DIR" --target ${TEST_TARGET} -j"$(nproc)" || exit 1

cd "$BUILD_DIR"
ctest --output-on-failure -L "^$1$"
