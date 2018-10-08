mod memory;
mod registers;

pub use self::memory::MemoryMap;

pub enum MirroringMode {
    Horizontal = 0,
    Vertical = 1,
    None = 2,
}

pub struct Ppu {
    pub memory_map: Option<MemoryMap>,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu { memory_map: None }
    }

    pub fn reset(&mut self) {
        self.memory_map_mut().r.write_ppu_ctrl(0);
        self.memory_map_mut().r.write_ppu_mask(0);
        self.memory_map_mut().r.oam_addr = 0;
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

    pub fn read_register(&mut self, index: usize) -> u8 {
        self.memory_map_mut().read_register(index)
    }

    pub fn write_register(&mut self, index: usize, val: u8) {
        self.memory_map_mut().write_register(index, val);
    }
}


