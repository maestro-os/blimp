#!/bin/sh

set -e

BUILD="$(cc -dumpmachine)"
if [ -z "$HOST" ]; then
	echo "Missing HOST environment variable"
	exit 1
fi

# Create base directories
mkdir -pv sysroot/tools sysroot/{etc,var} sysroot/usr/{bin,lib,lib32,sbin}
for i in bin lib sbin; do
  ln -sv "usr/$i" "sysroot/$i"
done
case ${HOST%%-*} in
	x86_64)
		ln -sv usr/lib sysroot/lib64
		ln -sv lib sysroot/usr/lib64
	;;
esac

# Build packages
# TODO use release mode builder instead?
OLD_PATH="$PATH"
export PATH="$(pwd)/sysroot/tools/bin:$(pwd)/../target/debug:$PATH"
HOST="$BUILD" TARGET="$HOST" blimp-builder --from desc/binutils1/ --to sysroot/
HOST="$BUILD" TARGET="$HOST" blimp-builder --from desc/gcc1/ --to sysroot/
PATH="$OLD_PATH" ../target/debug/blimp-builder --from desc/linux-headers/ --to sysroot/
HOST="$HOST" blimp-builder --from desc/musl/ --to sysroot/
HOST="$HOST" blimp-builder --from desc/zlib/ --to sysroot/
HOST="$HOST" blimp-builder --from desc/libstdc++/ --to sysroot/
HOST="$HOST" TARGET="$HOST" blimp-builder --from desc/binutils2/ --to sysroot/
HOST="$HOST" TARGET="$HOST" blimp-builder --from desc/gcc2/ --to sysroot/
