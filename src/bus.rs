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
    pub fn new(apu: &mut Apu, cpu: &mut Cpu, ppu: &mut Ppu, mapper: *mut Mapper) -> Self {
        Bus {
            apu: apu as *mut Apu,
            cpu: cpu as *mut Cpu,
            ppu: ppu as *mut Ppu,
            mapper,
        }
    }

    pub fn apu(&self) -> &Apu {
        unsafe { &(*self.apu) }
    }

    pub fn apu_mut(&mut self) -> &mut Apu {
        unsafe { &mut (*self.apu) }
    }

    pub fn cpu(&self) -> &Cpu {
        unsafe { &(*self.cpu) }
    }

    pub fn cpu_mut(&mut self) -> &mut Cpu {
        unsafe { &mut (*self.cpu) }
    }

    pub fn ppu(&self) -> &Ppu {
        unsafe { &(*self.ppu) }
    }

    pub fn ppu_mut(&mut self) -> &mut Ppu {
        unsafe { &mut (*self.ppu) }
    }

    pub fn mapper(&self) -> &Mapper {
        unsafe { &(*self.mapper) }
    }

    pub fn mapper_mut(&mut self) -> &mut Mapper {
        unsafe { &mut (*self.mapper) }
    }
}
