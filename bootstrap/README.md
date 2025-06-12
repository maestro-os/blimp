# Bootstrapping an environment

Bootstrapping is the process of creating an environment which allows the cross compilation of packages.



## Overview

Bootstrapping is done in several steps:

| Package                                     | Host triplet | Target triplet | Notes                                           |
|---------------------------------------------|--------------|----------------|-------------------------------------------------|
| **binutils**                                | A            | B              | binutils stage 1, used to link by gcc stage 1   |
| **gcc** (and **libgcc**)                    | A            | B              | gcc stage 1, used to compile the next step only |
| **linux headers**                           | n/a          | n/a            | required by libc                                |
| **musl**                                    | B            | n/a            | libc, used by gcc stage 2                       |
| **libstdc++**                               | B            | n/a            | used by gcc stage 2, requires libc              |
| **binutils**                                | B            | B              | binutils stage 2                                |
| **gcc** (with **libgcc** and **libstdc++**) | B            | B              | gcc stage 2, used to cross-compile packages     |

The second **gcc** is then able to cross compile other packages (autoconf, make, etc...)

> **Note**: one last compilation of gcc (stage 3) is necessary, but it is treated as a casual package.