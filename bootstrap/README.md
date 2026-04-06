# Bootstrapping an environment

Bootstrapping is the process of creating an environment which allows the cross-compilation of packages.

`./init.sh` builds a cross-compilation toolchain in `sysroot/` by default. The path to the sysroot can be changed by setting the `SYSROOT` environment variable.

> **Note**: one should not build several toolchains for different targets in the same sysroot

The following packages are built by `init.sh`:

| Package                                     | Host triplet | Target triplet | Notes                                           |
|---------------------------------------------|--------------|----------------|-------------------------------------------------|
| **binutils**                                | A            | B              | binutils stage 1, used to link by gcc stage 1   |
| **gcc** (and **libgcc**)                    | A            | B              | gcc stage 1, used to compile the next step only |
| **linux headers**                           | n/a          | n/a            | required by libc                                |
| **musl**                                    | B            | n/a            | libc, used by gcc stage 2                       |
| **zlib**                                    | B            | n/a            | zlib, used by binutils stage 2 while running    |
| **libstdc++**                               | B            | n/a            | used by gcc stage 2, requires libc              |
| **binutils**                                | B            | B              | binutils stage 2                                |
| **gcc** (with **libgcc** and **libstdc++**) | B            | B              | gcc stage 2, used to cross-compile packages     |

> **Note**: one last compilation of gcc (stage 3) will be necessary for a final system, but it is treated as a casual package and not discussed here.

Once built, the second **gcc** can be used to cross-compile packages on the target.

## Using the toolchain

A toolchain can be used by updating `PATH`:

```sh
PATH="$(pwd)/sysroot/tools/bin:$PATH"
```

The above command assumes we are in the `boostrap/` directory.