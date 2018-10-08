use mapper::Mapper;
use std::cell::RefCell;
use std::mem;
use std::rc::{Rc, Weak};

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
    pub memory_map: Option<MemoryMap>,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu { memory_map: None }
    }

    pub fn attach_memory_map(&mut self, memory_map: MemoryMap) {
        self.memory_map = Some(memory_map);
    }

    pub fn memory_map(&self) -> &MemoryMap {
        self.memory_map.as_ref().unwrap()
    }

    pub fn memory_map_mut(&mut self) -> &mut MemoryMap {
        self.memory_map.as_mut().unwrap()
    }

    pub fn read_register(&mut self, index: usize) -> u8 {
        self.memory_map_mut().read_register(index)
    }

    pub fn write_register(&mut self, index: usize, val: u8) {
        self.memory_map_mut().write_register(index, val);
    }
}

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
                let index = ((addr - 0x2000) / 0x400) as usize;
                let offset = ((addr - 0x2000) % 0x400) as usize;
                self.vram[MIRRORING_MODE_TABLE[index] * 0x400 + offset] = val;
            },
            0x3F00..=0x3FFF => self.palette_ram[((addr - 0x3F00) % 0x20) as usize] = val,
            _ => panic!("Invalid memory address: {:#6x}.", addr),
        }
    }

    pub fn read_register(&mut self, index: usize) -> u8 {
        match index {
            // PPUCTRL
            0 => self.r.last_written_byte,
            // PPUMASK
            1 => self.r.last_written_byte,
            // PPUSTATUS
            2 => self.r.read_ppu_scroll(),
            // OAMADDR
            3 => self.r.oam_addr,
            // OAMDATA
            4 => self.oam[self.r.oam_addr as usize],
            // PPUSCROLL
            5 => self.r.last_written_byte,
            // PPUADDR
            6 => self.r.last_written_byte,
            // PPUDATA
            7 => {
                let mut ret = self.read_byte(self.r.ppu_addr);
                if self.r.ppu_addr < 0x3F00 {
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
                    self.r.ppu_addr = val as u16;
                } else {
                    self.r.ppu_addr |= (val as u16) << 8;
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

pub struct Registers {
    // PPUCTRL
    nametable_address: u16,
    vram_address_increment: u16,
    sprite_pattern_table_address: u16,
    background_pattern_table_address: u16,
    sprite_size: (u16, u16),
    is_master: bool,
    nmi_enabled: bool,

    // PPUMASK
    greyscale_enabled: bool,
    show_left_background: bool,
    show_left_sprites: bool,
    show_background: bool,
    show_sprites: bool,
    emphasize_red: bool,
    emphasize_green: bool,
    emphasize_blue: bool,

    // PPUSTATUS
    sprite_overflow: bool,
    sprite_0_hit: bool,
    v_blank_started: bool,

    // OAMADDR
    oam_addr: u8,

    // PPUSCROLL
    scroll_x: u8,
    scroll_y: u8,

    // PPUADDR
    ppu_addr: u16,

    // PPUDATA
    buffer: u8,

    last_written_byte: u8,
    address_latch: bool,
}

const NAMETABLE_ADDRESSES: [u16; 4] = [0x2000, 0x2400, 0x2800, 0x2C00];
impl Registers {
    // TODO: check if there are defaults
    pub fn new() -> Registers {
        Registers {
            // PPUCTRL
            nametable_address: 0,
            vram_address_increment: 0,
            sprite_pattern_table_address: 0,
            background_pattern_table_address: 0,
            sprite_size: (8, 8),
            is_master: false,
            nmi_enabled: false,

            // PPUMASK
            greyscale_enabled: false,
            show_left_background: false,
            show_left_sprites: false,
            show_background: false,
            show_sprites: false,
            emphasize_red: false,
            emphasize_green: false,
            emphasize_blue: false,

            // PPUSTATUS
            sprite_overflow: false,
            sprite_0_hit: false,
            v_blank_started: false,

            // OAMADDR
            oam_addr: 0,

            // PPUSCROLL
            scroll_x: 0,
            scroll_y: 0,

            // PPUADDR
            ppu_addr: 0,

            // PPUDATA
            buffer: 0,

            last_written_byte: 0,
            address_latch: false,
        }
    }

    pub fn read_ppu_scroll(&mut self) -> u8 {
        self.address_latch = false;
        let mut ret = (self.last_written_byte & 0x1F)
            | if self.sprite_overflow { 0x20 } else { 0 }
            | if self.sprite_0_hit { 0x40 } else { 0 }
            | if self.v_blank_started { 0x80 } else { 0 };
        self.v_blank_started = false;
        ret
    }

    pub fn write_ppu_ctrl(&mut self, val: u8) {
        self.nametable_address = NAMETABLE_ADDRESSES[(val & 0x3) as usize];
        self.vram_address_increment = if val & 0x04 != 0 { 32 } else { 1 };
        self.sprite_pattern_table_address = if val & 0x08 != 0 { 0x1000 } else { 0x0000 };
        self.background_pattern_table_address = if val & 0x10 != 0 { 0x1000 } else { 0x0000 };
        self.sprite_size = if val & 0x20 != 0 { (8, 16) } else { (8, 8) };
        self.is_master = val & 0x40 != 0;
        self.nmi_enabled = val & 0x80 != 0;
    }

    pub fn write_ppu_mask(&mut self, val: u8) {
        self.greyscale_enabled = val & 0x01 != 0;
        self.show_left_background = val & 0x02 != 0;
        self.show_left_sprites = val & 0x04 != 0;
        self.show_background = val & 0x08 != 0;
        self.show_sprites = val & 0x10 != 0;
        self.emphasize_red = val & 0x20 != 0;
        self.emphasize_green = val & 0x40 != 0;
        self.emphasize_blue = val & 0x80 != 0;
    }
}
