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

## Test Rom Progress

CPU
  - kevtris's `nestest`: Pass
  - blargg's `instr_misc`: (2/4)
    - `01-abs_x_wrap.nes`: Pass
    - `02-branch_wrap.nes`: Pass
    - `03-dummy_reads.nes`: Fail
    - `04-dummy_reads_apu.nes`: Fail
  - blargg's `instr_test`: (16/16)
    - `01-basics.nes`: Pass
    - `02-implied.nes`: Pass
    - `03-immediate.nes`: Pass
    - `04-zero_page.nes`: Pass
    - `05-zp_xy.nes`: Pass
    - `06-absolute.nes`: Pass
    - `07-abs_xy.nes`: Pass
    - `08-ind_x.nes`: Pass
    - `09-ind_y.nes`: Pass
    - `10-branches.nes`: Pass
    - `11-stack.nes`: Pass
    - `12-jmp_jsr.nes`: Pass
    - `13-rts.nes`: Pass
    - `14-rti.nes`: Pass
    - `15-brk.nes`: Pass
    - `16-special.nes`: Pass
  - blargg's `instr_timing`: (0/2)
    - `01-instr_timing.nes`: Fail
    - `02-branch_timing.nes`: Fail
  - blargg's `interrupts`: (0/5)
    - `01-cli_latency`: Fail
    - `02-nmi_and_brk`: Fail
    - `03-nmi_and_irq`: Fail
    - `04-irq_and_dma`: Fail
    - `05-branch_delays_irq`: Fail
  - blargg's `branch_timing_tests`: (2/3)
    - `01-branch_basics.nes`: Fail
    - `02-backward_branch.nes`: Pass
    - `03-forward_branch.nes`: Pass
  - blargg's `cpu_timing_test`: Pass

PPU

APU

## References

 - [NESDev Wiki](https://wiki.nesdev.com)
 - [Obelisk 6502 Reference](http://www.obelisk.me.uk/6502/reference.html)

## License

`nes-rs` is distributed under the terms of both the MIT License and the Apache License (Version
2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for more details.
