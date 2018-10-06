use mapper::{Mapper, self};
use cartridge::Cartridge;

pub struct Memory {
    ram: [u8; 0x800],
    mapper: Option<Box<Mapper>>,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            ram: [0; 0x800],
            mapper: None,
        }
    }

    pub fn load_rom(&mut self, buffer: &[u8]) {
        let cartridge = Cartridge::from_buffer(buffer);
        self.mapper = Some(Box::new(mapper::from_cartridge(cartridge)));
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        // println!("READ AT {:x}", addr);
        let mapper = match self.mapper {
            Some(ref mapper) => mapper,
            None => panic!("No cartridge loaded."),
        };

        match addr {
            0x0000..=0x1FFF => self.ram[(addr % 0x0800) as usize],
            0x2000..=0x3FFF => panic!("PPU registers not implemented."),
            0x4000..=0x4017 => panic!("APU and IO registers not implemented."),
            0x4018..=0x401F => panic!("CPU Test Mode not implemented."),
            0x4020..=0xFFFE => mapper.read_byte(addr),
            _ => panic!(format!("Invalid memory address: {:#6x}.", addr)),
        }
    }

    pub fn read_word(&self, addr: u16) -> u16 {
        ((self.read_byte(addr + 1) as u16) << 8) | self.read_byte(addr) as u16
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        // println!("WRITE AT {:x} WITH {:x}", addr, val);
        let mapper = match self.mapper {
            Some(ref mut mapper) => mapper,
            None => panic!("No cartridge loaded."),
        };

        match addr {
            0x0000..=0x1FFF => self.ram[(addr % 0x0800) as usize] = val,
            0x2000..=0x3FFF => panic!("PPU registers not implemented."),
            0x4000..=0x4017 => panic!("APU and IO registers not implemented."),
            0x4018..=0x401F => panic!("CPU Test Mode not implemented."),
            0x4020..=0xFFFE => mapper.write_byte(addr, val),
            _ => panic!(format!("Invalid memory address: {:#6x}.", addr)),
        }
    }
}
