# Usage: `source env.sh`
# `TOOLCHAIN` is the path to the toolchain's root
# `TARGET` is the target triplet for the compiler

if [ -z $TOOLCHAIN ]; then
	TOOLCHAIN="toolchain/"
fi
TOOLCHAIN="$(realpath "$TOOLCHAIN")"

export PATH="$TOOLCHAIN/usr/bin:$TOOLCHAIN/bin:$PATH"

export CC="clang"
export LD="ld.lld"

export CFLAGS="-target $TARGET --sysroot $TOOLCHAIN --ld-path=$LD --rtlib=compiler-rt -Wno-unused-command-line-argument -Wno-ignored-optimization-argument -static"
export LDFLAGS="-target $TARGET --sysroot $TOOLCHAIN --ld-path=$LD --rtlib=compiler-rt -Wno-unused-command-line-argument -Wno-ignored-optimization-argument -static"

export RUSTFLAGS="-L$TOOLCHAIN/usr/lib -L$TOOLCHAIN/lib -Clinker=$LD"
