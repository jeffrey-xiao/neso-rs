mod registers;

use self::registers::Registers;
use bus::Bus;
use cpu::Interrupt;
use std::mem;

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 240;

// http://www.thealmightyguru.com/Games/Hacking/Wiki/index.php/NES_Palette
const PALETTE: [u32; 64] = [
    0x7C7C7C, 0x0000FC, 0x0000BC, 0x4428BC, 0x940084, 0xA80020, 0xA81000, 0x881400, //
    0x503000, 0x007800, 0x006800, 0x005800, 0x004058, 0x000000, 0x000000, 0x000000, //
    0xBCBCBC, 0x0078F8, 0x0058F8, 0x6844FC, 0xD800CC, 0xE40058, 0xF83800, 0xE45C10, //
    0xAC7C00, 0x00B800, 0x00A800, 0x00A844, 0x008888, 0x000000, 0x000000, 0x000000, //
    0xF8F8F8, 0x3CBCFC, 0x6888FC, 0x9878F8, 0xF878F8, 0xF85898, 0xF87858, 0xFCA044, //
    0xF8B800, 0xB8F818, 0x58D854, 0x58F898, 0x00E8D8, 0x787878, 0x000000, 0x000000, //
    0xFCFCFC, 0xA4E4FC, 0xB8B8F8, 0xD8B8F8, 0xF8B8F8, 0xF8A4C0, 0xF0D0B0, 0xFCE0A8, //
    0xF8D878, 0xD8F878, 0xB8F8B8, 0xB8F8D8, 0x00FCFC, 0xF8D8F8, 0x000000, 0x000000, //
];

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
    pub image_index: usize,
    pub image: [u8; SCREEN_WIDTH * SCREEN_HEIGHT * 3],
    pub primary_oam: [u8; 0x100],
    pub secondary_oam: [u8; 0x20],
    pub vram: [u8; 0x2000],
    pub palette_ram: [u8; 0x20],
    pub cycle: u16,    // [0, 340]
    pub scanline: i16, // [-1, 261]
    pub frame: u64,
    pub bus: Option<Bus>,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            r: Registers::new(),
            image_index: 0,
            image: [0; SCREEN_WIDTH * SCREEN_HEIGHT * 3],
            primary_oam: [0; 0x100],
            secondary_oam: [0; 0x20],
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
        self.cycle = 0;
        self.scanline = -1;
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
        // print!("READ PPU REGISTER {} {} {} {:x} {}; ", index, self.scanline, self.cycle, self.r.read_ppu_status(), self.r.v_blank_started);
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
            4 => self.primary_oam[self.r.oam_addr as usize],
            // PPUSCROLL
            5 => self.r.last_written_byte,
            // PPUADDR
            6 => self.r.last_written_byte,
            // PPUDATA
            7 => {
                let mut ret = self.read_byte(self.r.bus_address);
                if self.r.bus_address < 0x3F00 {
                    mem::swap(&mut ret, &mut self.r.buffer);
                } else {
                    self.read_byte(self.r.bus_address - 0x1000);
                }
                self.r.bus_address += self.r.vram_address_increment;
                ret
            },
            _ => panic!("Invalid ppu register index to read: {}.", index),
        }
    }

    pub fn write_register(&mut self, index: usize, val: u8) {
        // print!("WRITE PPU REGISTER {} {:08b}; ", index, val);
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
                self.primary_oam[self.r.oam_addr as usize] = val;
                self.r.oam_addr += 1;
            },
            // PPUSCROLL
            5 => self.r.write_ppu_scroll(val),
            // PPUADDR
            6 => self.r.write_ppu_addr(val),
            // PPUDATA
            7 => {
                // println!("WRITE TO ADDR {:x}", addr);
                let addr = self.r.bus_address;
                self.write_byte(addr, val);
                self.r.bus_address += self.r.vram_address_increment;
            },
            _ => panic!("Invalid ppu register index to write: {}.", index),
        }
    }

    fn fetch_nametable_byte(&mut self) {
        let addr = 0x2000 | (self.r.v & 0x0FFF);
        self.r.nametable_byte = self.read_byte(addr);
    }

    fn fetch_attribute_table_byte(&mut self) {
        let coarse_x = self.r.v >> 2;
        let coarse_y = self.r.v >> 7;
        let addr = 0x23C0 | (self.r.v & 0x0C00) | (coarse_x & 0x07) | ((coarse_y & 0x07) << 3);
        let attribute_table_byte = self.read_byte(addr);
        let offset = (self.r.v & 0x02) | ((self.r.v & 0x40) >> 4);
        self.r.palette = (attribute_table_byte >> offset) & 0x03;
    }

    fn fetch_tile_byte(&mut self, high: bool) {
        let fine_y = (self.r.v >> 12) & 0x07;
        let tile_offset = self.r.nametable_byte as u16 * 16;
        let addr = self.r.background_pattern_table_address + tile_offset + fine_y;
        if high {
            self.r.high_tile_byte = self.read_byte(addr + 8);
        } else {
            self.r.low_tile_byte = self.read_byte(addr);
        }
    }

    fn load_tile(&mut self) {
        let mut curr_tile = 0;
        for _ in 0..8 {
            let color =
                ((self.r.high_tile_byte >> 6) & 0x02) | ((self.r.low_tile_byte >> 7) & 0x01);
            self.r.high_tile_byte <<= 1;
            self.r.low_tile_byte <<= 1;
            curr_tile <<= 4;
            curr_tile |= ((self.r.palette as u64) << 2) | color as u64;
        }
        self.r.tile |= curr_tile;
    }

    fn compute_background_pixel(&self) -> u8 {
        let x = (self.cycle - 1) as u8;

        if (x < 8 && !self.r.show_left_background) || !self.r.show_background {
            return 0;
        }

        ((self.r.tile >> 32 >> ((7 - self.r.x) * 4)) & 0x0F) as u8
    }

    // TODO: implement priority
    fn compute_sprite_pixel(&self) -> u8 {
        let y = self.scanline as u8;
        let x = (self.cycle - 1) as u8;

        if (x < 8 && !self.r.show_left_sprites) || !self.r.show_sprites {
            return 0;
        }

        for i in 0..8 {
            let sprite_y = self.secondary_oam[i * 4];
            let sprite_x = self.secondary_oam[i * 4 + 3];
            let mut tile_index = self.secondary_oam[i * 4 + 1];
            let attributes = self.secondary_oam[i * 4 + 2];

            if sprite_y & tile_index & attributes & sprite_x == 0xFF {
                break;
            }

            if !(sprite_x <= x && x < sprite_x + 8) {
                continue;
            }

            if !(1 <= sprite_y && sprite_y <= 239) {
                continue;
            }

            let mut py = y - sprite_y;
            let mut px = 7 - (x - sprite_x);
            let mut nametable_address = self.r.sprite_pattern_table_address;

            if attributes & 0x40 != 0 {
                px = self.r.sprite_size.0 - 1 - px;
            }

            if attributes & 0x80 != 0 {
                py = self.r.sprite_size.1 - 1 - py;
            }

            if self.r.sprite_size.1 == 16 {
                nametable_address = (attributes as u16 & 0x01) * 0x1000;
                tile_index &= 0xFE;
                if py >= 8 {
                    py -= 8;
                    tile_index += 1;
                }
            }

            let addr = nametable_address + tile_index as u16 * 16 + py as u16;
            let low_tile_bit = (self.read_byte(addr) >> px) & 0x01;
            let high_tile_bit = (self.read_byte(addr + 8) >> px) & 0x01;
            let palette = (attributes & 0x03) as u8;
            let color = low_tile_bit | (high_tile_bit << 1);

            if color == 0 {
                continue;
            }

            return (palette << 2) | color;
        }

        0
    }

    fn draw_pixel(&mut self) {
        let background_pixel = self.compute_background_pixel() as u16;
        let sprite_pixel = self.compute_sprite_pixel() as u16;

        let color;
        if sprite_pixel & 0x03 != 0 {
            color = PALETTE[self.read_byte(0x3F10 + sprite_pixel) as usize];
        } else if background_pixel & 0x03 != 0 {
            color = PALETTE[self.read_byte(0x3F00 + background_pixel) as usize];
        } else {
            color = PALETTE[self.read_byte(0x3F00) as usize];
        }

        self.image[self.image_index] = ((color >> 16) & 0xFF) as u8;
        self.image[self.image_index + 1] = ((color >> 8) & 0xFF) as u8;
        self.image[self.image_index + 2] = ((color >> 0) & 0xFF) as u8;
        self.image_index += 3;
    }

    pub fn step(&mut self) {
        self.cycle += 1;
        if self.cycle == 341 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline == 261 {
                self.scanline = -1;
                self.frame += 1;
                self.image_index = 0;
            }
        }

        let visible_scanline = 0 <= self.scanline && self.scanline <= 239;
        let visible_cycle = 1 <= self.cycle && self.cycle <= 256;
        let prefetch_cycle = 321 <= self.cycle && self.cycle <= 336;
        let sprite_clear_cycle = 1 <= self.cycle && self.cycle <= 64;
        let sprite_evaluation_cycle = 65 <= self.cycle && self.cycle <= 256;
        let sprite_fetch_cycle = 257 <= self.cycle && self.cycle <= 320;

        if visible_scanline || self.scanline == -1 {
            if visible_scanline && visible_cycle {
                self.draw_pixel();
            }

            if self.scanline == -1 && 280 <= self.cycle && self.cycle <= 304 {
                self.r.copy_scroll_y();
            }

            if self.cycle == 257 {
                self.r.copy_scroll_x();
            }

            // background pipeline
            if visible_cycle || prefetch_cycle {
                self.r.tile <<= 4;
                match self.cycle & 0x07 {
                    1 => self.fetch_nametable_byte(),
                    3 => self.fetch_attribute_table_byte(),
                    5 => self.fetch_tile_byte(false),
                    7 => self.fetch_tile_byte(true),
                    0 => {
                        self.load_tile();
                        if self.cycle == 256 {
                            self.r.increment_scroll_y();
                        } else {
                            self.r.increment_scroll_x();
                        }
                    },
                    _ => {},
                }
            }

            // sprite pipeline
            // TODO: make fetches cycle accurate, add sprite data
            // if sprite_clear_cycle && self.cycle & 0x01 != 0 {
            //     self.secondary_oam[self.cycle as usize / 2] = 0xFF;
            // }

            if self.cycle == 257 {
                for i in 0..0x20 {
                    self.secondary_oam[i] = 0;
                }
                let mut secondary_oam_index = 0;
                for i in 0..64 {
                    let y = self.primary_oam[i * 4] as i16;
                    let lo = y;
                    let hi = y + self.r.sprite_size.1 as i16 - 1;
                    if !(lo <= self.scanline + 1 && self.scanline + 1 <= hi) {
                        continue;
                    }

                    if secondary_oam_index < 0x20 {
                        self.secondary_oam[secondary_oam_index] = self.primary_oam[i * 4];
                        self.secondary_oam[secondary_oam_index + 1] = self.primary_oam[i * 4 + 1];
                        self.secondary_oam[secondary_oam_index + 2] = self.primary_oam[i * 4 + 2];
                        self.secondary_oam[secondary_oam_index + 3] = self.primary_oam[i * 4 + 3];
                        secondary_oam_index += 4;
                    } else {
                        self.r.sprite_overflow = true;
                    }
                }
            }
        }

        if self.scanline == 241 && self.cycle == 1 {
            // println!("V BLANK ON");
            self.r.v_blank_started = true;
            if self.r.nmi_enabled {
                let cpu = self.bus().cpu();
                cpu.borrow_mut().trigger_interrupt(Interrupt::NMI);
            }
        }

        if self.scanline == -1 && self.cycle == 1 {
            self.r.v_blank_started = false;
            // println!("V BLANK OFF");
        }
    }
}
