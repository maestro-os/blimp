# This file contains an environment to cross-compile packages to the i686 architecture
# The script requires the TOOLCHAIN variable to be set to the toolchain's path

export TARGET=i686-unknown-linux-musl

export CC=clang
export CFLAGS="-target i686-unknown-linux-musl --sysroot $TOOLCHAIN"

export LD=ld.lld
export LDFLAGS="-target i686-unknown-linux-musl --sysroot $TOOLCHAIN -L$TOOLCHAIN/usr/lib -L$TOOLCHAIN/lib -fuse-ld=lld --rtlib=compiler-rt"

export RUSTFLAGS="-L$TOOLCHAIN/usr/lib -L$TOOLCHAIN/lib -Clink-arg=-fuse-ld=lld -Clinker=clang -Clink-arg=--rtlib=compiler-rt"
