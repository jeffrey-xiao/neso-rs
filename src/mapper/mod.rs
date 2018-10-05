mod nrom;

pub use self::nrom::Nrom;
use cartridge::Cartridge;

fn from_cartridge(cartridge: Cartridge) -> impl Mapper {
    match cartridge.mapper() {
        0 => Nrom::new(cartridge),
        _ => panic!(format!("Unsupported mapper: {}.", cartridge.mapper())),
    }
}

trait Mapper {
    fn read(addr: u16) -> u8;
    fn write(addr: u16, val: u8);
}
