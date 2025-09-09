#!/bin/sh

set -e

A="$(cc -dumpmachine)"
if [ -z "$HOST" ]; then
	echo "Missing HOST environment variable"
	exit 1
fi
B="$HOST"
unset BUILD HOST TARGET

# Create base directories
mkdir -pv sysroot/tools sysroot/{etc,var} sysroot/usr/{bin,lib,lib32,sbin}
for i in bin lib sbin; do
  ln -sv "usr/$i" "sysroot/$i"
done
case ${B%%-*} in
	x86_64)
		ln -sv usr/lib sysroot/lib64
		ln -sv lib sysroot/usr/lib64
	;;
esac

# Build packages
# TODO use release mode builder instead?
OLD_PATH="$PATH"
export PATH="$(pwd)/sysroot/tools/bin:$(pwd)/../target/debug:$PATH"
HOST="$A" TARGET="$B" blimp-builder --from desc/binutils1/ --to sysroot/
HOST="$A" TARGET="$B" blimp-builder --from desc/gcc1/ --to sysroot/
blimp-builder --from desc/linux-headers/ --to sysroot/
HOST="$B" blimp-builder --from desc/musl/ --to sysroot/
HOST="$B" blimp-builder --from desc/zlib/ --to sysroot/
HOST="$B" blimp-builder --from desc/libstdc++/ --to sysroot/
HOST="$B" TARGET="$B" blimp-builder --from desc/binutils2/ --to sysroot/
HOST="$B" TARGET="$B" blimp-builder --from desc/gcc2/ --to sysroot/
