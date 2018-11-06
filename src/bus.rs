use apu::Apu;
use cpu::Cpu;
use mapper::Mapper;
use ppu::Ppu;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Clone)]
pub struct Bus {
    pub apu: Weak<RefCell<Apu>>,
    pub cpu: Weak<RefCell<Cpu>>,
    pub ppu: Weak<RefCell<Ppu>>,
    pub mapper: Weak<RefCell<Box<Mapper>>>,
}

impl Bus {
    pub fn new(
        apu: &Rc<RefCell<Apu>>,
        cpu: &Rc<RefCell<Cpu>>,
        ppu: &Rc<RefCell<Ppu>>,
        mapper: &Rc<RefCell<Box<Mapper>>>,
    ) -> Self {
        Bus {
            apu: Rc::downgrade(apu),
            cpu: Rc::downgrade(cpu),
            ppu: Rc::downgrade(ppu),
            mapper: Rc::downgrade(mapper),
        }
    }

    pub fn apu(&self) -> Rc<RefCell<Apu>> {
        self.apu.upgrade().expect("Expected APU to exist on bus.")
    }

    pub fn cpu(&self) -> Rc<RefCell<Cpu>> {
        self.cpu.upgrade().expect("Expected CPU to exist on bus.")
    }

    pub fn ppu(&self) -> Rc<RefCell<Ppu>> {
        self.ppu.upgrade().expect("Expected PPU to exist on bus.")
    }

    pub fn mapper(&self) -> Rc<RefCell<Box<Mapper>>> {
        self.mapper.upgrade().expect("Expected mapper to exist on bus.")
    }
}
