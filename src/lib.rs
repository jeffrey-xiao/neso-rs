extern crate cfg_if;
extern crate js_sys;
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
use std::cell::RefCell;
use std::rc::Rc;

// TODO: Leave better error messages for panics that should never happen.
pub struct Nes {
    pub apu: Rc<RefCell<Apu>>,
    pub cpu: Rc<RefCell<Cpu>>,
    pub ppu: Rc<RefCell<Ppu>>,
    pub mapper: Option<Rc<RefCell<Box<Mapper>>>>,
}

impl Nes {
    pub fn new() -> Self {
        let apu = Rc::new(RefCell::new(Apu::new()));
        let cpu = Rc::new(RefCell::new(Cpu::new()));
        let ppu = Rc::new(RefCell::new(Ppu::new()));
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
        let mapper = Rc::new(RefCell::new(mapper::from_cartridge(cartridge)));
        let bus = Bus::new(&self.apu, &self.cpu, &self.ppu, &mapper);
        self.apu.borrow_mut().attach_bus(bus.clone());
        self.cpu.borrow_mut().attach_bus(bus.clone());
        self.ppu.borrow_mut().attach_bus(bus.clone());
        mapper.borrow_mut().attach_bus(bus);
        self.mapper = Some(mapper);
    }

    pub fn step(&mut self) {
        self.cpu.borrow_mut().step();
        for _ in 0..3 {
            self.ppu.borrow_mut().step();
            self.mapper.as_ref().unwrap().borrow_mut().step();
        }
        self.apu.borrow_mut().step();
    }

    pub fn step_scanline(&mut self) {
        let scanline = self.ppu.borrow().scanline;
        while self.ppu.borrow().scanline == scanline {
            self.step();
        }
    }

    pub fn step_frame(&mut self) {
        /*
        for val in self.apu.borrow_mut().buffer.iter() {
            print!("{} ", val);
        }
        println!("");
        */
        self.apu.borrow_mut().buffer_index = 0;
        let frame = self.ppu.borrow().frame;
        while self.ppu.borrow().frame == frame {
            self.step();
        }
    }
}

impl Default for Nes {
    fn default() -> Self {
        Nes::new()
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
                    let buffer = fs::read($path).expect("Expected test rom to exist.");
                    let mut nes = Nes::new();
                    nes.load_rom(&buffer);

                    // Run until test status is running by polling $6000.
                    let mut addr = 0x6000;
                    let mut byte = nes.cpu.borrow_mut().read_byte(addr);
                    while byte != 0x80 {
                        nes.step();
                        byte = nes.cpu.borrow_mut().read_byte(addr);
                    }

                    // Run until test status is finished by polling $6000.
                    byte = nes.cpu.borrow_mut().read_byte(addr);
                    while byte == 0x80 {
                        nes.step();
                        byte = nes.cpu.borrow_mut().read_byte(addr);
                    }

                    // Read output at $6004.
                    let mut output = Vec::new();
                    addr = 0x6004;
                    byte = nes.cpu.borrow_mut().read_byte(addr);
                    while byte != '\0' as u8 {
                        output.push(byte);
                        addr += 1;
                        byte = nes.cpu.borrow_mut().read_byte(addr);
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
                let buffer = fs::read($path).expect("Expected test rom to exist.");
                let mut nes = Nes::new();
                nes.load_rom(&buffer);

                for i in 0..$frames {
                    nes.step_frame();
                }

                let mut hasher = DefaultHasher::new();

                for addr in 0x2000..0x3000 {
                    hasher.write_u8(nes.ppu.borrow().read_byte(addr));
                }

                assert_eq!(hasher.finish(), $hash);
            }
        )*
    }
}

    #[cfg(test)]
    mod cpu {
        #[cfg(test)]
        mod instr_tests {
            use std::fs;
            use Nes;

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

        #[cfg(test)]
        mod instr_misc {
            use std::fs;
            use Nes;

            fn test_path(file_name: &str) -> String {
                format!("./tests/cpu/instr_misc/{}", file_name)
            }

            text_tests!(
                test_01_abs_x_wrap: test_path("01-abs_x_wrap.nes"),
                test_02_branch_wrap: test_path("02-branch_wrap.nes"),
            );
        }

        #[cfg(test)]
        mod instr_timing {
            use std::collections::hash_map::DefaultHasher;
            use std::fs;
            use std::hash::Hasher;
            use Nes;

            fn test_path(file_name: &str) -> String {
                format!("./tests/cpu/instr_timing/{}", file_name)
            }

            graphical_tests!(
                test_cpu_timing_test: (test_path("cpu_timing_test.nes"), 612, 0x884F_461F_A12C_F454),
            );
        }

        #[cfg(test)]
        mod branch_timing {
            use std::collections::hash_map::DefaultHasher;
            use std::fs;
            use std::hash::Hasher;
            use Nes;

            fn test_path(file_name: &str) -> String {
                format!("./tests/cpu/branch_timing/{}", file_name)
            }

            graphical_tests!(
                test_02_backward_branch: (test_path("02-backward_branch.nes"), 15, 0x1B68_8FBE_CDE5_8EA7),
                test_03_forward_branch: (test_path("03-forward_branch.nes"), 15, 0x4275_CE0C_E944_E57C),
            );
        }
    }
}
