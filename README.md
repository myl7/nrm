# nrm

Use [ptrace][ptrace-man] to trap [`unlink*`][unlink-man] syscall and do path validation to protect your files from sad unexpected unrecoverable deletion

## Usage

Run `nrm -h` to view help of nrm part.
Any args after `--` or started from the first non-`-`-prefixed arg are considered child commands and re-executed.

Currently no deletions are allowed

## Installation

Install from crates.io: `cargo install nrm`

Build from source: `cargo build --release`

Features:

- `bigendian`: In case your device is (the rare) big endian

## License

Copyright (c) 2022 myl7

SPDX-License-Identifier: Apache-2.0

[ptrace-man]: https://man7.org/linux/man-pages/man2/ptrace.2.html
[unlink-man]: https://man7.org/linux/man-pages/man2/unlink.2.html
