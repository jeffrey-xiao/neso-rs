# neso-rs

[![NPM version](https://img.shields.io/npm/v/neso.svg?style=flat)](https://www.npmjs.com/package/neso)
[![neso](http://meritbadge.herokuapp.com/neso)](https://crates.io/crates/neso)
[![Documentation](https://docs.rs/neso/badge.svg)](https://docs.rs/neso)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Build Status](https://travis-ci.org/jeffrey-xiao/neso-rs.svg?branch=master)](https://travis-ci.org/jeffrey-xiao/neso-rs)
[![codecov](https://codecov.io/gh/jeffrey-xiao/neso-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/jeffrey-xiao/neso-rs)

NES Oxidized (NESO) is a Nintendo Entertainment System emulator written in Rust that can compile to
WebAssembly.

## JavaScript Usage

Install `neso` from [npm](https://www.npmjs.com/):
```
$ npm install neso
```

Example JavaScript usage: [`neso-web`](https://gitlab.com/jeffrey-xiao/neso-web).

## Rust Usage

Add this to your `Cargo.toml`:
```toml
[dependencies]
neso = "*"
```
and this to your crate root if you are using Rust 2015:
```rust
extern crate neso;
```

Example Rust usage: [`neso-gui`](https://gitlab.com/jeffrey-xiao/neso-gui).

## Features

 - Instruction-cycle accurate MOS 6502 CPU with unofficial instructions.
 - Mostly cycle accurate PPU.
 - Mostly accurate APU.

## Compatibility

The following mappers are implemented:
 - `000`: [NROM](http://bootgod.dyndns.org:7777/search.php?ines=0)
 - `001`: [MMC1](http://bootgod.dyndns.org:7777/search.php?ines=1)
 - `002`: [UNROM](http://bootgod.dyndns.org:7777/search.php?ines=2)
 - `003`: [CNROM](http://bootgod.dyndns.org:7777/search.php?ines=3)
 - `004`: [MMC3](http://bootgod.dyndns.org:7777/search.php?ines=4)
 - `007`: [AxROM](http://bootgod.dyndns.org:7777/search.php?ines=7)
 - `011`: [ColorDreams](http://bootgod.dyndns.org:7777/search.php?ines=11)
 - `094`: [UN1ROM](http://bootgod.dyndns.org:7777/search.php?ines=94)
 - `180`: [_Crazy Climber_](http://bootgod.dyndns.org:7777/search.php?ines=180)

These mappers provide support for approximately 89% (1417/1591) games listed in this
[comprehensive mapper list](http://tuxnes.sourceforge.net/nesmapper.txt).

## Test Rom Coverage

See [TEST_ROM_COVERAGE](TEST_ROM_COVERAGE.md) for more details.

## Changelog

See [CHANGELOG](CHANGELOG.md) for more details.

## References

 - [NESDev Wiki](https://wiki.nesdev.com)
 - [Obelisk 6502 Reference](http://www.obelisk.me.uk/6502/reference.html)

## License

`neso-rs` is distributed under the terms of both the MIT License and the Apache License (Version
2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for more details.
