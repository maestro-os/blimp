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
blimp-builder build --from desc/binutils1/ --to sysroot/ --host "$A" --target "$B"
blimp-builder build --from desc/gcc1/ --to sysroot/ --host "$A" --target "$B"
blimp-builder build --from desc/linux-headers/ --to sysroot/
blimp-builder build --from desc/musl/ --to sysroot/ --host "$B"
blimp-builder build --from desc/zlib/ --to sysroot/ --host "$B"
blimp-builder build --from desc/libstdc++/ --to sysroot/ --host "$B"
blimp-builder build --from desc/binutils2/ --to sysroot/ --host "$B" --target "$B"
blimp-builder build --from desc/gcc2/ --to sysroot/ --host "$B" --target "$B"
