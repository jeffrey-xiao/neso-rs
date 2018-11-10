# Test Rom Coverage

## CPU

  - blargg's `branch_timing_tests`: (3/3)
    - `01-branch_basics`: Pass
    - `02-backward_branch`: Pass
    - `03-forward_branch`: Pass
  - blargg's `cpu_dummy_reads`: (0/1)
  - bisqwit's `cpu_dummy_writes`: (0/2)
    - `cpu_dummy_writes_oam`: Fail
    - `cpu_dummy_writes_ppumem`: Fail
  - bisqwit's `cpu_exec_space`: (0/2)
    - `test_cpu_exec_space_apu`: Fail
    - `test_cpu_exec_space_ppuio`: Fail
  - blargg's `cpu_interrupts_v2`: (0/5)
    - `01-cli_latency`: Fail
    - `02-nmi_and_brk`: Fail
    - `03-nmi_and_irq`: Fail
    - `04-irq_and_dma`: Fail
    - `05-branch_delays_irq`: Fail
  - blargg's `cpu_reset`: (2/2)
    - `ram_after_reset`: Pass
    - `registers`: Pass
  - blargg's `cpu_timing_test6`: (1/1)
  - blargg's `instr_misc`: (2/4)
    - `01-abs_x_wrap`: Pass
    - `02-branch_wrap`: Pass
    - `03-dummy_reads`: Fail
    - `04-dummy_reads_apu`: Fail
  - blargg's `instr_test_v5`: (16/16)
    - `01-basics`: Pass
    - `02-implied`: Pass
    - `03-immediate`: Pass
    - `04-zero_page`: Pass
    - `05-zp_xy`: Pass
    - `06-absolute`: Pass
    - `07-abs_xy`: Pass
    - `08-ind_x`: Pass
    - `09-ind_y`: Pass
    - `10-branches`: Pass
    - `11-stack`: Pass
    - `12-jmp_jsr`: Pass
    - `13-rts`: Pass
    - `14-rti`: Pass
    - `15-brk`: Pass
    - `16-special`: Pass
  - blargg's `instr_timing`: (2/2)
    - `01-instr_timing`: Pass
    - `02-branch_timing`: Pass
  - kevtris's `nestest`: (1/1)

## PPU

  - blargg's `ppu_tests`: (4/5)
    - `palette_ram`: Pass
    - `power_up_palette`: Pass
    - `sprite_ram`: Pass
    - `vbl_clear_time`: Fail
    - `vram_access`: Pass

## APU

 - blargg's `apu_mixer` (4/4)
   - `dmc`: Pass
   - `noise`: Pass
   - `square`: Pass
   - `triangle`: Pass
 - Rahsennor's `apu_phase_reset`: (1/1)
 - blargg's `apu_test`: (4/8)
   - `01-len_ctr`: Pass
   - `02-len_table`: Pass
   - `03-irq_flag`: Pass
   - `04-jitter`: Fail
   - `05-len_timing`: Fail
   - `06-irq_flag_timing`: Fail
   - `07-dmc_basics`: Pass
   - `08-dmc_rates`: Fail
