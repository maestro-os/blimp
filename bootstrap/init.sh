#!/bin/sh

set -e

A="$(cc -dumpmachine)"
if [ -z "$HOST" ]; then
	echo "Missing HOST environment variable"
	exit 1
fi
B="$HOST"
unset BUILD HOST TARGET
SYSROOT="${SYSROOT:-sysroot/}"

# Create base directories
mkdir -pv "$SYSROOT/tools" "$SYSROOT"/{etc,var} "$SYSROOT"/usr/{bin,lib,lib32,sbin}
for i in bin lib sbin; do
  ln -fsv "usr/$i" "$SYSROOT/$i"
done
case ${B%%-*} in
	x86_64)
		ln -fsv usr/lib "$SYSROOT/lib64"
		ln -fsv lib "$SYSROOT/usr/lib64"
	;;
esac

# Build packages
# TODO use release mode builder instead?
OLD_PATH="$PATH"
export PATH="$(pwd)/$SYSROOT/tools/bin:$(pwd)/../target/debug:$PATH"
blimp-builder build --from desc/binutils1/ --to "$SYSROOT" --host "$A" --target "$B"
blimp-builder build --from desc/gcc1/ --to "$SYSROOT" --host "$A" --target "$B"
blimp-builder build --from desc/linux-headers/ --to "$SYSROOT"
blimp-builder build --from desc/musl/ --to "$SYSROOT" --host "$B"
blimp-builder build --from desc/zlib/ --to "$SYSROOT" --host "$B"
blimp-builder build --from desc/libstdc++/ --to "$SYSROOT" --host "$B"
blimp-builder build --from desc/binutils2/ --to "$SYSROOT" --host "$B" --target "$B"
blimp-builder build --from desc/gcc2/ --to "$SYSROOT" --host "$B" --target "$B"
