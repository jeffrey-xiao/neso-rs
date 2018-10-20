mod mmc1;
mod nrom;
mod uxrom;
mod cnrom;

pub use self::mmc1::MMC1;
pub use self::nrom::NROM;
pub use self::uxrom::UxROM;
pub use self::cnrom::CNROM;
use cartridge::Cartridge;
use ppu::MirroringMode;

pub fn from_cartridge(cartridge: Cartridge) -> Box<dyn Mapper> {
    match cartridge.mapper {
        0 => Box::new(NROM::new(cartridge)),
        1 => Box::new(MMC1::new(cartridge)),
        2 => Box::new(UxROM::new(cartridge)),
        3 => Box::new(CNROM::new(cartridge)),
        _ => panic!("[MAPPER] Unsupported mapper: {}.", cartridge.mapper),
    }
}

pub trait Mapper {
    fn read_byte(&self, addr: u16) -> u8;
    fn write_byte(&mut self, addr: u16, val: u8);
    fn mirroring_mode(&self) -> MirroringMode;
}
