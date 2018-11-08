use cartridge::Cartridge;
use mapper::Mapper;
use ppu::MirroringMode;

pub struct ColorDreams {
    cartridge: Cartridge,
    prg_rom_bank: u8,
    chr_rom_bank: u8,
}

impl ColorDreams {
    pub fn new(cartridge: Cartridge) -> Self {
        ColorDreams {
            cartridge,
            prg_rom_bank: 0,
            chr_rom_bank: 0,
        }
    }
}

impl Mapper for ColorDreams {
    fn read_byte(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        match addr {
            0x0000..=0x1FFF => {
                let addr = self.chr_rom_bank as usize * 0x2000 + addr;
                self.cartridge.read_chr_rom(addr)
            },
            0x8000..=0xFFFF => {
                let addr = self.prg_rom_bank as usize * 0x8000 + addr - 0x8000;
                self.cartridge.read_prg_rom(addr)
            },
            _ => 0,
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        let addr = addr as usize;
        match addr {
            0x0000..=0x1FFF => {
                let addr = self.chr_rom_bank as usize * 0x2000 + addr;
                self.cartridge.write_chr_rom(addr, val);
            },
            0x8000..=0xFFFF => {
                self.prg_rom_bank = val & 0x03;
                println!("[ColorDreams] Write prg rom bank: {}.", self.prg_rom_bank);
                self.chr_rom_bank = (val >> 4) & 0x0F;
                println!("[ColorDreams] Write chr rom bank: {}.", self.chr_rom_bank);
            },
            _ => {},
        }
    }

    fn chr_bank(&self, index: usize) -> *const u8 {
        self.cartridge.chr_bank(self.chr_rom_bank as usize * 8 + index)
    }

    fn mirroring_mode(&self) -> MirroringMode {
        self.cartridge.mirroring_mode
    }
}
