# Cross compilation

This directory contains utilities to build a toolchain to be used to cross-compile packages.

Build the toolchain into the `toolchain` directory using:
```sh
./build.sh
```

Then, use the following command to set up the environment to compile using the toolchain.
```sh
TARGET="<target-triplet>" source ./env.sh
```

The `TARGET` environment variable is used by `env.sh` to determine the target to compile for.
