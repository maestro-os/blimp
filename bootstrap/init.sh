#!/bin/sh

set -e

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