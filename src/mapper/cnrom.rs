use cartridge::Cartridge;
use mapper::Mapper;
use ppu::MirroringMode;

pub struct CNROM {
    cartridge: Cartridge,
    chr_rom_bank: u8,
}

impl CNROM {
    pub fn new(cartridge: Cartridge) -> Self {
        CNROM {
            cartridge,
            chr_rom_bank: 0,
        }
    }
}

impl Mapper for CNROM {
    fn read_byte(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        match addr {
            0x0000..=0x1FFF => {
                let addr = self.chr_rom_bank as usize * 0x2000 + addr;
                self.cartridge.read_chr_rom(addr)
            },
            0x8000..=0xFFFF if self.cartridge.prg_rom_len() == 0x8000 => {
                self.cartridge.read_prg_rom(addr - 0x8000)
            },
            0x8000..=0xFFFF => self.cartridge.read_prg_rom((addr - 0x8000) % 0x4000),
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
            0x8000..=0xFFFF => self.chr_rom_bank = val & 0x03,
            _ => {},
        }
    }

    fn mirroring_mode(&self) -> MirroringMode {
        self.cartridge.mirroring_mode
    }
}
