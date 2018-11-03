# Test Coverage

## CPU

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

## PPU

  - blargg's `ppu_tests`: (4/5)
    - `palette_ram`: Pass
    - `power_up_palette`: Pass
    - `sprite_ram`: Pass
    - `vbl_clear_time`: Fail
    - `vram_access`: Pass

## APU
