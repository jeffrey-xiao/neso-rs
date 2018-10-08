mod nrom;

pub use self::nrom::Nrom;
use cartridge::Cartridge;

pub fn from_cartridge(cartridge: Cartridge) -> Box<Mapper> {
    match cartridge.mapper {
        0 => Box::new(Nrom::new(cartridge)),
        _ => panic!("Unsupported mapper: {}.", cartridge.mapper),
    }
}

pub trait Mapper {
    fn read_byte(&self, addr: u16) -> u8;
    fn write_byte(&mut self, addr: u16, val: u8);
}
