#!/bin/sh

set -e

# If the target is not specified, set to default
if [ -z "$TARGET" ]; then
  export TARGET=x86_64-unknown-linux-musl
fi
if [ -z "$TOOLCHAIN" ]; then
	export TOOLCHAIN="toolchain/"
fi
if [ -z "$LOCAL_REPO" ]; then
	export LOCAL_REPO="$TOOLCHAIN/repo/"
fi

# Prepare
mkdir -p "$TOOLCHAIN" "$LOCAL_REPO"
TOOLCHAIN="$(realpath "$TOOLCHAIN")"
export SYSROOT="$TOOLCHAIN"
LOCAL_REPO="$(realpath "$LOCAL_REPO")"
PATH="$(pwd)/../target/release:$PATH"

## binutils
blimp-builder desc/binutils "$LOCAL_REPO"
yes | blimp install binutils

# musl
source ./env.sh
HOST="$TARGET" blimp-builder desc/musl "$LOCAL_REPO"
unset CC CXX LD CFLAGS CXXFLAGS LDFLAGS LIBCC RUSTFLAGS
yes | blimp install musl

# clang
blimp-builder desc/clang "$LOCAL_REPO"
yes | blimp install clang
