pub struct Ppu {
    pub registers: Registers,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            registers: Registers::new(),
        }
    }

    pub fn read_register(&self, index: usize) -> u8 {
        self.registers.read_register(index)
    }

    pub fn write_register(&mut self, index: usize, val: u8) {
        self.registers.write_register(index, val)
    }
}

pub struct Registers {
    data: [u8; 8],
}

impl Registers {
    pub fn new() -> Registers {
        Registers { data: [0; 8] }
    }

    pub fn read_register(&self, index: usize) -> u8 {
        self.data[index]
    }

    pub fn write_register(&mut self, index: usize, val: u8) {
        self.data[index] = val
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
