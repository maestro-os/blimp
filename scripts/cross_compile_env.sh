# This file contains an environment to cross-compile packages to the i686 architecture
# The script requires the TOOLCHAIN variable to be set to the toolchain's path

# If the target is not specified, set to default
if [ -z $TARGET ]; then
  export TARGET=i686-unknown-linux-musl
fi

export CC="clang"
export LD="ld.lld"

export CFLAGS="-target $TARGET --sysroot $TOOLCHAIN --ld-path=$LD --rtlib=compiler-rt -Wno-unused-command-line-argument -static"
export LDFLAGS="-target $TARGET --sysroot $TOOLCHAIN --ld-path=$LD --rtlib=compiler-rt -Wno-unused-command-line-argument -static"

export RUSTFLAGS="-L$TOOLCHAIN/usr/lib -L$TOOLCHAIN/lib -Clinker=$LD"
