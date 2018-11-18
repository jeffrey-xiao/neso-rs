#[cfg_attr(not(target_arch = "wasm32"), derive(Deserialize, Serialize))]
pub struct Registers {
    pub high_tile_byte: u8,
    pub low_tile_byte: u8,
    pub nametable_byte: u8,
    pub palette: u8,
    pub tile: u64, // 16 4 bits of data describing a pixel

    pub v: u16, // 15 bits
    pub t: u16, // 15 bits
    pub x: u8,  //  3 bits
    pub w: u8,  //  1 bit

    // PPUCTRL
    pub nametable_address: u16,
    pub vram_address_increment: u16,
    pub sprite_pattern_table_address: u16,
    pub background_pattern_table_address: u16,
    pub sprite_size: (u8, u8),
    pub is_master: bool,
    pub nmi_enabled: bool,

    // PPUMASK
    pub greyscale_enabled: bool,
    pub show_left_background: bool,
    pub show_left_sprites: bool,
    pub show_background: bool,
    pub show_sprites: bool,
    pub emphasize_red: bool,
    pub emphasize_green: bool,
    pub emphasize_blue: bool,
    pub rendering_enabled: bool,

    // PPUSTATUS
    pub sprite_overflow: bool,
    pub sprite_0_hit: bool,
    pub v_blank_started: bool,

    // OAMADDR
    pub oam_addr: u8,

    // PPUSCROLL
    pub scroll_x: u8,
    pub scroll_y: u8,

    // PPUDATA
    pub bus_address: u16,
    pub buffer: u8,

    pub last_written_byte: u8,
}

const NAMETABLE_ADDRESSES: [u16; 4] = [0x2000, 0x2400, 0x2800, 0x2C00];
impl Registers {
    // TODO: check if there are defaults
    pub fn new() -> Registers {
        Registers {
            high_tile_byte: 0,
            low_tile_byte: 0,
            nametable_byte: 0,
            palette: 0,
            tile: 0,

            v: 0,
            t: 0,
            x: 0,
            w: 0,

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
            rendering_enabled: false,

            // PPUSTATUS
            sprite_overflow: false,
            sprite_0_hit: false,
            v_blank_started: false,

            // OAMADDR
            oam_addr: 0,

            // PPUSCROLL
            scroll_x: 0,
            scroll_y: 0,

            // PPUDATA
            bus_address: 0,
            buffer: 0,

            last_written_byte: 0,
        }
    }

    pub fn increment_scroll_x(&mut self) {
        if (self.v & 0x001F) == 31 {
            self.v &= !0x001F;
            self.v ^= 0x0400;
        } else {
            self.v += 1;
        }
    }

    pub fn increment_scroll_y(&mut self) {
        if (self.v & 0x7000) != 0x7000 {
            self.v += 0x1000;
        } else {
            self.v &= !0x7000;
            let mut y = (self.v & 0x03E0) >> 5;
            if y == 29 {
                y = 0;
                self.v ^= 0x0800;
            } else if y == 31 {
                y = 0;
            } else {
                y += 1;
            }
            self.v = (self.v & !0x03E0) | (y << 5);
        }
    }

    pub fn copy_scroll_x(&mut self) {
        self.v = (self.v & 0xFBE0) | (self.t & 0x041F)
    }

    pub fn copy_scroll_y(&mut self) {
        self.v = (self.v & 0x841F) | (self.t & 0x7BE0)
    }

    pub fn read_ppu_status(&mut self) -> u8 {
        let ret = (self.last_written_byte & 0x1F)
            | if self.sprite_overflow { 0x20 } else { 0 }
            | if self.sprite_0_hit { 0x40 } else { 0 }
            | if self.v_blank_started { 0x80 } else { 0 };
        self.v_blank_started = false;
        self.w = 0;
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
        self.t = (self.t & !0x0C00) | ((u16::from(val) & 0x03) << 10);
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
        self.rendering_enabled = self.show_background || self.show_sprites;
    }

    pub fn write_ppu_scroll(&mut self, val: u8) {
        if self.w == 0 {
            self.t = (self.t & !0x001F) | (u16::from(val) >> 3);
            self.x = val & 0x07;
            self.w = 1;
        } else {
            self.t = (self.t & !0x73E0)
                | ((u16::from(val) & 0x07) << 12)
                | ((u16::from(val) & 0xF8) << 2);
            self.w = 0;
        }
    }

    pub fn write_ppu_addr(&mut self, val: u8) {
        if self.w == 0 {
            self.t = (self.t & !0x7F00) | ((u16::from(val) & 0x3F) << 8);
            self.w = 1;
        } else {
            self.t = (self.t & !0x00FF) | u16::from(val);
            self.v = self.t;
            self.bus_address = self.t & 0x3FFF;
            self.w = 0;
        }
    }
}

impl Default for Registers {
    fn default() -> Self {
        Registers::new()
    }
}
