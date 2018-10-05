use mapper::Mapper;
use cartridge::Cartridge;

pub struct Nrom {
    cartridge: Cartridge,
}

impl Nrom {
    pub fn new(cartridge: Cartridge) -> Self {
        Nrom {
            cartridge,
        }
    }
}

impl Mapper for Nrom {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.cartridge.read_chr_rom(addr),
            0x6000..=0x7FFF => {
                let addr = (addr - 0x6000) % self.cartridge.prg_ram_len() as u16;
                self.cartridge.read_prg_ram(addr)
            },
            0x8000..=0xBFFF => self.cartridge.read_prg_rom(addr - 0x8000),
            0xC000..=0xFFFF => {
                if self.cartridge.prg_rom_len() == 0x4000 {
                    self.cartridge.read_prg_rom(addr - 0xC000)
                } else {
                    self.cartridge.read_prg_rom(addr - 0x8000)
                }
            },
            _ => panic!("Invalid memory address to read."),
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => self.cartridge.write_chr_rom(addr, val),
            _ => panic!("Invalid memory address to write."),
        }
    }
}
