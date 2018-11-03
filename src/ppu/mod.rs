mod registers;

use self::registers::Registers;
use bus::Bus;
use cpu::Interrupt;
use std::mem;

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 240;

// http://www.thealmightyguru.com/Games/Hacking/Wiki/index.php/NES_Palette
#[cfg_attr(rustfmt, rustfmt_skip)]
const PALETTE: [u32; 64] = [
    0x007C_7C7C, 0x0000_00FC, 0x0000_00BC, 0x0044_28BC, 0x0094_0084, 0x00A8_0020, 0x00A8_1000, 0x0088_1400, //
    0x0050_3000, 0x0000_7800, 0x0000_6800, 0x0000_5800, 0x0000_4058, 0x0000_0000, 0x0000_0000, 0x0000_0000, //
    0x00BC_BCBC, 0x0000_78F8, 0x0000_58F8, 0x0068_44FC, 0x00D8_00CC, 0x00E4_0058, 0x00F8_3800, 0x00E4_5C10, //
    0x00AC_7C00, 0x0000_B800, 0x0000_A800, 0x0000_A844, 0x0000_8888, 0x0000_0000, 0x0000_0000, 0x0000_0000, //
    0x00F8_F8F8, 0x003C_BCFC, 0x0068_88FC, 0x0098_78F8, 0x00F8_78F8, 0x00F8_5898, 0x00F8_7858, 0x00FC_A044, //
    0x00F8_B800, 0x00B8_F818, 0x0058_D854, 0x0058_F898, 0x0000_E8D8, 0x0078_7878, 0x0000_0000, 0x0000_0000, //
    0x00FC_FCFC, 0x00A4_E4FC, 0x00B8_B8F8, 0x00D8_B8F8, 0x00F8_B8F8, 0x00F8_A4C0, 0x00F0_D0B0, 0x00FC_E0A8, //
    0x00F8_D878, 0x00D8_F878, 0x00B8_F8B8, 0x00B8_F8D8, 0x0000_FCFC, 0x00F8_D8F8, 0x0000_0000, 0x0000_0000, //
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MirroringMode {
    Horizontal = 0,
    Vertical = 1,
    Lower = 2,
    Upper = 3,
    None = 4,
}

impl Default for MirroringMode {
    fn default() -> MirroringMode {
        MirroringMode::Horizontal
    }
}

const MIRRORING_MODE_TABLE: [usize; 20] = [
    0, 0, 1, 1, // Horizontal
    0, 1, 0, 1, // Vertical
    0, 0, 0, 0, // Lower
    1, 1, 1, 1, // Upper
    0, 1, 2, 3, // None
];

pub struct Ppu {
    pub r: Registers,
    pub image_index: usize,
    pub image: [u8; SCREEN_WIDTH * SCREEN_HEIGHT * 3],
    pub primary_oam: [u8; 0x100],
    pub secondary_oam: [u8; 0x20],
    pub is_sprite_0: [bool; 8],
    pub vram: [u8; 0x2000],
    pub palette_ram: [u8; 0x20],
    pub cycle: u16,    // [0, 340]
    pub scanline: i16, // [-1, 260]
    pub frame: u64,
    pub bus: Option<Bus>,
}

impl Ppu {
    pub fn new() -> Ppu {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let palette_ram = [
            0x09, 0x01, 0x00, 0x01,
            0x00, 0x02, 0x02, 0x0D,
            0x08, 0x10, 0x08, 0x24,
            0x00, 0x00, 0x04, 0x2C,
            0x09, 0x01, 0x34, 0x03,
            0x00, 0x04, 0x00, 0x14,
            0x08, 0x3A, 0x00, 0x02,
            0x00, 0x20, 0x2C, 0x08,
        ];

        Ppu {
            r: Registers::new(),
            image_index: 0,
            image: [0; SCREEN_WIDTH * SCREEN_HEIGHT * 3],
            primary_oam: [0; 0x100],
            secondary_oam: [0; 0x20],
            is_sprite_0: [false; 8],
            vram: [0; 0x2000],
            palette_ram,
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

    fn bus(&self) -> &Bus {
        self.bus.as_ref().expect("[PPU] No bus attached.")
    }

    // memory map related functions
    pub fn read_byte(&self, mut addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                let mapper = self.bus().mapper();
                let ret = mapper.borrow().read_byte(addr);
                ret
            },
            0x2000..=0x3EFF => {
                let mapper = self.bus().mapper();
                let addr = (addr - 0x2000) % 0x1000;
                let index = (addr / 0x400) as usize;
                let offset = (addr % 0x400) as usize;
                let mirroring_mode = mapper.borrow().mirroring_mode() as usize;
                self.vram[MIRRORING_MODE_TABLE[mirroring_mode * 4 + index] * 0x400 + offset]
            },
            0x3F00..=0x3FFF => {
                if addr == 0x3F10 || addr == 0x3F14 || addr == 0x3F18 || addr == 0x3F1C {
                    addr -= 0x10;
                }
                self.palette_ram[((addr - 0x3F00) % 0x20) as usize]
            },
            _ => panic!("[PPU] Invalid read with memory address: {:#06x}.", addr),
        }
    }

    pub fn write_byte(&mut self, mut addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => {
                let mapper = self.bus().mapper();
                mapper.borrow_mut().write_byte(addr, val);
            },
            0x2000..=0x3EFF => {
                let mapper = self.bus().mapper();
                let addr = (addr - 0x2000) % 0x1000;
                let index = (addr / 0x400) as usize;
                let offset = (addr % 0x400) as usize;
                let mirroring_mode = mapper.borrow().mirroring_mode() as usize;
                self.vram[MIRRORING_MODE_TABLE[mirroring_mode * 4 + index] * 0x400 + offset] = val;
            },

            0x3F00..=0x3FFF => {
                if addr == 0x3F10 || addr == 0x3F14 || addr == 0x3F18 || addr == 0x3F1C {
                    addr -= 0x10;
                }
                self.palette_ram[((addr - 0x3F00) % 0x20) as usize] = val
            },
            _ => panic!("[PPU] Invalid write with memory address: {:#06x}.", addr),
        }
    }

    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr {
            // PPUCTRL
            0x2000 => self.r.last_written_byte,
            // PPUMASK
            0x2001 => self.r.last_written_byte,
            // PPUSTATUS
            0x2002 => self.r.read_ppu_status(),
            // OAMADDR
            0x2003 => self.r.last_written_byte,
            // OAMDATA
            0x2004 => self.primary_oam[self.r.oam_addr as usize],
            // PPUSCROLL
            0x2005 => self.r.last_written_byte,
            // PPUADDR
            0x2006 => self.r.last_written_byte,
            // PPUDATA
            0x2007 => {
                let mut ret = self.read_byte(self.r.bus_address);
                if self.r.bus_address < 0x3F00 {
                    mem::swap(&mut ret, &mut self.r.buffer);
                } else {
                    self.r.buffer = self.read_byte(self.r.bus_address - 0x1000);
                }
                self.r.bus_address += self.r.vram_address_increment;
                ret
            },
            _ => panic!("[PPU] Invalid ppu register to read: {:#06x}.", addr),
        }
    }

    pub fn write_register(&mut self, addr: u16, val: u8) {
        self.r.last_written_byte = val;
        match addr {
            // PPUCTRL
            0x2000 => self.r.write_ppu_ctrl(val),
            // PPUMASK
            0x2001 => self.r.write_ppu_mask(val),
            // PPUSTATUS
            0x2002 => return,
            // OAMADDR
            0x2003 => self.r.oam_addr = val,
            // OAMDATA
            0x2004 => {
                self.primary_oam[self.r.oam_addr as usize] = val;
                self.r.oam_addr = self.r.oam_addr.wrapping_add(1);
            },
            // PPUSCROLL
            0x2005 => self.r.write_ppu_scroll(val),
            // PPUADDR
            0x2006 => self.r.write_ppu_addr(val),
            // PPUDATA
            0x2007 => {
                let addr = self.r.bus_address;
                self.write_byte(addr, val);
                self.r.bus_address += self.r.vram_address_increment;
            },
            _ => panic!("[PPU] Invalid ppu register to write: {:#06x}.", addr),
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
        let tile_offset = u16::from(self.r.nametable_byte) * 16;
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
            curr_tile |= ((u64::from(self.r.palette)) << 2) | u64::from(color);
        }
        self.r.tile |= curr_tile;
    }

    fn compute_background_pixel(&self) -> u16 {
        let x = (self.cycle - 1) as u8;

        if (x < 8 && !self.r.show_left_background) || !self.r.show_background {
            return 0;
        }

        ((self.r.tile >> 32 >> ((7 - self.r.x) * 4)) & 0x0F) as u16
    }

    fn compute_sprite_pixel(&self) -> (u16, bool, bool) {
        let y = self.scanline as u8;
        let x = (self.cycle - 1) as u8;

        if (x < 8 && !self.r.show_left_sprites) || !self.r.show_sprites {
            return (0, false, false);
        }

        for i in 0..8 {
            let sprite_y = self.secondary_oam[i * 4].wrapping_add(1);
            let sprite_x = self.secondary_oam[i * 4 + 3];
            let mut tile_index = self.secondary_oam[i * 4 + 1];
            let attributes = self.secondary_oam[i * 4 + 2];

            if sprite_y & tile_index & attributes & sprite_x == 0xFF {
                break;
            }

            if !(sprite_x <= x && x < sprite_x.wrapping_add(8)) {
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
                nametable_address = (u16::from(tile_index) & 0x01) * 0x1000;
                tile_index &= 0xFE;
                if py >= 8 {
                    py -= 8;
                    tile_index += 1;
                }
            }

            let addr = nametable_address + u16::from(tile_index) * 16 + u16::from(py);
            let low_tile_bit = (self.read_byte(addr) >> px) & 0x01;
            let high_tile_bit = (self.read_byte(addr + 8) >> px) & 0x01;
            let palette = (attributes & 0x03) as u8;
            let color = low_tile_bit | (high_tile_bit << 1);

            if color == 0 {
                continue;
            }

            return (
                u16::from((palette << 2) | color),
                (attributes & 0x20) != 0,
                self.is_sprite_0[i],
            );
        }

        (0, false, false)
    }

    fn draw_pixel(&mut self) {
        let background_pixel = self.compute_background_pixel();
        let (sprite_pixel, sprite_priority, is_sprite_0) = self.compute_sprite_pixel();

        let background_on = background_pixel & 0x03 != 0;
        let sprite_on = sprite_pixel & 0x03 != 0;

        let addr = match (background_on, sprite_on) {
            (false, false) => 0x3F00,
            (false, true) => 0x3F10 + sprite_pixel,
            (true, false) => 0x3F00 + background_pixel,
            (true, true) => {
                if self.cycle != 256 && is_sprite_0 {
                    self.r.sprite_0_hit = true;
                }

                if !sprite_priority {
                    0x3F10 + sprite_pixel
                } else {
                    0x3F00 + background_pixel
                }
            },
        };

        let color = PALETTE[self.read_byte(addr) as usize & 0x3F];
        self.image[self.image_index] = ((color >> 16) & 0xFF) as u8;
        self.image[self.image_index + 1] = ((color >> 8) & 0xFF) as u8;
        self.image[self.image_index + 2] = (color & 0xFF) as u8;
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
                    self.secondary_oam[i] = 0xFF;
                }
                let mut secondary_oam_index = 0;
                for i in 0..64 {
                    let y = i16::from(self.primary_oam[i * 4]) + 1;
                    let lo = y;
                    let hi = y + i16::from(self.r.sprite_size.1) - 1;
                    if !(lo <= self.scanline + 1 && self.scanline + 1 <= hi) {
                        continue;
                    }

                    if secondary_oam_index < 0x20 {
                        self.secondary_oam[secondary_oam_index] = self.primary_oam[i * 4];
                        self.secondary_oam[secondary_oam_index + 1] = self.primary_oam[i * 4 + 1];
                        self.secondary_oam[secondary_oam_index + 2] = self.primary_oam[i * 4 + 2];
                        self.secondary_oam[secondary_oam_index + 3] = self.primary_oam[i * 4 + 3];
                        self.is_sprite_0[secondary_oam_index / 4] = i == 0;
                        secondary_oam_index += 4;
                    } else {
                        self.r.sprite_overflow = true;
                    }
                }
            }
        }

        if self.scanline == 241 && self.cycle == 1 {
            self.r.v_blank_started = true;
            if self.r.nmi_enabled {
                let cpu = self.bus().cpu();
                cpu.borrow_mut().trigger_interrupt(Interrupt::NMI);
            } else {
                // println!("[PPU] NMI is not enabled, so interrupt is skipped.");
            }
        }

        if self.scanline == -1 && self.cycle == 1 {
            self.r.v_blank_started = false;
            self.r.sprite_0_hit = false;
            self.r.sprite_overflow = false;
        }
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Ppu::new()
    }
}
