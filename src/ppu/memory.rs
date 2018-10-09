use std::rc::{Rc, Weak};
use ppu::registers::Registers;
use mapper::Mapper;
use std::cell::RefCell;
use std::mem;

const MIRRORING_MODE_TABLE: [usize; 12] = [
    0, 0, 1, 1, // Horizontal
    0, 1, 0, 1, // Vertical
    0, 1, 2, 3, // None
];

pub struct MemoryMap {
    pub r: Registers,
    pub oam: [u8; 0x100],
    pub vram: [u8; 0x2000],
    pub palette_ram: [u8; 0x20],
    mapper: Weak<RefCell<Box<Mapper>>>,
}

impl MemoryMap {
    pub fn new(mapper: &Rc<RefCell<Box<Mapper>>>) -> Self {
        MemoryMap {
            r: Registers::new(),
            oam: [0; 0x100],
            vram: [0; 0x2000],
            palette_ram: [0; 0x20],
            mapper: Rc::downgrade(mapper),
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                let mapper = self.mapper.upgrade().unwrap();
                let ret = mapper.borrow().read_byte(addr);
                ret
            },
            0x2000..=0x3EFF => {
                let index = ((addr - 0x2000) / 0x400) as usize;
                let offset = ((addr - 0x2000) % 0x400) as usize;
                self.vram[MIRRORING_MODE_TABLE[index] * 0x400 + offset]
            },
            0x3F00..=0x3FFF => self.palette_ram[((addr - 0x3F00) % 0x20) as usize],
            _ => panic!("Invalid memory address: {:#6x}.", addr),
        }
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => {
                let mapper = self.mapper.upgrade().unwrap();
                mapper.borrow_mut().read_byte(addr);
            },
            0x2000..=0x3EFF => {
                let mapper = self.mapper.upgrade().unwrap();
                let addr = (addr - 0x2000) % 0x1000;
                let index = (addr / 0x400) as usize;
                let offset = (addr % 0x400) as usize;
                let mirroring_mode = mapper.borrow().mirroring_mode() as usize;
                self.vram[MIRRORING_MODE_TABLE[mirroring_mode + index] * 0x400 + offset] = val;
            },
            0x3F00..=0x3FFF => self.palette_ram[((addr - 0x3F00) % 0x20) as usize] = val,
            _ => panic!("Invalid memory address: {:#6x}.", addr),
        }
    }

    pub fn read_register(&mut self, index: usize) -> u8 {
        println!("READ PPU REGISTER {}", index);
        match index {
            // PPUCTRL
            0 => self.r.last_written_byte,
            // PPUMASK
            1 => self.r.last_written_byte,
            // PPUSTATUS
            2 => self.r.read_ppu_status(),
            // OAMADDR
            3 => self.r.last_written_byte,
            // OAMDATA
            4 => self.oam[self.r.oam_addr as usize],
            // PPUSCROLL
            5 => self.r.last_written_byte,
            // PPUADDR
            6 => self.r.last_written_byte,
            // PPUDATA
            7 => {
                let mut ret = self.read_byte(self.r.ppu_addr);
                if self.r.ppu_addr % 0x4000 < 0x3F00 {
                    mem::swap(&mut ret, &mut self.r.buffer);
                } else {
                    self.read_byte(self.r.ppu_addr - 0x1000);
                }
                self.r.ppu_addr += self.r.vram_address_increment;
                ret
            },
            _ => panic!("Invalid ppu register index to read: {}.", index),
        }
    }

    pub fn write_register(&mut self, index: usize, val: u8) {
        println!("WRITE PPU REGISTER {} {:08b}", index, val);
        self.r.last_written_byte = val;
        match index {
            // PPUCTRL
            0 => self.r.write_ppu_ctrl(val),
            // PPUMASK
            1 => self.r.write_ppu_mask(val),
            // PPUSTATUS
            2 => return,
            // OAMADDR
            3 => self.r.oam_addr = val,
            // OAMDATA
            4 => {
                self.oam[self.r.oam_addr as usize] = val;
                self.r.oam_addr += 1;
            },
            // PPUSCROLL
            5 => {
                if !self.r.address_latch {
                    self.r.scroll_x = val;
                } else {
                    self.r.scroll_y = val;
                }
                self.r.address_latch ^= true;
            },
            // PPUADDR
            6 => {
                if !self.r.address_latch {
                    self.r.ppu_addr = (val as u16) << 8;
                } else {
                    self.r.ppu_addr |= val as u16;
                }
                self.r.address_latch ^= true;
            },
            // PPUDATA
            7 => {
                let addr = self.r.ppu_addr;
                self.write_byte(addr, val);
                self.r.ppu_addr += self.r.vram_address_increment;
            },
            _ => panic!("Invalid ppu register index to write: {}.", index),
        }
    }
}
