mod nrom;
mod mmc1;

pub use self::nrom::Nrom;
pub use self::mmc1::Mmc1;
use cartridge::Cartridge;
use ppu::MirroringMode;

pub fn from_cartridge(cartridge: Cartridge) -> Box<dyn Mapper> {
    match cartridge.mapper {
        0 => Box::new(Nrom::new(cartridge)),
        1 => Box::new(Mmc1::new(cartridge)),
        _ => panic!("Unsupported mapper: {}.", cartridge.mapper),
    }
}

pub trait Mapper {
    fn read_byte(&self, addr: u16) -> u8;
    fn write_byte(&mut self, addr: u16, val: u8);
    fn mirroring_mode(&self) -> MirroringMode;
}
