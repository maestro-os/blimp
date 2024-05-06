#!/bin/sh

# Prepare
mkdir -p toolchain/{repo,usr/bin}
PATH="$(pwd)/toolchain/usr/bin/:$PATH"
TOOLCHAIN="toolchain" source scripts/cross_compile_env.sh
# Build and install packages
for pkg in {binutils,musl,clang}; do
  target/release/blimp-builder "$1/$pkg" "toolchain/repo/$pkg"
  LOCAL_REPO="toolchain/repo/" SYSROOT="toolchain/" target/release/blimp install "$pkg"
done
