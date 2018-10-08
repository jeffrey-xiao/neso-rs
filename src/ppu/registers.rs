pub struct Registers {
    // PPUCTRL
    pub nametable_address: u16,
    pub vram_address_increment: u16,
    pub sprite_pattern_table_address: u16,
    pub background_pattern_table_address: u16,
    pub sprite_size: (u16, u16),
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

    // PPUSTATUS
    pub sprite_overflow: bool,
    pub sprite_0_hit: bool,
    pub v_blank_started: bool,

    // OAMADDR
    pub oam_addr: u8,

    // PPUSCROLL
    pub scroll_x: u8,
    pub scroll_y: u8,

    // PPUADDR
    pub ppu_addr: u16,

    // PPUDATA
    pub buffer: u8,

    pub last_written_byte: u8,
    pub address_latch: bool,
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

    pub fn read_ppu_status(&mut self) -> u8 {
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
