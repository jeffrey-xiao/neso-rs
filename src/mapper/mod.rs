mod axrom;
mod cnrom;
mod color_dreams;
mod mmc1;
mod mmc3;
mod nrom;
mod uxrom;

use self::axrom::AxROM;
use self::cnrom::CNROM;
use self::color_dreams::ColorDreams;
use self::mmc1::MMC1;
use self::mmc3::MMC3;
use self::nrom::NROM;
use self::uxrom::UxROM;
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
        11 => Box::new(ColorDreams::new(cartridge)),
        94 => Box::new(UxROM::new(cartridge, uxrom::Variant::UN1ROM)),
        180 => Box::new(UxROM::new(cartridge, uxrom::Variant::Mapper180)),
        _ => panic!("[MAPPER] Unsupported mapper: {}.", cartridge.mapper),
    }
}

pub trait Mapper {
    fn read_byte(&self, addr: u16) -> u8;
    fn write_byte(&mut self, addr: u16, val: u8);
    fn chr_bank(&self, index: usize) -> *const u8;
    fn mirroring_mode(&self) -> MirroringMode;
    fn attach_bus(&mut self, _bus: Bus) {}
    fn step(&mut self) {}
    #[cfg(not(target_arch = "wasm32"))]
    fn save(&self) -> bincode::Result<Option<Vec<u8>>>;
    #[cfg(not(target_arch = "wasm32"))]
    fn load(&mut self, save_data: &[u8]) -> bincode::Result<()>;
    #[cfg(not(target_arch = "wasm32"))]
    fn save_state(&self) -> bincode::Result<Vec<u8>>;
    #[cfg(not(target_arch = "wasm32"))]
    fn load_state(&mut self, mapper_data: &[u8], save_data_opt: Option<Vec<u8>>) -> bincode::Result<()>;
}
