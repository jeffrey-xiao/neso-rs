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

    pub fn step_frame(&mut self) {
        let frame = self.ppu.borrow().frame;
        while self.ppu.borrow().frame == frame {
            self.step();
        }
    }
}

mod tests {
    use super::Nes;
    use std::fs;

    #[test]
    fn test_rom() {
        let buffer = fs::read("./tests/cpu/instr_test/03-immediate.nes").unwrap();
        let mut nes = Nes::new();
        nes.load_rom(&buffer);

        let mut b = nes.cpu.borrow_mut().read_byte(0x6000);
        while b != 0x80 {
            nes.step();
            b = nes.cpu.borrow_mut().read_byte(0x6000);
        }

        b = nes.cpu.borrow_mut().read_byte(0x6000);
        while b == 0x80 {
            nes.step();
            b = nes.cpu.borrow_mut().read_byte(0x6000);
        }

        let mut addr = 0x6004;
        let mut b = nes.cpu.borrow_mut().read_byte(addr);
        let mut output = Vec::new();
        while b != '\0' as u8 {
            output.push(b);
            addr += 1;
            b = nes.cpu.borrow_mut().read_byte(addr);
        }
        println!("'{}'", String::from_utf8_lossy(&output));
    }
}
