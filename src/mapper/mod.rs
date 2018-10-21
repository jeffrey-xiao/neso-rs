mod cnrom;
mod mmc1;
mod mmc3;
mod nrom;
mod uxrom;
mod axrom;

pub use self::cnrom::CNROM;
pub use self::mmc1::MMC1;
pub use self::mmc3::MMC3;
pub use self::nrom::NROM;
pub use self::uxrom::UxROM;
pub use self::axrom::AxROM;
use bus::Bus;
use cartridge::Cartridge;
use ppu::MirroringMode;

pub fn from_cartridge(cartridge: Cartridge) -> Box<dyn Mapper> {
    match cartridge.mapper {
        0 => Box::new(NROM::new(cartridge)),
        1 => Box::new(MMC1::new(cartridge)),
        2 => Box::new(UxROM::new(cartridge, uxrom::Variant::UNROM)),
        3 => Box::new(CNROM::new(cartridge)),
        4 => Box::new(MMC3::new(cartridge)),
        7 => Box::new(AxROM::new(cartridge)),
        94 => Box::new(UxROM::new(cartridge, uxrom::Variant::UN1ROM)),
        180 => Box::new(UxROM::new(cartridge, uxrom::Variant::Mapper180)),
        _ => panic!("[MAPPER] Unsupported mapper: {}.", cartridge.mapper),
    }
}

pub trait Mapper {
    fn read_byte(&self, addr: u16) -> u8;
    fn write_byte(&mut self, addr: u16, val: u8);
    fn mirroring_mode(&self) -> MirroringMode;
    fn attach_bus(&mut self, _bus: Bus) {}
    fn step(&mut self) {}
}
