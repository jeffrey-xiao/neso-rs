use cartridge::Cartridge;
use mapper::Mapper;
use ppu::MirroringMode;

pub struct UxROM {
    cartridge: Cartridge,
    prg_rom_bank: u8,
}

impl UxROM {
    pub fn new(cartridge: Cartridge) -> Self {
        UxROM {
            cartridge,
            prg_rom_bank: 0,
        }
    }
}

impl Mapper for UxROM {
    fn read_byte(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        match addr {
            0x0000..=0x1FFF => self.cartridge.read_chr_rom(addr),
            0x8000..=0xBFFF => {
                let addr = self.prg_rom_bank as usize * 0x4000 + addr - 0x8000;
                self.cartridge.read_prg_rom(addr)
            },
            0xC000..=0xFFFF => {
                let bank = self.cartridge.prg_rom_len() / 0x4000 - 1;
                let addr = bank * 0x4000 + addr - 0xC000;
                self.cartridge.read_prg_rom(addr)
            },
            _ => 0,
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        if addr < 0x8000 {
            return;
        }
        let addr = addr as usize;
        match addr {
            0x0000..=0x1FFF => self.cartridge.write_chr_rom(addr, val),
            0xC000..=0xFFFF => self.prg_rom_bank = val & 0x07,
            _ => {},
        }
    }

    fn mirroring_mode(&self) -> MirroringMode {
        self.cartridge.mirroring_mode
    }
}
