mod registers;

use self::registers::Registers;
use bus::Bus;
use std::mem;

#[derive(Clone, Copy)]
pub enum MirroringMode {
    Horizontal = 0,
    Vertical = 1,
    None = 2,
}

const MIRRORING_MODE_TABLE: [usize; 12] = [
    0, 0, 1, 1, // Horizontal
    0, 1, 0, 1, // Vertical
    0, 1, 2, 3, // None
];

pub struct Ppu {
    pub r: Registers,
    pub oam: [u8; 0x100],
    pub vram: [u8; 0x2000],
    pub palette_ram: [u8; 0x20],
    pub cycle: u16,
    pub scanline: u16,
    pub frame: u64,
    pub bus: Option<Bus>,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            r: Registers::new(),
            oam: [0; 0x100],
            vram: [0; 0x2000],
            palette_ram: [0; 0x20],
            bus: None,
            cycle: 0,
            scanline: 0,
            frame: 0,
        }
    }

    pub fn reset(&mut self) {
        self.r.write_ppu_ctrl(0);
        self.r.write_ppu_mask(0);
        self.r.oam_addr = 0;
        self.cycle = 340;
        self.scanline = 240;
        self.frame = 0;
    }

    pub fn attach_bus(&mut self, bus: Bus) {
        self.bus = Some(bus);
    }

    // memory map related functions
    // TODO: Handle error with no bus attached.
    fn bus(&self) -> &Bus {
        self.bus.as_ref().unwrap()
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                let mapper = self.bus().mapper();
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
                let mapper = self.bus().mapper();
                mapper.borrow_mut().read_byte(addr);
            },
            0x2000..=0x3EFF => {
                let mapper = self.bus().mapper();
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

    pub fn step(&mut self) {
        self.cycle += 1;
        if self.cycle == 341 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline == 262 {
                self.scanline = 0;
                self.frame += 1;
            }
        }
    }
}
