use cpu::Cpu;
use mapper::Mapper;
use ppu::Ppu;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct Bus {
    pub cpu: Weak<RefCell<Cpu>>,
    pub ppu: Weak<RefCell<Ppu>>,
    pub mapper: Weak<RefCell<Box<Mapper>>>,
}

impl Bus {
    pub fn new(
        cpu: &Rc<RefCell<Cpu>>,
        ppu: &Rc<RefCell<Ppu>>,
        mapper: &Rc<RefCell<Box<Mapper>>>,
    ) -> Self {
        Bus {
            cpu: Rc::downgrade(cpu),
            ppu: Rc::downgrade(ppu),
            mapper: Rc::downgrade(mapper),
        }
    }

    pub fn cpu(&self) -> Rc<RefCell<Cpu>> {
        self.cpu.upgrade().unwrap()
    }

    pub fn ppu(&self) -> Rc<RefCell<Ppu>> {
        self.ppu.upgrade().unwrap()
    }

    pub fn mapper(&self) -> Rc<RefCell<Box<Mapper>>> {
        self.mapper.upgrade().unwrap()
    }
}
