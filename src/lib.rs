#![feature(nll)]

extern crate cfg_if;
extern crate wasm_bindgen;

mod apu;
mod bus;
mod cartridge;
mod controller;
mod cpu;
mod mapper;
mod ppu;
mod utils;

use apu::Apu;
use bus::Bus;
use cartridge::Cartridge;
use cpu::Cpu;
use mapper::Mapper;
use ppu::Ppu;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Nes {
    apu: Apu,
    cpu: Cpu,
    ppu: Ppu,
    mapper: Option<*mut Mapper>,
}

#[wasm_bindgen]
impl Nes {
    pub fn new() -> Self {
        utils::set_panic_hook();
        let apu = Apu::new();
        let cpu = Cpu::new();
        let ppu = Ppu::new();
        let mapper = None;

        Nes {
            apu,
            cpu,
            ppu,
            mapper,
        }
    }

    pub fn load_rom(&mut self, buffer: &[u8]) {
        let cartridge = Cartridge::from_buffer(buffer);
        let mapper = mapper::from_cartridge(cartridge);
        let bus = Bus::new(&mut self.apu, &mut self.cpu, &mut self.ppu, mapper);
        self.apu.attach_bus(bus.clone());
        self.cpu.attach_bus(bus.clone());
        self.ppu.attach_bus(bus.clone());
        bus.mapper().attach_bus(bus.clone());
        self.mapper = Some(bus.mapper());
    }

    pub fn step(&mut self) {
        self.cpu.step();
        let mapper = unsafe { &mut (*self.mapper.expect("[NES] No ROM loaded.")) };
        for _ in 0..3 {
            self.ppu.step();
            mapper.step();
        }
        self.apu.step();
    }

    pub fn step_frame(&mut self) {
        self.apu.buffer_index = 0;
        let frame = self.ppu.frame;
        while self.ppu.frame == frame {
            self.step();
        }
    }

    pub fn image_buffer(&self) -> *const u8 {
        self.ppu.buffer.as_ptr()
    }

    pub fn audio_buffer(&self) -> *const f32 {
        self.apu.buffer.as_ptr()
    }

    pub fn audio_buffer_len(&self) -> usize {
        self.apu.buffer_index
    }

    pub fn chr_bank(&self, index: usize) -> *const u8 {
        assert!(index < 8);
        let mapper = unsafe { &mut (*self.mapper.expect("[NES] No ROM loaded.")) };
        mapper.chr_bank(index)
    }

    pub fn nametable_bank(&self, index: usize) -> *const u8 {
        assert!(index < 8);
        self.ppu.nametable_bank(index)
    }

    pub fn background_chr_bank(&self) -> usize {
        if self.ppu.r.background_pattern_table_address == 0x1000 {
            4
        } else {
            0
        }
    }

    pub fn press_button(&mut self, controller_index: usize, button_index: u8) {
        self.cpu.controllers[controller_index].press_button(button_index);
    }

    pub fn release_button(&mut self, controller_index: usize, button_index: u8) {
        self.cpu.controllers[controller_index].release_button(button_index);
    }
}

impl Default for Nes {
    fn default() -> Self {
        Nes::new()
    }
}

