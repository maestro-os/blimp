# Bootstrapping an environment

Bootstrapping is the process of creating an environment which allows the cross compilation of packages.



## Overview

The following build steps are required for bootstrapping:

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

## Building

First, the `sysroot` directory must be created:
```sh
mkdir sysroot/
```

Then, each package has to be built, in the order of the table above.

The command to use for building a package is:
```sh
PATH="$(pwd)/sysroot/tools:$PATH" HOST=<host-triplet> TARGET=<target-triplet> blimp-builder --from desc/<pkg>/ --to sysroot/
```

Once this is done, the second **gcc** can be used to cross compile packages (autoconf, make, etc...) on the target.
