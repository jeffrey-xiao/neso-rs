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
    fn read(addr: u16) -> u8 { 0 }
    fn write(addr: u16, val: u8) {}
}
