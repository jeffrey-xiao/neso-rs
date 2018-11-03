# nes-rs

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Build Status](https://travis-ci.org/jeffrey-xiao/nes-rs.svg?branch=master)](https://travis-ci.org/jeffrey-xiao/nes-rs)
[![codecov](https://codecov.io/gh/jeffrey-xiao/nes-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/jeffrey-xiao/nes-rs)

An NES emulator written in Rust that compiles to WebAssembly.

## Features

 - Instruction-cycle accurate MOS 6502 CPU with unofficial instructions.
 - Mostly cycle accurate PPU.

## Compatibility

The following mappers are implemented:
 - `000`: NROM
 - `001`: MMC1
 - `002`: UNROM
 - `003`: CNROM
 - `004`: MMC3
 - `007`: AxROM
 - `011`: ColorDreams
 - `094`: UN1ROM
 - `180`: _Crazy Climber_

## Test Coverage

See [TEST_COVERAGE](TEST_COVERAGE.md) for more details.

## References

 - [NESDev Wiki](https://wiki.nesdev.com)
 - [Obelisk 6502 Reference](http://www.obelisk.me.uk/6502/reference.html)

## License

`nes-rs` is distributed under the terms of both the MIT License and the Apache License (Version
2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for more details.
