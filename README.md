# norm

Use [ptrace][ptrace-man] to trap [`unlink*`][unlink-man] syscall and do path validation to protect your files from sad unexpected unrecoverable deletion

## Usage

Run `norm -h` to view help of norm part.
Any args after `--` or started from the first non-`-`-prefixed arg are considered child commands and re-executed.

Currently no deletions are allowed

## License

Copyright (c) 2022 myl7

SPDX-License-Identifier: Apache-2.0

[ptrace-man]: https://man7.org/linux/man-pages/man2/ptrace.2.html
[unlink-man]: https://man7.org/linux/man-pages/man2/unlink.2.html
