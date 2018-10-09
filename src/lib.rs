extern crate cfg_if;
extern crate js_sys;
extern crate wasm_bindgen;

mod cartridge;
pub mod cpu;
mod mapper;
mod ppu;
mod utils;

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
            .attach_memory_map(cpu::MemoryMap::new(&self.ppu, &mapper));
        self.ppu
            .borrow_mut()
            .attach_memory_map(ppu::MemoryMap::new(&mapper));
        self.mapper = Some(mapper);
    }

    pub fn execute_cycle(&mut self) {
        self.cpu.borrow_mut().execute_cycle();
    }
}

mod tests {
    use super::Nes;
    use std::fs;

    #[test]
    fn test_rom() {
        let buffer = fs::read("./tests/nestest.nes").unwrap();
        let mut nes = Nes::new();
        nes.load_rom(&buffer);
        for i in 0..8991 {
            nes.execute_cycle();
        }

        // for i in 0..10 {
        //     let mut tile = [0; 64];

        //     for index in 0..8 {
        //         let byte = nes.ppu.borrow().memory_map().read_byte(index + i * 16);
        //         for y in (0..8).rev() {
        //             tile[index as usize * 8 + y] |= if byte & 1 << y != 0 { 1 } else { 0 };
        //         }
        //     }

        //     for index in 0..8 {
        //         let byte = nes.ppu.borrow().memory_map().read_byte(index + 8 + i * 16);
        //         for y in (0..8).rev() {
        //             tile[index as usize * 8 + y] |= if byte & 1 << y != 0 { 2 } else { 0 };
        //         }
        //     }

        //     for row in 0..8 {
        //         for col in 0..8 {
        //             print!("{}", tile[row * 8 + col]);
        //         }
        //         println!("");
        //     }
        //     println!("");
        // }
    }
}
