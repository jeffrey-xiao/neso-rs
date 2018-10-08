use mapper::Mapper;
use ppu::Ppu;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct MemoryMap {
    pub ram: [u8; 0x800],
    ppu: Weak<RefCell<Ppu>>,
    mapper: Weak<RefCell<Box<Mapper>>>,
}

impl MemoryMap {
    pub fn new(ppu: &Rc<RefCell<Ppu>>, mapper: &Rc<RefCell<Box<Mapper>>>) -> Self {
        MemoryMap {
            ram: [0; 0x800],
            ppu: Rc::downgrade(ppu),
            mapper: Rc::downgrade(mapper),
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr % 0x0800) as usize],
            0x2000..=0x3FFF => {
                let ppu = self.ppu.upgrade().unwrap();
                let ret = ppu
                    .borrow_mut()
                    .read_register(((addr - 0x2000) % 8) as usize);
                ret
            },
            // TODO: Implement APU and IO maps
            0x4016 => 0,
            0x4017 => 0,
            0x4018..=0x401F => panic!("CPU Test Mode not implemented."),
            0x4020..=0xFFFE => {
                let mapper = self.mapper.upgrade().unwrap();
                let ret = mapper.borrow().read_byte(addr);
                ret
            },
            _ => panic!("Invalid memory address: {:#6x}.", addr),
        }
    }

    pub fn read_word(&self, addr: u16) -> u16 {
        ((self.read_byte(addr + 1) as u16) << 8) | self.read_byte(addr) as u16
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr % 0x0800) as usize] = val,
            0x2000..=0x3FFF => {
                let mut ppu = self.ppu.upgrade().unwrap();
                ppu.borrow_mut()
                    .write_register(((addr - 0x2000) % 8) as usize, val);
            },
            // TODO: Implement APU and IO maps
            0x4014 => {
                let cpu_addr = (val as u16) << 8;
                let mut ppu = self.ppu.upgrade().unwrap();
                for offset in 0..0xFF {
                    ppu.borrow_mut().memory_map_mut().oam[offset] = self.read_byte(cpu_addr);
                }
                // TODO increment cycles
            },
            0x4016 => {},
            0x4017 => {},
            0x4018..=0x401F => panic!("CPU Test Mode not implemented."),
            0x4020..=0xFFFE => {
                let mut mapper = self.mapper.upgrade().unwrap();
                mapper.borrow_mut().write_byte(addr, val);
            },
            _ => panic!("Invalid memory address: {:#6x}.", addr),
        }
    }
}
