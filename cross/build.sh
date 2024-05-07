#!/bin/sh

set -e

# If the target is not specified, set to default
if [ -z $TARGET ]; then
  export TARGET=i686-unknown-linux-musl
fi

# Prepare
mkdir -p toolchain/repo
PATH="$(pwd)/../target/release:$PATH"
export SYSROOT="$(pwd)/toolchain/"
export LOCAL_REPO="$(pwd)/toolchain/repo/"

# binutils
blimp-builder desc/binutils toolchain/repo/
yes | blimp install binutils

# musl
source ./env.sh
HOST="$TARGET" blimp-builder desc/musl toolchain/repo/
unset CC LD CFLAGS LDFLAGS RUSTFLAGS
yes | blimp install musl

# TODO clang