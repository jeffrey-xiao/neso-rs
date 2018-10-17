extern crate cfg_if;
extern crate js_sys;
extern crate wasm_bindgen;

mod bus;
mod cartridge;
mod controller;
pub mod cpu;
mod mapper;
mod ppu;
mod utils;

use bus::Bus;
use cartridge::Cartridge;
use cpu::Cpu;
use mapper::Mapper;
use ppu::Ppu;
use std::cell::RefCell;
use std::rc::Rc;

// TODO: Leave better error messages for panics that should never happen.
pub struct Nes {
    pub cpu: Rc<RefCell<Cpu>>,
    pub ppu: Rc<RefCell<Ppu>>,
    pub mapper: Option<Rc<RefCell<Box<Mapper>>>>,
}

impl Nes {
    pub fn new() -> Self {
        let cpu = Rc::new(RefCell::new(Cpu::new()));
        let ppu = Rc::new(RefCell::new(Ppu::new()));
        let mapper = None;

        Nes { cpu, ppu, mapper }
    }

    pub fn load_rom(&mut self, buffer: &[u8]) {
        let cartridge = Cartridge::from_buffer(buffer);
        let mapper = Rc::new(RefCell::new(mapper::from_cartridge(cartridge)));
        self.cpu
            .borrow_mut()
            .attach_bus(Bus::new(&self.cpu, &self.ppu, &mapper));
        self.ppu
            .borrow_mut()
            .attach_bus(Bus::new(&self.cpu, &self.ppu, &mapper));
        self.mapper = Some(mapper);
    }

    pub fn step(&mut self) {
        self.cpu.borrow_mut().step();
        self.ppu.borrow_mut().step();
    }

    pub fn step_scanline(&mut self) {
        let scanline = self.ppu.borrow().scanline;
        while self.ppu.borrow().scanline == scanline {
            self.step();
        }
    }

    pub fn step_frame(&mut self) {
        let frame = self.ppu.borrow().frame;
        while self.ppu.borrow().frame == frame {
            self.step();
        }
    }
}

#[cfg(test)]
mod tests {
    macro_rules! instr_tests {
        ($($test_name:ident: $file_name:expr,)*) => {
            $(
                #[test]
                fn $test_name() {
                    let buffer = fs::read(format!("./tests/cpu/instr_test/{}.nes", $file_name))
                        .expect("Expected test rom to exist.");
                    let mut nes = Nes::new();
                    nes.load_rom(&buffer);

                    // Run until test status is running
                    let mut addr = 0x6000;
                    let mut byte = nes.cpu.borrow_mut().read_byte(addr);
                    while byte != 0x80 {
                        nes.step();
                        byte = nes.cpu.borrow_mut().read_byte(addr);
                    }

                    // Run until test status is finished
                    byte = nes.cpu.borrow_mut().read_byte(addr);
                    while byte == 0x80 {
                        nes.step();
                        byte = nes.cpu.borrow_mut().read_byte(addr);
                    }

                    // Read output
                    let mut output = Vec::new();
                    addr = 0x6004;
                    byte = nes.cpu.borrow_mut().read_byte(addr);
                    while byte != '\0' as u8 {
                        output.push(byte);
                        addr += 1;
                        byte = nes.cpu.borrow_mut().read_byte(addr);
                    }

                    assert_eq!(
                        String::from_utf8_lossy(&output),
                        format!("\n{}\n\nPassed\n", $file_name),
                    );
                }
            )*
        }
    }

    mod instr_tests {
        use super::super::Nes;
        use std::fs;

        instr_tests!(
            test_01_basics: "01-basics",
            test_02_implied: "02-implied",
            test_03_immediate: "03-immediate",
            test_04_zero_page: "04-zero_page",
            test_05_zp_xy: "05-zp_xy",
            test_06_absolute: "06-absolute",
            test_07_abs_xy: "07-abs_xy",
            test_08_ind_x: "08-ind_x",
            test_09_ind_y: "09-ind_y",
            test_10_branches: "10-branches",
            test_11_stack: "11-stack",
            test_12_jmp_jsr: "12-jmp_jsr",
            test_13_rts: "13-rts",
        );
    }
}
