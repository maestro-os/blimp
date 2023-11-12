<p align="center">
  <picture>
    <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/llenotre/maestro-lnf/master/logo-light.svg">
    <img src="https://raw.githubusercontent.com/llenotre/maestro-lnf/master/logo.svg" alt="logo" width="50%" />
  </picture>
</p>

[![MIT license](https://img.shields.io/badge/license-MIT-blue.svg?style=for-the-badge&logo=book)](./LICENSE)
![Version](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2Fllenotre%2Fblimp%2Fmaster%2Fclient%2FCargo.toml&query=%24.package.version&style=for-the-badge&label=version)
![Continuous integration](https://img.shields.io/github/actions/workflow/status/llenotre/blimp/check.yml?style=for-the-badge&logo=github)

# About

Blimp is a simple package manager for Unix-like operating systems, more specifically for [Maestro](https://github.com/llenotre/maestro).

This repository contains the following components:
- `blimp`: The package manager itself
- `blimp-builder`: An utility to build packages
- `blimp-server`: The package manager's server

The `common` crate is a library with utilities shared across components.



# Build

Build the package manager using:

```sh
cargo build           # Debug mode
cargo build --release # Release mode
```

Building with network support required the `network` feature:

```sh
cargo build --features network           # Debug mode
cargo build --features network --release # Release mode
```



# Usage

## Blimp

Synchronize packages information with remotes:

```sh
blimp update
```

Install package(s):

```sh
blimp install <package>
```

Upgrade packages:

```sh
blimp upgrade
```

Remove package(s):

```sh
blimp remove <package>
```

Show the whole usage of the command:

```sh
blimp
```



## Blimp-builder

The general usage of the command is:

```sh
blimp-builder <package descriptor> <output repository>
```

The command builds the package according to the descriptor, then writes the result in the given output repository.

TODO: description of the structure of package descriptors and output packages



### Cross compilation

Cross compilation is required when building package for a system with a different target triplet than the current system.

To cross compile, you first need to build a [toolchain](TODO).

Then, run the following command to set environment variables for your current shell:

```sh
TOOLCHAIN=<path to the toolchain> source cross_compile_env.sh
```



# Blimp-server

TODO
