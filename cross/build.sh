#!/bin/sh

set -e

# If the target is not specified, set to default
if [ -z "$TARGET" ]; then
  export TARGET=x86_64-unknown-linux-musl
fi
if [ -z "$SYSROOT" ]; then
	export SYSROOT="toolchain/"
fi
if [ -z "$LOCAL_REPO" ]; then
	export LOCAL_REPO="$SYSROOT/repo/"
fi

# Prepare
mkdir -p "$SYSROOT" "$LOCAL_REPO"
SYSROOT="$(realpath "$SYSROOT")"
LOCAL_REPO="$(realpath "$LOCAL_REPO")"
PATH="$(pwd)/../target/release:$PATH"

# binutils
blimp-builder desc/binutils "$LOCAL_REPO"
yes | blimp install binutils

# musl
TOOLCHAIN="$SYSROOT" source ./env.sh
HOST="$TARGET" blimp-builder desc/musl "$LOCAL_REPO"
unset CC LD CFLAGS LDFLAGS RUSTFLAGS
yes | blimp install musl

# clang
blimp-builder desc/clang "$LOCAL_REPO"
yes | blimp install clang
