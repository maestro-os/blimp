#!/bin/sh

set -e

# Prepare
mkdir -p toolchain/{repo,usr/bin}
PATH="$(pwd)/toolchain/usr/bin/:$PATH"
export SYSROOT="$(pwd)/toolchain/"
export LOCAL_REPO="$(pwd)/toolchain/repo/"
# If the target is not specified, set to default
if [ -z $TARGET ]; then
  export TARGET=i686-unknown-linux-musl
fi

# binutils
target/release/blimp-builder "$1/binutils" toolchain/repo/
target/release/blimp install binutils

# musl
TOOLCHAIN="toolchain" source scripts/cross_compile_env.sh
HOST="$TARGET" target/release/blimp-builder "$1/musl" toolchain/repo/
unset CC LD CFLAGS LDFLAGS RUSTFLAGS
target/release/blimp install musl

# TODO clang