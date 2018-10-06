extern crate cfg_if;
extern crate js_sys;
extern crate wasm_bindgen;

mod cartridge;
mod cpu;
mod mapper;
mod memory;
mod utils;

use cpu::Cpu;
use memory::Memory;
use std::cell::RefCell;
use std::rc::Rc;

struct Nes {
    cpu: Cpu,
    memory: Rc<RefCell<Memory>>,
}

impl Nes {
    pub fn new() -> Self {
        let memory = Rc::new(RefCell::new(Memory::new()));
        Nes {
            cpu: Cpu::new(Rc::clone(&memory)),
            memory,
        }
    }

    pub fn load_rom(&mut self, buffer: &[u8]) {
        self.memory.borrow_mut().load_rom(buffer);
    }

    pub fn execute_cycle(&mut self) {
        self.cpu.execute_cycle();
    }
}

mod tests {
    use super::Nes;
    use std::fs;
    use std::io::Read;

    #[test]
    fn test_rom() {
        let mut buffer = fs::read("./tests/nestest.nes").unwrap();
        let mut nes = Nes::new();
        nes.load_rom(&buffer);
        for i in 0..8991 {
            nes.execute_cycle();
        }
    }
}
