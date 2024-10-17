# Usage: `source env.sh`
# `TARGET` is the target triplet for the compiler

TOOLCHAIN="$(pwd)/toolchain"

export PATH="$TOOLCHAIN/usr/bin:$TOOLCHAIN/bin:$PATH"

export CC="clang"
export LD="ld.lld"

export CFLAGS="-target $TARGET --sysroot $TOOLCHAIN --ld-path=$LD --rtlib=compiler-rt -Wno-unused-command-line-argument -static"
export LDFLAGS="-target $TARGET --sysroot $TOOLCHAIN --ld-path=$LD --rtlib=compiler-rt -Wno-unused-command-line-argument -static"

export RUSTFLAGS="-L$TOOLCHAIN/usr/lib -L$TOOLCHAIN/lib -Clinker=$LD"
