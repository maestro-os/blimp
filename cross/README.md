# Cross compilation

This directory contains utilities to build a toolchain to be used to cross-compile packages.


## Build the toolchain

First make sure `blimp-builder` has been built. Go to its directory and use:

```sh
cargo build --release --features network
```

Then, you can specify the target triplet for which the toolchain compiles with the `TARGET` environment variable. Example:

```sh
export TARGET="i686-unknown-linux-musl"
```

Then, in this directory use `build.sh` to build the toolchain:

```sh
./build.sh
```

This may take a while. When done, the toolchain is available in the `toolchain/` directory.



## Use the toolchain

The environment to use the toolchain can be set using:

```sh
TARGET="<target-triplet>" source ./env.sh
```

The `TARGET` environment variable is used by `env.sh` to determine the target to compile for.

Then, `blimp-builder` can be used to cross compile packages.
