use apu::Apu;
use cpu::Cpu;
use mapper::Mapper;
use ppu::Ppu;

#[derive(Clone)]
pub struct Bus {
    pub apu: *mut Apu,
    pub cpu: *mut Cpu,
    pub ppu: *mut Ppu,
    pub mapper: *mut Mapper,
}

impl Bus {
    pub fn new(
        apu: &mut Apu,
        cpu: &mut Cpu,
        ppu: &mut Ppu,
        mapper: Box<Mapper>,
    ) -> Self {
        Bus {
            apu: apu as *mut Apu,
            cpu: cpu as *mut Cpu,
            ppu: ppu as *mut Ppu,
            mapper: Box::into_raw(mapper),
        }
    }

    pub fn apu(&self) -> &mut Apu {
        unsafe { &mut (*self.apu) }
    }

    pub fn cpu(&self) -> &mut Cpu {
        unsafe { &mut (*self.cpu) }
    }

    pub fn ppu(&self) -> &mut Ppu {
        unsafe { &mut (*self.ppu) }
    }

    pub fn mapper(&self) -> &mut Mapper {
        unsafe { &mut (*self.mapper) }
    }
}
