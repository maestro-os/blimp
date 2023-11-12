# This file contains an environment to cross-compile packages to the i686 architecture
# The script requires the TOOLCHAIN variable to be set to the toolchain's path

# If the target is not specified, set to default
if [ -z $TARGET ]; then
  export TARGET=i686-unknown-linux-musl
fi

export CC="clang"
export CFLAGS="-target $TARGET --sysroot $TOOLCHAIN -I$TOOLCHAIN/usr/include -I$TOOLCHAIN/include"

export LD=ld.lld
export LDFLAGS="-target $TARGET --sysroot $TOOLCHAIN -L$TOOLCHAIN/usr/lib -L$TOOLCHAIN/lib -fuse-ld=lld --rtlib=compiler-rt"

export RUSTFLAGS="-L$TOOLCHAIN/usr/lib -L$TOOLCHAIN/lib -Clink-arg=-fuse-ld=lld -Clinker=clang -Clink-arg=--rtlib=compiler-rt"