impl Drop for Nes {
    fn drop(&mut self) {
        if let Some(mapper) = self.mapper {
            unsafe {
                Box::from_raw(mapper);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Test output is at $6004.
    macro_rules! text_tests {
        ($($test_name:ident: $path:expr$(,)*)*) => {
            $(
                #[test]
                fn $test_name() {
                    use std::fs;
                    use Nes;

                    let buffer = fs::read($path).expect("Expected test rom to exist.");
                    let mut nes = Nes::new();
                    nes.load_rom(&buffer);

                    // Run until test status is running by polling $6000.
                    let mut addr = 0x6000;
                    let mut byte = nes.cpu.read_byte(addr);
                    while byte != 0x80 {
                        nes.step_frame();
                        byte = nes.cpu.read_byte(addr);
                    }

                    // Run until test status is finished by polling $6000.
                    byte = nes.cpu.read_byte(addr);
                    while byte == 0x80 {
                        nes.step_frame();
                        byte = nes.cpu.read_byte(addr);
                    }

                    // Read output at $6004.
                    let mut output = Vec::new();
                    addr = 0x6004;
                    byte = nes.cpu.read_byte(addr);
                    while byte != '\0' as u8 {
                        output.push(byte);
                        addr += 1;
                        byte = nes.cpu.read_byte(addr);
                    }

                    assert!(String::from_utf8_lossy(&output).contains("Passed"));
                }
            )*
        }
    }

    // Compare hash of nametables after specified frames for no text output tests.
    macro_rules! graphical_tests {
        ($($test_name:ident: ($path:expr, $frames:expr, $hash:expr)$(,)*)*) => {
            $(
                #[test]
                fn $test_name() {
                    use std::collections::hash_map::DefaultHasher;
                    use std::fs;
                    use std::hash::Hasher;
                    use Nes;

                    let buffer = fs::read($path).expect("Expected test rom to exist.");
                    let mut nes = Nes::new();
                    nes.load_rom(&buffer);

                    for _ in 0..$frames {
                        nes.step_frame();
                    }

                    let mut hasher = DefaultHasher::new();

                    for val in nes.ppu.buffer.iter() {
                        hasher.write_u8(*val);
                    }

                    assert_eq!(hasher.finish(), $hash);
                }
            )*
        }
    }

    mod cpu {
        mod instr_tests {
            fn test_path(file_name: &str) -> String {
                format!("./tests/cpu/instr_test/{}", file_name)
            }

            text_tests!(
                test_01_basics: test_path("01-basics.nes"),
                test_02_implied: test_path("02-implied.nes"),
                test_03_immediate: test_path("03-immediate.nes"),
                test_04_zero_page: test_path("04-zero_page.nes"),
                test_05_zp_xy: test_path("05-zp_xy.nes"),
                test_06_absolute: test_path("06-absolute.nes"),
                test_07_abs_xy: test_path("07-abs_xy.nes"),
                test_08_ind_x: test_path("08-ind_x.nes"),
                test_09_ind_y: test_path("09-ind_y.nes"),
                test_10_branches: test_path("10-branches.nes"),
                test_11_stack: test_path("11-stack.nes"),
                test_12_jmp_jsr: test_path("12-jmp_jsr.nes"),
                test_13_rts: test_path("13-rts.nes"),
            );
        }

        mod instr_misc {
            fn test_path(file_name: &str) -> String {
                format!("./tests/cpu/instr_misc/{}", file_name)
            }

            text_tests!(
                test_01_abs_x_wrap: test_path("01-abs_x_wrap.nes"),
                test_02_branch_wrap: test_path("02-branch_wrap.nes"),
            );
        }

        mod instr_timing {
            fn test_path(file_name: &str) -> String {
                format!("./tests/cpu/instr_timing/{}", file_name)
            }

            text_tests!(
                test_01_instr_timing: test_path("01-instr_timing.nes"),
                test_02_branch_timing: test_path("02-branch_timing.nes"),
            );

            graphical_tests!(
                test_cpu_timing_test: (test_path("timing_test.nes"), 612, 0x2F89_29CE_711F_FBD4),
            );
        }

        mod branch_timing {
            fn test_path(file_name: &str) -> String {
                format!("./tests/cpu/branch_timing/{}", file_name)
            }

            graphical_tests!(
                test_01_branch_basics: (test_path("01-branch_basics.nes"), 13, 0xDB8E_7124_029B_C022),
                test_02_backward_branch: (test_path("02-backward_branch.nes"), 15, 0xDF84_2558_1C2B_C9A7),
                test_03_forward_branch: (test_path("03-forward_branch.nes"), 15, 0x528E_9396_828A_8125),
            );
        }
    }

    mod ppu {
        mod general {
            fn test_path(file_name: &str) -> String {
                format!("./tests/ppu/general/{}", file_name)
            }

            graphical_tests!(
                test_palette_ram: (test_path("palette_ram.nes"), 18, 0x657D_9167_290E_F938),
                test_power_up_palette: (test_path("power_up_palette.nes"), 18, 0x657D_9167_290E_F938),
                test_sprite_ram: (test_path("sprite_ram.nes"), 18, 0x657D_9167_290E_F938),
                test_vram_access: (test_path("vram_access.nes"), 18, 0x657D_9167_290E_F938),
            );
        }
    }

    mod apu {
        mod general {
            fn test_path(file_name: &str) -> String {
                format!("./tests/apu/general/{}", file_name)
            }

            text_tests!(
                test_01_len_ctr: test_path("01-len_ctr.nes"),
                test_07_dmc_basics: test_path("07-dmc_basics.nes"),
            );
        }
    }
}
