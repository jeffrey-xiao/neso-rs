use cartridge::Cartridge;
use mapper::Mapper;
use ppu::MirroringMode;

pub struct NROM {
    cartridge: Cartridge,
}

impl NROM {
    pub fn new(cartridge: Cartridge) -> Self {
        NROM { cartridge }
    }
}

impl Mapper for NROM {
    fn read_byte(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        match addr {
            0x0000..=0x1FFF => self.cartridge.read_chr_rom(addr),
            0x6000..=0x7FFF => {
                let addr = (addr - 0x6000) % self.cartridge.prg_ram_len();
                self.cartridge.read_prg_ram(addr)
            },
            0x8000..=0xFFFF => {
                let addr = (addr - 0x8000) % self.cartridge.prg_rom_len();
                self.cartridge.read_prg_rom(addr)
            },
            _ => 0,
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        let addr = addr as usize;
        match addr {
            0x0000..=0x1FFF => self.cartridge.write_chr_rom(addr, val),
            0x6000..=0x7FFF => {
                let addr = (addr - 0x6000) % self.cartridge.prg_ram_len();
                self.cartridge.write_prg_ram(addr, val);
            },
            _ => {},
        }
    }

    fn mirroring_mode(&self) -> MirroringMode {
        self.cartridge.mirroring_mode
    }
}
