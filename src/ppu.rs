use std::rc::{Rc, Weak};
use std::cell::RefCell;
use mapper::Mapper;

pub enum MirroringMode {
    Horizontal = 0,
    Vertical = 1,
    None = 2,
}
const MIRRORING_MODE_TABLE: [usize; 12] = [
    0, 0, 1, 1, // Horizontal
    0, 1, 0, 1, // Vertical
    0, 1, 2, 3, // None
];

pub struct Ppu {
    pub memory_map: Option<MemoryMap>,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            memory_map: None,
        }
    }

    pub fn attach_memory_map(&mut self, memory_map: MemoryMap) {
        self.memory_map = Some(memory_map);
    }

    pub fn memory_map(&self) -> &MemoryMap {
        self.memory_map.as_ref().unwrap()
    }

    pub fn memory_map_mut(&mut self) -> &mut MemoryMap {
        self.memory_map.as_mut().unwrap()
    }

    pub fn read_register(&self, index: usize) -> u8 {
        self.memory_map().registers.read_register(index)
    }

    pub fn write_register(&mut self, index: usize, val: u8) {
        self.memory_map_mut().registers.write_register(index, val);
    }
}

pub struct MemoryMap {
    pub registers: Registers,
    pub oam: [u8; 0x100],
    pub vram: [u8; 0x2000],
    pub palette_ram: [u8; 0x20],
    mapper: Weak<RefCell<Box<Mapper>>>,
}

impl MemoryMap {
    pub fn new(mapper: &Rc<RefCell<Box<Mapper>>>) -> Self {
        MemoryMap {
            registers: Registers::new(),
            oam: [0; 0x100],
            vram: [0; 0x2000],
            palette_ram: [0; 0x20],
            mapper: Rc::downgrade(mapper),
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                let mapper = self.mapper.upgrade().unwrap();
                let ret = mapper.borrow().read_byte(addr);
                ret
            },
            0x2000..=0x3EFF => {
                let index = ((addr - 0x2000) / 0x400) as usize;
                let offset = ((addr - 0x2000) % 0x400) as usize;
                self.vram[MIRRORING_MODE_TABLE[index] * 0x400 + offset]
            },
            0x3F00..=0x3FFF => self.palette_ram[((addr - 0x3F00) % 0x20) as usize],
            _ => panic!("Invalid memory address: {:#6x}.", addr),
        }
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => {
                let mapper = self.mapper.upgrade().unwrap();
                mapper.borrow_mut().read_byte(addr);
            },
            0x2000..=0x3EFF => {
                let index = ((addr - 0x2000) / 0x400) as usize;
                let offset = ((addr - 0x2000) % 0x400) as usize;
                self.vram[MIRRORING_MODE_TABLE[index] * 0x400 + offset] = val;
            },
            0x3F00..=0x3FFF => self.palette_ram[((addr - 0x3F00) % 0x20) as usize] = val,
            _ => panic!("Invalid memory address: {:#6x}.", addr),
        }
    }
}

pub struct Registers {
    data: [u8; 8],
    last_written_byte: u8,
}

impl Registers {
    pub fn new() -> Registers {
        Registers { data: [0; 8], last_written_byte: 0 }
    }

    pub fn read_register(&self, index: usize) -> u8 {
        match index {
            0 => self.last_written_byte,
            1 => self.last_written_byte,
            2 => self.data[index],
            3 => self.data[index],
            4 => self.data[index],
            5 => self.last_written_byte,
            6 => self.last_written_byte,
            7 => self.data[index],
            _ => panic!("Invalid index to read."),
        }
    }

    pub fn write_register(&mut self, index: usize, val: u8) {
        self.last_written_byte = val;
        if index != 2 {
            self.data[index] = val
        }
    }
}

pub enum RegisterName {
    PpuCtrl = 0,
    PpuMask = 1,
    PpuStatus = 2,
    OamAddr = 3,
    OamData = 4,
    PpuScroll = 5,
    PpuAddr = 6,
    PpuData = 7,
}
